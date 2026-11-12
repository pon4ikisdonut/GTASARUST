use crate::error::FormatError;
use crate::reader::BinReader;
use std::collections::HashMap;

/// Parsed GTA: San Andreas `.gxt` localization archive (`TABL` + `TKEY` + `TDAT`).
#[derive(Debug, Clone, Default)]
pub struct GxtArchive {
    pub tables: HashMap<String, HashMap<u32, String>>,
}

impl GxtArchive {
    /// Parse a GTA:SA PC `.gxt` binary file (version 4, 8-bit strings).
    pub fn parse(data: &[u8]) -> Result<Self, FormatError> {
        let mut r = BinReader::new(data);
        if data.len() < 12 {
            return Err(FormatError::UnexpectedEof {
                offset: 0,
                needed: 12,
                remaining: data.len(),
            });
        }

        let version = r.read_u16_le()?;
        let char_bits = r.read_u16_le()?;
        if version != 4 || char_bits != 8 {
            return Err(FormatError::Unsupported(format!(
                "expected SA GXT v4/8-bit, got v{version}/{char_bits}-bit"
            )));
        }

        let tabl_tag = r.read_bytes(4)?;
        if tabl_tag != b"TABL" {
            return Err(FormatError::InvalidMagic {
                expected: "TABL".to_string(),
                actual: String::from_utf8_lossy(tabl_tag).into_owned(),
            });
        }
        let tabl_size = r.read_u32_le()? as usize;
        let table_count = tabl_size / 12;

        let mut table_defs = Vec::with_capacity(table_count);
        for _ in 0..table_count {
            let name = r.read_fixed_str(8)?;
            let offset = r.read_u32_le()? as usize;
            table_defs.push((name, offset));
        }

        let mut tables = HashMap::with_capacity(table_count);
        for (name, offset) in table_defs {
            r.seek(offset)?;
            // Non-MAIN tables carry an 8-byte table name header before TKEY
            let tag = r.read_bytes(4)?;
            let tkey_tag = if tag == b"TKEY" {
                tag
            } else {
                r.skip(4)?; // remainder of 8-byte table name
                r.read_bytes(4)?
            };
            if tkey_tag != b"TKEY" {
                return Err(FormatError::InvalidMagic {
                    expected: "TKEY".to_string(),
                    actual: String::from_utf8_lossy(tkey_tag).into_owned(),
                });
            }

            let tkey_size = r.read_u32_le()? as usize;
            let key_count = tkey_size / 8;
            let mut entries = Vec::with_capacity(key_count);
            for _ in 0..key_count {
                let text_offset = r.read_u32_le()? as usize;
                let hash = r.read_u32_le()?;
                entries.push((text_offset, hash));
            }

            let tdat_tag = r.read_bytes(4)?;
            if tdat_tag != b"TDAT" {
                return Err(FormatError::InvalidMagic {
                    expected: "TDAT".to_string(),
                    actual: String::from_utf8_lossy(tdat_tag).into_owned(),
                });
            }
            let tdat_size = r.read_u32_le()? as usize;
            let tdat_bytes = r.read_bytes(tdat_size)?;

            let mut map = HashMap::with_capacity(entries.len());
            for (text_offset, hash) in entries {
                if text_offset < tdat_bytes.len() {
                    let slice = &tdat_bytes[text_offset..];
                    let end = slice.iter().position(|&b| b == 0).unwrap_or(slice.len());
                    let text = decode_win1252(&slice[..end]);
                    map.insert(hash, text);
                }
            }
            tables.insert(name, map);
        }

        Ok(Self { tables })
    }

    /// Look up a GXT key across `MAIN` or a specific mission sub-table using SA's CRC32 hash.
    pub fn get(&self, table: &str, key: &str) -> Option<&str> {
        let hash = gxt_key_hash(key);
        self.tables
            .get(table)
            .and_then(|t| t.get(&hash))
            .or_else(|| self.tables.get("MAIN").and_then(|t| t.get(&hash)))
            .map(String::as_str)
    }

    pub fn total_entries(&self) -> usize {
        self.tables.values().map(HashMap::len).sum()
    }
}

/// San Andreas GXT key hashing uses standard IEEE CRC32 over uppercase ASCII bytes,
/// but GTA:SA actually uses the bitwise-complemented / jamcrc convention (`!crc32`).
pub fn gxt_key_hash(key: &str) -> u32 {
    let mut hasher = crc32fast::Hasher::new();
    for b in key.as_bytes() {
        hasher.update(&[b.to_ascii_uppercase()]);
    }
    !hasher.finalize()
}

fn decode_win1252(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len());
    for &b in bytes {
        if b < 0x80 {
            s.push(b as char);
        } else {
            s.push(char::from(b));
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_synthetic_gxt() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&4u16.to_le_bytes());
        buf.extend_from_slice(&8u16.to_le_bytes());
        buf.extend_from_slice(b"TABL");
        buf.extend_from_slice(&12u32.to_le_bytes());
        buf.extend_from_slice(b"MAIN\0\0\0\0");
        buf.extend_from_slice(&24u32.to_le_bytes()); // offset of MAIN table

        // MAIN starts at 24
        buf.extend_from_slice(b"TKEY");
        buf.extend_from_slice(&8u32.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes()); // text offset 0
        buf.extend_from_slice(&gxt_key_hash("TESTKEY").to_le_bytes());

        let msg = b"Grove Street\0";
        buf.extend_from_slice(b"TDAT");
        buf.extend_from_slice(&(msg.len() as u32).to_le_bytes());
        buf.extend_from_slice(msg);

        let gxt = GxtArchive::parse(&buf).unwrap();
        assert_eq!(gxt.get("MAIN", "TESTKEY"), Some("Grove Street"));
    }
}
