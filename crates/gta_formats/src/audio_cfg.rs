use crate::FormatError;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

/// Rotating 16-byte XOR key used by GTA: San Andreas `audio/streams/*` archives.
pub const SA_STREAM_XOR_KEY: [u8; 16] = [
    0xEA, 0x3A, 0xC4, 0xA1, 0x9A, 0xA8, 0x14, 0xF3, 0x48, 0xB0, 0xD7, 0x23, 0x9D, 0xE8, 0xFF, 0xF1,
];

/// Beat/timing header size preceding every Ogg Vorbis track inside `audio/streams/*`.
pub const SA_STREAM_HEADER_BYTES: u64 = 8068;

/// Fixed 4804-byte bank header size (`u32 sound_count` + 400 × 12-byte slots) inside `audio/SFX/*`.
pub const SA_SFX_BANK_HEADER_BYTES: u64 = 4804;

/// Metadata describing an external GTA:SA audio stream file (e.g., radio station or cutscene stream).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioStreamStatus {
    pub name: String,
    pub byte_len: u64,
    pub is_present: bool,
}

impl AudioStreamStatus {
    pub fn from_metadata(name: impl Into<String>, byte_len: u64) -> Self {
        Self {
            name: name.into(),
            byte_len,
            is_present: byte_len > 0,
        }
    }
}

/// Entry parsed from `audio/CONFIG/AudioEvents.txt` or `data/AudioEvents.txt`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioEventMapping {
    pub id: i32,
    pub name: String,
}

pub fn parse_audio_events(text: &str) -> Vec<AudioEventMapping> {
    let mut out = Vec::new();
    for line in text.lines() {
        let trimmed = line.split('#').next().unwrap_or("").trim();
        if trimmed.is_empty() {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        if let (Some(a), Some(b)) = (parts.next(), parts.next())
            && let Ok(id) = b.parse::<i32>()
        {
            out.push(AudioEventMapping {
                id,
                name: a.to_string(),
            });
        }
    }
    out
}

/// XOR-unscrambles (or scrambles) a slice taken at `file_offset` inside a GTA:SA stream pack.
pub fn decode_stream_slice(data: &mut [u8], file_offset: u64) {
    let base = (file_offset & 0x0F) as usize;
    for (idx, byte) in data.iter_mut().enumerate() {
        *byte ^= SA_STREAM_XOR_KEY[(base.wrapping_add(idx)) & 0x0F];
    }
}

/// Track entry decoded from `StrmPaks.dat` + `TrakLkup.dat`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackLookupEntry {
    pub track_index: usize,
    pub pack_index: u8,
    pub pack_name: String,
    pub offset: u32,
    pub size: u32,
}

/// Index of GTA:SA radio/ambience Ogg Vorbis streams inside `audio/streams/*`.
#[derive(Debug, Clone, Default)]
pub struct AudioStreamIndex {
    pub pack_names: Vec<String>,
    pub tracks: Vec<TrackLookupEntry>,
}

impl AudioStreamIndex {
    /// Parses `StrmPaks.dat` (16-byte fixed strings) and `TrakLkup.dat` (12-byte records).
    pub fn parse_bytes(strm_paks: &[u8], trak_lkup: &[u8]) -> Result<Self, FormatError> {
        let (pack_chunks, _) = strm_paks.as_chunks::<16>();
        let pack_names: Vec<String> = pack_chunks.iter().map(|c| read_ascii_z(c)).collect();
        if pack_names.is_empty() {
            return Err(FormatError::Malformed {
                context: "StrmPaks.dat".into(),
                detail: "no 16-byte pack entries found".into(),
            });
        }

        let (track_chunks, _) = trak_lkup.as_chunks::<12>();
        let mut tracks = Vec::with_capacity(track_chunks.len());
        for (track_index, rec) in track_chunks.iter().enumerate() {
            let pack_index = rec[0];
            let offset = u32::from_le_bytes([rec[4], rec[5], rec[6], rec[7]]);
            let size = u32::from_le_bytes([rec[8], rec[9], rec[10], rec[11]]);
            let pack_name = pack_names
                .get(usize::from(pack_index))
                .cloned()
                .unwrap_or_default();
            tracks.push(TrackLookupEntry {
                track_index,
                pack_index,
                pack_name,
                offset,
                size,
            });
        }

        Ok(Self { pack_names, tracks })
    }

    /// Loads `StrmPaks.dat` + `TrakLkup.dat` from `<game_root>/audio`.
    pub fn from_audio_dir(audio_root: &Path) -> Result<Self, FormatError> {
        let cfg_dir = resolve_ci_child(audio_root, "CONFIG").ok_or_else(|| {
            FormatError::EntryNotFound(format!("{}/CONFIG", audio_root.display()))
        })?;
        let strm_path = resolve_ci_child(&cfg_dir, "StrmPaks.dat")
            .ok_or_else(|| FormatError::EntryNotFound("StrmPaks.dat".into()))?;
        let trak_path = resolve_ci_child(&cfg_dir, "TrakLkup.dat")
            .ok_or_else(|| FormatError::EntryNotFound("TrakLkup.dat".into()))?;

        let strm_bytes = read_file_bytes(&strm_path)?;
        let trak_bytes = read_file_bytes(&trak_path)?;
        Self::parse_bytes(&strm_bytes, &trak_bytes)
    }

    /// Returns playable tracks belonging to `pack_name` whose file range fits on disk and starts with `OggS`.
    pub fn available_tracks_for_pack(
        &self,
        audio_root: &Path,
        pack_name: &str,
    ) -> Vec<&TrackLookupEntry> {
        let Some(streams_dir) = resolve_ci_child(audio_root, "streams") else {
            return Vec::new();
        };
        let Some(pack_path) = resolve_ci_child(&streams_dir, pack_name) else {
            return Vec::new();
        };
        let Ok(mut file) = File::open(&pack_path) else {
            return Vec::new();
        };
        let Ok(meta) = file.metadata() else {
            return Vec::new();
        };
        let file_len = meta.len();

        let mut available = Vec::new();
        for entry in &self.tracks {
            if !entry.pack_name.eq_ignore_ascii_case(pack_name) || entry.size < 64 {
                continue;
            }
            let ogg_offset = u64::from(entry.offset).saturating_add(SA_STREAM_HEADER_BYTES);
            let ogg_end = ogg_offset.saturating_add(u64::from(entry.size));
            if ogg_end > file_len {
                continue;
            }
            let mut magic = [0u8; 4];
            if file.seek(SeekFrom::Start(ogg_offset)).is_ok() && file.read_exact(&mut magic).is_ok()
            {
                decode_stream_slice(&mut magic, ogg_offset);
                if &magic == b"OggS" {
                    available.push(entry);
                }
            }
        }
        available
    }

    /// Reads and unscrambles the full Ogg Vorbis stream for `entry` from `audio/streams/<pack>`.
    pub fn extract_ogg_track(
        &self,
        audio_root: &Path,
        entry: &TrackLookupEntry,
    ) -> Result<Vec<u8>, FormatError> {
        const MAX_TRACK_BYTES: u32 = 32 * 1024 * 1024;
        if entry.size == 0 || entry.size > MAX_TRACK_BYTES {
            return Err(FormatError::Malformed {
                context: format!("stream track {}", entry.track_index),
                detail: format!("invalid track size {}", entry.size),
            });
        }
        let streams_dir = resolve_ci_child(audio_root, "streams").ok_or_else(|| {
            FormatError::EntryNotFound(format!("{}/streams", audio_root.display()))
        })?;
        let pack_path = resolve_ci_child(&streams_dir, &entry.pack_name)
            .ok_or_else(|| FormatError::EntryNotFound(entry.pack_name.clone()))?;

        let mut file = File::open(&pack_path).map_err(|source| FormatError::Io {
            path: pack_path.clone(),
            source,
        })?;
        let ogg_offset = u64::from(entry.offset).saturating_add(SA_STREAM_HEADER_BYTES);
        file.seek(SeekFrom::Start(ogg_offset))
            .map_err(|source| FormatError::Io {
                path: pack_path.clone(),
                source,
            })?;

        let mut buf = vec![0u8; entry.size as usize];
        file.read_exact(&mut buf)
            .map_err(|source| FormatError::Io {
                path: pack_path,
                source,
            })?;
        decode_stream_slice(&mut buf, ogg_offset);

        if buf.get(..4) != Some(b"OggS") {
            let actual = String::from_utf8_lossy(buf.get(..4).unwrap_or(&[])).into_owned();
            return Err(FormatError::InvalidMagic {
                expected: "OggS".into(),
                actual,
            });
        }
        Ok(buf)
    }
}

/// Entry parsed from `PakFiles.dat` + `BankLkup.dat`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SfxBankEntry {
    pub global_bank_index: usize,
    pub local_bank_index: usize,
    pub pack_index: u8,
    pub pack_name: String,
    pub offset: u32,
    pub size: u32,
}

/// Uncompressed 16-bit mono PCM clip extracted from an `audio/SFX/*` archive.
#[derive(Debug, Clone)]
pub struct SfxPcmClip {
    pub sample_rate: u32,
    pub samples: Vec<i16>,
    pub loop_offset: Option<u32>,
}

/// Index of all GTA:SA SFX banks inside `audio/SFX/*`.
#[derive(Debug, Clone, Default)]
pub struct SfxBankIndex {
    pub pack_names: Vec<String>,
    pub banks: Vec<SfxBankEntry>,
}

impl SfxBankIndex {
    /// Parses `PakFiles.dat` (52-byte strings) and `BankLkup.dat` (12-byte records).
    pub fn parse_bytes(pak_files: &[u8], bank_lkup: &[u8]) -> Result<Self, FormatError> {
        let (pak_chunks, _) = pak_files.as_chunks::<52>();
        let pack_names: Vec<String> = pak_chunks.iter().map(|c| read_ascii_z(c)).collect();
        if pack_names.is_empty() {
            return Err(FormatError::Malformed {
                context: "PakFiles.dat".into(),
                detail: "no 52-byte SFX pack entries found".into(),
            });
        }

        let mut local_counts = vec![0usize; pack_names.len()];
        let (bank_chunks, _) = bank_lkup.as_chunks::<12>();
        let mut banks = Vec::with_capacity(bank_chunks.len());
        for (global_bank_index, rec) in bank_chunks.iter().enumerate() {
            let pack_index = rec[0];
            let offset = u32::from_le_bytes([rec[4], rec[5], rec[6], rec[7]]);
            let size = u32::from_le_bytes([rec[8], rec[9], rec[10], rec[11]]);
            let pack_idx_usize = usize::from(pack_index);
            let pack_name = pack_names.get(pack_idx_usize).cloned().unwrap_or_default();
            let local_bank_index = if let Some(cnt) = local_counts.get_mut(pack_idx_usize) {
                let cur = *cnt;
                *cnt += 1;
                cur
            } else {
                0
            };
            banks.push(SfxBankEntry {
                global_bank_index,
                local_bank_index,
                pack_index,
                pack_name,
                offset,
                size,
            });
        }

        Ok(Self { pack_names, banks })
    }

    /// Loads `PakFiles.dat` + `BankLkup.dat` from `<game_root>/audio`.
    pub fn from_audio_dir(audio_root: &Path) -> Result<Self, FormatError> {
        let cfg_dir = resolve_ci_child(audio_root, "CONFIG").ok_or_else(|| {
            FormatError::EntryNotFound(format!("{}/CONFIG", audio_root.display()))
        })?;
        let pak_path = resolve_ci_child(&cfg_dir, "PakFiles.dat")
            .ok_or_else(|| FormatError::EntryNotFound("PakFiles.dat".into()))?;
        let lkup_path = resolve_ci_child(&cfg_dir, "BankLkup.dat")
            .ok_or_else(|| FormatError::EntryNotFound("BankLkup.dat".into()))?;

        let pak_bytes = read_file_bytes(&pak_path)?;
        let lkup_bytes = read_file_bytes(&lkup_path)?;
        Self::parse_bytes(&pak_bytes, &lkup_bytes)
    }

    /// Reads one uncompressed 16-bit LE signed mono PCM sound from `audio/SFX/<pack_name>`.
    pub fn load_clip(
        &self,
        audio_root: &Path,
        pack_name: &str,
        local_bank_idx: usize,
        sound_idx: usize,
    ) -> Result<SfxPcmClip, FormatError> {
        let bank = self
            .banks
            .iter()
            .find(|b| {
                b.pack_name.eq_ignore_ascii_case(pack_name) && b.local_bank_index == local_bank_idx
            })
            .ok_or_else(|| {
                FormatError::EntryNotFound(format!("SFX {pack_name}[{local_bank_idx}]"))
            })?;

        let sfx_dir = resolve_ci_child(audio_root, "SFX")
            .ok_or_else(|| FormatError::EntryNotFound(format!("{}/SFX", audio_root.display())))?;
        let pack_path = resolve_ci_child(&sfx_dir, &bank.pack_name)
            .ok_or_else(|| FormatError::EntryNotFound(bank.pack_name.clone()))?;

        let mut file = File::open(&pack_path).map_err(|source| FormatError::Io {
            path: pack_path.clone(),
            source,
        })?;
        file.seek(SeekFrom::Start(u64::from(bank.offset)))
            .map_err(|source| FormatError::Io {
                path: pack_path.clone(),
                source,
            })?;

        let mut hdr = vec![0u8; SA_SFX_BANK_HEADER_BYTES as usize];
        file.read_exact(&mut hdr)
            .map_err(|source| FormatError::Io {
                path: pack_path.clone(),
                source,
            })?;

        let num_sounds = u16::from_le_bytes([hdr[0], hdr[1]]) as usize;
        if num_sounds == 0 || num_sounds > 400 || sound_idx >= num_sounds {
            return Err(FormatError::Malformed {
                context: format!("SFX {pack_name}[{local_bank_idx}]"),
                detail: format!("sound_idx {sound_idx} out of range (num_sounds={num_sounds})"),
            });
        }

        let slot_off = 4 + sound_idx * 12;
        let s_off = u32::from_le_bytes([
            hdr[slot_off],
            hdr[slot_off + 1],
            hdr[slot_off + 2],
            hdr[slot_off + 3],
        ]);
        let loop_raw = u32::from_le_bytes([
            hdr[slot_off + 4],
            hdr[slot_off + 5],
            hdr[slot_off + 6],
            hdr[slot_off + 7],
        ]);
        let sample_rate = u32::from(u16::from_le_bytes([hdr[slot_off + 8], hdr[slot_off + 9]]));

        let next_off = if sound_idx + 1 < num_sounds {
            let n_slot = 4 + (sound_idx + 1) * 12;
            u32::from_le_bytes([
                hdr[n_slot],
                hdr[n_slot + 1],
                hdr[n_slot + 2],
                hdr[n_slot + 3],
            ])
        } else {
            bank.size
        };

        const MAX_CLIP_BYTES: u32 = 8 * 1024 * 1024;
        if next_off <= s_off || next_off > bank.size || (next_off - s_off) > MAX_CLIP_BYTES {
            return Err(FormatError::Malformed {
                context: format!("SFX {pack_name}[{local_bank_idx}][{sound_idx}]"),
                detail: format!(
                    "invalid PCM span {s_off}..{next_off} (bank_size={})",
                    bank.size
                ),
            });
        }

        let pcm_byte_len = (next_off - s_off) as usize;
        let pcm_pos = u64::from(bank.offset) + SA_SFX_BANK_HEADER_BYTES + u64::from(s_off);
        file.seek(SeekFrom::Start(pcm_pos))
            .map_err(|source| FormatError::Io {
                path: pack_path.clone(),
                source,
            })?;

        let mut raw_pcm = vec![0u8; pcm_byte_len];
        file.read_exact(&mut raw_pcm)
            .map_err(|source| FormatError::Io {
                path: pack_path,
                source,
            })?;

        let (sample_chunks, _) = raw_pcm.as_chunks::<2>();
        let samples: Vec<i16> = sample_chunks
            .iter()
            .map(|c| i16::from_le_bytes(*c))
            .collect();

        Ok(SfxPcmClip {
            sample_rate: sample_rate.max(8000),
            samples,
            loop_offset: (loop_raw != u32::MAX && loop_raw > 0).then_some(loop_raw),
        })
    }
}

fn read_ascii_z(bytes: &[u8]) -> String {
    let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..len]).trim().to_string()
}

fn read_file_bytes(path: &Path) -> Result<Vec<u8>, FormatError> {
    fs::read(path).map_err(|source| FormatError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn resolve_ci_child(dir: &Path, name: &str) -> Option<PathBuf> {
    let direct = dir.join(name);
    if direct.exists() {
        return Some(direct);
    }
    let entries = fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        if let Some(fname) = entry.file_name().to_str()
            && fname.eq_ignore_ascii_case(name)
        {
            return Some(entry.path());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn stream_xor_roundtrip_and_index_parse() {
        let mut strm_paks = vec![0u8; 32];
        strm_paks[..2].copy_from_slice(b"NJ");
        strm_paks[16..21].copy_from_slice(b"BEATS");

        let mut trak_lkup = vec![0u8; 12];
        trak_lkup[0] = 1;
        trak_lkup[4..8].copy_from_slice(&0u32.to_le_bytes());
        trak_lkup[8..12].copy_from_slice(&64u32.to_le_bytes());

        let index =
            AudioStreamIndex::parse_bytes(&strm_paks, &trak_lkup).expect("parse stream idx");
        assert_eq!(index.tracks.len(), 1);
        assert_eq!(index.tracks[0].pack_name, "BEATS");

        let mut raw = vec![0u8; SA_STREAM_HEADER_BYTES as usize + 64];
        raw[SA_STREAM_HEADER_BYTES as usize..SA_STREAM_HEADER_BYTES as usize + 4]
            .copy_from_slice(b"OggS");
        decode_stream_slice(&mut raw, 0);
        assert_ne!(
            &raw[SA_STREAM_HEADER_BYTES as usize..SA_STREAM_HEADER_BYTES as usize + 4],
            b"OggS"
        );
        decode_stream_slice(
            &mut raw[SA_STREAM_HEADER_BYTES as usize..],
            SA_STREAM_HEADER_BYTES,
        );
        assert_eq!(
            &raw[SA_STREAM_HEADER_BYTES as usize..SA_STREAM_HEADER_BYTES as usize + 4],
            b"OggS"
        );
    }

    #[test]
    fn sfx_bank_loads_pcm_clip_from_temp_archive() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let tmp_root = std::env::temp_dir().join(format!("gtasarust_sfx_test_{nanos}"));
        fs::create_dir_all(tmp_root.join("CONFIG")).unwrap();
        fs::create_dir_all(tmp_root.join("SFX")).unwrap();

        let mut pak_files = vec![0u8; 52];
        pak_files[..4].copy_from_slice(b"FEET");
        fs::write(tmp_root.join("CONFIG").join("PakFiles.dat"), &pak_files).unwrap();

        let mut bank_lkup = vec![0u8; 12];
        bank_lkup[0] = 0;
        bank_lkup[4..8].copy_from_slice(&0u32.to_le_bytes());
        bank_lkup[8..12].copy_from_slice(&8u32.to_le_bytes());
        fs::write(tmp_root.join("CONFIG").join("BankLkup.dat"), &bank_lkup).unwrap();

        let mut archive = vec![0u8; SA_SFX_BANK_HEADER_BYTES as usize + 8];
        archive[0..2].copy_from_slice(&1u16.to_le_bytes());
        // Slot 0: off=0, loop=u32::MAX, sr=22050, headroom=0
        archive[4..8].copy_from_slice(&0u32.to_le_bytes());
        archive[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
        archive[12..14].copy_from_slice(&22050u16.to_le_bytes());
        let samples = [1000i16, -1000, 2000, -2000];
        for (i, s) in samples.iter().enumerate() {
            let off = SA_SFX_BANK_HEADER_BYTES as usize + i * 2;
            archive[off..off + 2].copy_from_slice(&s.to_le_bytes());
        }
        fs::write(tmp_root.join("SFX").join("FEET"), &archive).unwrap();

        let idx = SfxBankIndex::from_audio_dir(&tmp_root).expect("load sfx idx");
        let clip = idx.load_clip(&tmp_root, "FEET", 0, 0).expect("load clip");
        assert_eq!(clip.sample_rate, 22050);
        assert_eq!(clip.samples, samples);

        let _ = fs::remove_dir_all(&tmp_root);
    }
}
