use crate::error::FormatError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

pub const IMG_SECTOR_SIZE: u64 = 2048;

/// Directory entry inside a GTA: San Andreas IMG (`VER2`) archive.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImgEntry {
    pub name: String,
    pub offset_sectors: u32,
    pub streaming_sectors: u16,
    pub archive_sectors: u16,
}

impl ImgEntry {
    /// Byte offset inside the `.img` archive.
    pub fn byte_offset(&self) -> u64 {
        u64::from(self.offset_sectors) * IMG_SECTOR_SIZE
    }

    /// Sector count used when reading from disk.
    /// In SA `VER2` archives, `archive_sectors` is often `0`, meaning `streaming_sectors` holds the size.
    pub fn effective_sectors(&self) -> u64 {
        let count = if self.archive_sectors > 0 {
            self.archive_sectors
        } else {
            self.streaming_sectors
        };
        u64::from(count)
    }

    /// Maximum byte size allocated for this entry in the archive.
    pub fn byte_size(&self) -> u64 {
        self.effective_sectors() * IMG_SECTOR_SIZE
    }
}

/// Streaming reader for GTA: San Andreas `VER2` `.img` archives (e.g., `gta3.img`, `gta_int.img`).
#[derive(Debug)]
pub struct ImgArchive {
    path: PathBuf,
    entries: Vec<ImgEntry>,
    by_lower_name: HashMap<String, usize>,
}

impl ImgArchive {
    /// Open an IMG archive and index its directory without reading entry payloads into memory.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, FormatError> {
        let path_buf = path.as_ref().to_path_buf();
        let mut file = File::open(&path_buf).map_err(|e| FormatError::Io {
            path: path_buf.clone(),
            source: e,
        })?;

        let mut header = [0u8; 8];
        file.read_exact(&mut header).map_err(|e| FormatError::Io {
            path: path_buf.clone(),
            source: e,
        })?;

        if &header[0..4] != b"VER2" {
            return Err(FormatError::InvalidMagic {
                expected: "VER2".to_string(),
                actual: String::from_utf8_lossy(&header[0..4]).into_owned(),
            });
        }

        let entry_count = u32::from_le_bytes([header[4], header[5], header[6], header[7]]) as usize;
        let file_len = file
            .metadata()
            .map_err(|e| FormatError::Io {
                path: path_buf.clone(),
                source: e,
            })?
            .len();

        let dir_bytes_len = entry_count
            .checked_mul(32)
            .ok_or_else(|| FormatError::Malformed {
                context: "IMG directory".to_string(),
                detail: "entry count overflow".to_string(),
            })?;
        if (dir_bytes_len as u64) + 8 > file_len {
            return Err(FormatError::Malformed {
                context: "IMG directory".to_string(),
                detail: format!("directory size {dir_bytes_len} exceeds file length {file_len}"),
            });
        }

        let mut dir_buf = vec![0u8; dir_bytes_len];
        file.read_exact(&mut dir_buf).map_err(|e| FormatError::Io {
            path: path_buf.clone(),
            source: e,
        })?;

        let mut entries = Vec::with_capacity(entry_count);
        let mut by_lower_name = HashMap::with_capacity(entry_count);

        for (idx, chunk) in dir_buf.as_chunks::<32>().0.iter().enumerate() {
            let offset_sectors = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            let streaming_sectors = u16::from_le_bytes([chunk[4], chunk[5]]);
            let archive_sectors = u16::from_le_bytes([chunk[6], chunk[7]]);
            let name_bytes = &chunk[8..32];
            let nul = name_bytes.iter().position(|&b| b == 0).unwrap_or(24);
            let name = String::from_utf8_lossy(&name_bytes[..nul])
                .trim()
                .to_string();
            by_lower_name.insert(name.to_ascii_lowercase(), idx);
            entries.push(ImgEntry {
                name,
                offset_sectors,
                streaming_sectors,
                archive_sectors,
            });
        }

        Ok(Self {
            path: path_buf,
            entries,
            by_lower_name,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn entries(&self) -> &[ImgEntry] {
        &self.entries
    }

    pub fn get_entry(&self, name: &str) -> Option<&ImgEntry> {
        self.by_lower_name
            .get(&name.to_ascii_lowercase())
            .map(|&idx| &self.entries[idx])
    }

    /// Read an entry's raw bytes directly from disk by case-insensitive filename.
    pub fn read_entry(&self, name: &str) -> Result<Vec<u8>, FormatError> {
        let entry = self
            .get_entry(name)
            .ok_or_else(|| FormatError::EntryNotFound(name.to_string()))?;
        self.read_entry_data(entry)
    }

    /// Read an entry's raw bytes using an indexed `ImgEntry`.
    pub fn read_entry_data(&self, entry: &ImgEntry) -> Result<Vec<u8>, FormatError> {
        let mut file = File::open(&self.path).map_err(|e| FormatError::Io {
            path: self.path.clone(),
            source: e,
        })?;
        let file_len = file
            .metadata()
            .map_err(|e| FormatError::Io {
                path: self.path.clone(),
                source: e,
            })?
            .len();

        let offset = entry.byte_offset();
        let size = entry.byte_size();
        if offset >= file_len {
            return Err(FormatError::Malformed {
                context: format!("IMG entry '{}'", entry.name),
                detail: format!("offset {offset} beyond file length {file_len}"),
            });
        }
        let clamped_size = size.min(file_len - offset) as usize;
        let mut buf = vec![0u8; clamped_size];
        file.seek(SeekFrom::Start(offset))
            .map_err(|e| FormatError::Io {
                path: self.path.clone(),
                source: e,
            })?;
        file.read_exact(&mut buf).map_err(|e| FormatError::Io {
            path: self.path.clone(),
            source: e,
        })?;
        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_synthetic_ver2_img() {
        let dir = tempfile::tempdir().unwrap();
        let img_path = dir.path().join("test.img");

        let mut buf = vec![0u8; 4096];
        buf[0..4].copy_from_slice(b"VER2");
        buf[4..8].copy_from_slice(&1u32.to_le_bytes());
        // Entry 0 at sector 1, streaming 1 sector, archive 0
        buf[8..12].copy_from_slice(&1u32.to_le_bytes());
        buf[12..14].copy_from_slice(&1u16.to_le_bytes());
        buf[14..16].copy_from_slice(&0u16.to_le_bytes());
        let name = b"hello.txt\0";
        buf[16..16 + name.len()].copy_from_slice(name);
        buf[2048..2053].copy_from_slice(b"GTASA");
        std::fs::write(&img_path, &buf).unwrap();

        let archive = ImgArchive::open(&img_path).unwrap();
        assert_eq!(archive.entries().len(), 1);
        let payload = archive.read_entry("HELLO.TXT").unwrap();
        assert_eq!(&payload[0..5], b"GTASA");
    }
}
