use crate::error::FormatError;
use crate::reader::BinReader;
use crate::scm_opcodes::{OpcodeShape, lookup_opcode_meta};
use serde::{Deserialize, Serialize};

/// Parsed summary of a compiled GTA:SA `main.scm` bytecode image header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScmHeader {
    pub total_bytes: usize,
    pub main_jump_target: u32,
    pub largest_mission_size: u32,
    pub mission_count: usize,
    pub global_var_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScmSummary {
    pub header: ScmHeader,
    pub mission_offsets: Vec<u32>,
}

/// Streamed sub-script entry declared in chunk `2` of `main.scm`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScmStreamedScript {
    pub name: String,
    pub offset: u32,
    pub size: u32,
}

/// Complete parsed GTA:SA `main.scm` bytecode container per `SCMChunks.hpp`.
#[derive(Debug, Clone)]
pub struct ScmImage {
    pub data: Vec<u8>,
    pub globals_end: usize,
    pub main_offset: usize,
    pub main_script_size: u32,
    pub largest_mission_size: u32,
    pub mission_offsets: Vec<u32>,
    pub used_objects: Vec<String>,
    pub streamed_scripts: Vec<ScmStreamedScript>,
}

/// Typed GTA:SA array index descriptor (`scm::ArrayAccess` = 6 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScmArrayRef {
    pub base_offset: u16,
    pub index_var: u16,
    pub array_size: u8,
    pub elem_type: u8,
    pub index_is_global: bool,
}

/// Decoded SCM instruction argument covering all `eScriptParameterType` (`1..=19`) variants.
#[derive(Debug, Clone, PartialEq)]
pub enum ScmArg {
    Int(i32),
    Float(f32),
    GlobalVar(u16),
    LocalVar(u16),
    GlobalArray(ScmArrayRef),
    LocalArray(ScmArrayRef),
    StaticShortString(String),
    GlobalShortStringVar(u16),
    LocalShortStringVar(u16),
    GlobalShortStringArray(ScmArrayRef),
    LocalShortStringArray(ScmArrayRef),
    VariableString(String),
    StaticLongString(String),
    GlobalLongStringVar(u16),
    LocalLongStringVar(u16),
    GlobalLongStringArray(ScmArrayRef),
    LocalLongStringArray(ScmArrayRef),
}

impl ScmArg {
    pub fn as_i32_literal(&self) -> Option<i32> {
        match self {
            Self::Int(v) => Some(*v),
            Self::Float(f) => Some(*f as i32),
            _ => None,
        }
    }

    pub fn as_f32_literal(&self) -> Option<f32> {
        match self {
            Self::Float(f) => Some(*f),
            Self::Int(v) => Some(*v as f32),
            _ => None,
        }
    }

    pub fn as_str_literal(&self) -> Option<&str> {
        match self {
            Self::StaticShortString(s) | Self::StaticLongString(s) | Self::VariableString(s) => {
                Some(s.as_str())
            }
            _ => None,
        }
    }
}

/// Decoded GTA:SA SCM bytecode instruction with exact next instruction offset.
#[derive(Debug, Clone)]
pub struct ScmDecodedInstruction {
    pub offset: usize,
    pub next_offset: usize,
    pub opcode: u16,
    pub not_flag: bool,
    pub name: &'static str,
    pub args: Vec<ScmArg>,
}

impl ScmImage {
    /// Parse a full compiled GTA:SA `main.scm` file following `tSCMChunkHeader` chains (`'s'`, `0..=4`).
    pub fn parse(data: Vec<u8>) -> Result<Self, FormatError> {
        if data.len() < 16 {
            return Err(FormatError::UnexpectedEof {
                offset: 0,
                needed: 16,
                remaining: data.len(),
            });
        }

        let mut cur_offset = 0usize;
        let mut globals_end = 0usize;
        let mut main_script_size = 0u32;
        let mut largest_mission_size = 0u32;
        let mut mission_offsets = Vec::new();
        let mut used_objects = Vec::new();
        let mut streamed_scripts = Vec::new();

        for _chunk_pass in 0..8 {
            if cur_offset + 8 > data.len() || data[cur_offset..cur_offset + 3] != [0x02, 0x00, 0x01]
            {
                break;
            }
            let next_offset = u32::from_le_bytes([
                data[cur_offset + 3],
                data[cur_offset + 4],
                data[cur_offset + 5],
                data[cur_offset + 6],
            ]) as usize;
            let chunk_id = data[cur_offset + 7];

            if next_offset <= cur_offset + 7 || next_offset > data.len() {
                return Err(FormatError::Malformed {
                    context: "main.scm".to_string(),
                    detail: format!(
                        "invalid SCM chunk 0x{chunk_id:02X} next offset {next_offset} at {cur_offset}"
                    ),
                });
            }

            let body = &data[cur_offset + 8..next_offset];
            match chunk_id {
                b's' => {
                    globals_end = next_offset;
                }
                0 => {
                    if body.len() >= 4 {
                        let count =
                            u32::from_le_bytes([body[0], body[1], body[2], body[3]]) as usize;
                        let mut pos = 4usize;
                        for _ in 0..count.min(4096) {
                            if pos + 24 > body.len() {
                                break;
                            }
                            used_objects.push(decode_fixed_cstr(&body[pos..pos + 24]));
                            pos += 24;
                        }
                    }
                }
                1 => {
                    if body.len() >= 16 {
                        main_script_size = u32::from_le_bytes([body[0], body[1], body[2], body[3]]);
                        largest_mission_size =
                            u32::from_le_bytes([body[4], body[5], body[6], body[7]]);
                        let n_missions = u16::from_le_bytes([body[8], body[9]]) as usize;
                        let mut pos = 16usize;
                        for _ in 0..n_missions.min(2048) {
                            if pos + 4 > body.len() {
                                break;
                            }
                            mission_offsets.push(u32::from_le_bytes([
                                body[pos],
                                body[pos + 1],
                                body[pos + 2],
                                body[pos + 3],
                            ]));
                            pos += 4;
                        }
                    }
                }
                2 => {
                    if body.len() >= 8 {
                        let count =
                            u32::from_le_bytes([body[4], body[5], body[6], body[7]]) as usize;
                        let mut pos = 8usize;
                        for _ in 0..count.min(1024) {
                            if pos + 28 > body.len() {
                                break;
                            }
                            let name = decode_fixed_cstr(&body[pos..pos + 20]);
                            let offset = u32::from_le_bytes([
                                body[pos + 20],
                                body[pos + 21],
                                body[pos + 22],
                                body[pos + 23],
                            ]);
                            let size = u32::from_le_bytes([
                                body[pos + 24],
                                body[pos + 25],
                                body[pos + 26],
                                body[pos + 27],
                            ]);
                            streamed_scripts.push(ScmStreamedScript { name, offset, size });
                            pos += 28;
                        }
                    }
                }
                3 | 4 => {}
                _ => {
                    cur_offset = next_offset;
                    break;
                }
            }

            cur_offset = next_offset;
            if chunk_id == 4 {
                break;
            }
        }

        if globals_end == 0 {
            return Err(FormatError::Malformed {
                context: "main.scm".to_string(),
                detail: "missing initial 's' global variable chunk".to_string(),
            });
        }

        Ok(Self {
            data,
            globals_end,
            main_offset: cur_offset,
            main_script_size,
            largest_mission_size,
            mission_offsets,
            used_objects,
            streamed_scripts,
        })
    }

    /// Decode a single SCM bytecode instruction at byte `offset` in the image.
    pub fn decode_instruction_at(
        &self,
        offset: usize,
    ) -> Result<ScmDecodedInstruction, FormatError> {
        if offset + 2 > self.data.len() {
            return Err(FormatError::UnexpectedEof {
                offset,
                needed: 2,
                remaining: self.data.len().saturating_sub(offset),
            });
        }
        let op_raw = u16::from_le_bytes([self.data[offset], self.data[offset + 1]]);
        let opcode = op_raw & 0x7FFF;
        let not_flag = (op_raw & 0x8000) != 0;
        let Some(meta) = lookup_opcode_meta(opcode) else {
            return Err(FormatError::Malformed {
                context: "main.scm".to_string(),
                detail: format!("unknown opcode 0x{opcode:04X} at offset {offset}"),
            });
        };

        let mut pos = offset + 2;
        let mut args = Vec::new();
        match meta.shape {
            OpcodeShape::Fixed(count) => {
                for _ in 0..count {
                    let arg = self.read_typed_arg(&mut pos)?;
                    args.push(arg);
                }
            }
            OpcodeShape::VarArgTerminated => {
                let first = self.read_typed_arg(&mut pos)?;
                args.push(first);
                while pos < self.data.len() {
                    if self.data[pos] == 0 {
                        pos += 1;
                        break;
                    }
                    let extra = self.read_typed_arg(&mut pos)?;
                    args.push(extra);
                }
            }
            OpcodeShape::SwitchTable => {
                // First argument is the switch target expression, followed by case count + cases.
                let val_arg = self.read_typed_arg(&mut pos)?;
                args.push(val_arg);
                if pos < self.data.len() {
                    let _ = self.read_typed_arg(&mut pos);
                }
            }
        }

        Ok(ScmDecodedInstruction {
            offset,
            next_offset: pos,
            opcode,
            not_flag,
            name: meta.name,
            args,
        })
    }

    fn read_typed_arg(&self, pos: &mut usize) -> Result<ScmArg, FormatError> {
        if *pos >= self.data.len() {
            return Err(FormatError::UnexpectedEof {
                offset: *pos,
                needed: 1,
                remaining: 0,
            });
        }
        let ty = self.data[*pos];
        *pos += 1;
        let need = |p: usize, n: usize, len: usize| -> Result<(), FormatError> {
            if p + n > len {
                Err(FormatError::UnexpectedEof {
                    offset: p,
                    needed: n,
                    remaining: len.saturating_sub(p),
                })
            } else {
                Ok(())
            }
        };

        let len = self.data.len();
        let arg = match ty {
            1 => {
                need(*pos, 4, len)?;
                let v = i32::from_le_bytes(self.data[*pos..*pos + 4].try_into().unwrap());
                *pos += 4;
                ScmArg::Int(v)
            }
            2 | 3 | 10 | 11 | 16 | 17 => {
                need(*pos, 2, len)?;
                let v = u16::from_le_bytes([self.data[*pos], self.data[*pos + 1]]);
                *pos += 2;
                match ty {
                    2 => ScmArg::GlobalVar(v),
                    3 => ScmArg::LocalVar(v),
                    10 => ScmArg::GlobalShortStringVar(v),
                    11 => ScmArg::LocalShortStringVar(v),
                    16 => ScmArg::GlobalLongStringVar(v),
                    _ => ScmArg::LocalLongStringVar(v),
                }
            }
            4 => {
                need(*pos, 1, len)?;
                let v = i32::from(self.data[*pos] as i8);
                *pos += 1;
                ScmArg::Int(v)
            }
            5 => {
                need(*pos, 2, len)?;
                let v = i32::from(i16::from_le_bytes([self.data[*pos], self.data[*pos + 1]]));
                *pos += 2;
                ScmArg::Int(v)
            }
            6 => {
                need(*pos, 4, len)?;
                let v = f32::from_le_bytes(self.data[*pos..*pos + 4].try_into().unwrap());
                *pos += 4;
                ScmArg::Float(v)
            }
            7 | 8 | 12 | 13 | 18 | 19 => {
                need(*pos, 6, len)?;
                let arr = ScmArrayRef {
                    base_offset: u16::from_le_bytes([self.data[*pos], self.data[*pos + 1]]),
                    index_var: u16::from_le_bytes([self.data[*pos + 2], self.data[*pos + 3]]),
                    array_size: self.data[*pos + 4],
                    elem_type: self.data[*pos + 5] & 0x7F,
                    index_is_global: (self.data[*pos + 5] & 0x80) != 0,
                };
                *pos += 6;
                match ty {
                    7 => ScmArg::GlobalArray(arr),
                    8 => ScmArg::LocalArray(arr),
                    12 => ScmArg::GlobalShortStringArray(arr),
                    13 => ScmArg::LocalShortStringArray(arr),
                    18 => ScmArg::GlobalLongStringArray(arr),
                    _ => ScmArg::LocalLongStringArray(arr),
                }
            }
            9 => {
                need(*pos, 8, len)?;
                let s = decode_fixed_cstr(&self.data[*pos..*pos + 8]);
                *pos += 8;
                ScmArg::StaticShortString(s)
            }
            14 => {
                need(*pos, 1, len)?;
                let slen = usize::from(self.data[*pos]);
                *pos += 1;
                need(*pos, slen, len)?;
                let s = String::from_utf8_lossy(&self.data[*pos..*pos + slen]).into_owned();
                *pos += slen;
                ScmArg::VariableString(s)
            }
            15 => {
                need(*pos, 16, len)?;
                let s = decode_fixed_cstr(&self.data[*pos..*pos + 16]);
                *pos += 16;
                ScmArg::StaticLongString(s)
            }
            other => {
                return Err(FormatError::Malformed {
                    context: "main.scm".to_string(),
                    detail: format!(
                        "invalid SCM argument type 0x{other:02X} at offset {}",
                        *pos - 1
                    ),
                });
            }
        };
        Ok(arg)
    }
}

fn decode_fixed_cstr(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).into_owned()
}

/// Inspect and decode the header tables of a compiled GTA:SA `main.scm` bytecode image.
pub fn inspect_scm_header(data: &[u8]) -> Result<ScmSummary, FormatError> {
    if let Ok(image) = ScmImage::parse(data.to_vec()) {
        return Ok(ScmSummary {
            header: ScmHeader {
                total_bytes: image.data.len(),
                main_jump_target: image.globals_end as u32,
                largest_mission_size: image.largest_mission_size,
                mission_count: image.mission_offsets.len(),
                global_var_bytes: image.globals_end,
            },
            mission_offsets: image.mission_offsets,
        });
    }

    if data.len() < 16 {
        return Err(FormatError::UnexpectedEof {
            offset: 0,
            needed: 16,
            remaining: data.len(),
        });
    }
    let mut r = BinReader::new(data);
    let jump_op = r.read_u16_le()?;
    if jump_op != 0x0002 {
        return Err(FormatError::Malformed {
            context: "main.scm".to_string(),
            detail: format!("expected initial JUMP opcode 0x0002, got 0x{jump_op:04X}"),
        });
    }
    let arg_type = r.read_u8()?;
    let main_jump_target = if arg_type == 0x01 {
        r.read_u32_le()?
    } else {
        return Err(FormatError::Malformed {
            context: "main.scm".to_string(),
            detail: format!("unexpected jump argument type 0x{arg_type:02X}"),
        });
    };

    let global_var_bytes = main_jump_target as usize;
    if global_var_bytes > data.len() {
        return Err(FormatError::Malformed {
            context: "main.scm".to_string(),
            detail: format!(
                "global space offset {global_var_bytes} exceeds file size {}",
                data.len()
            ),
        });
    }

    Ok(ScmSummary {
        header: ScmHeader {
            total_bytes: data.len(),
            main_jump_target,
            largest_mission_size: 0,
            mission_count: 0,
            global_var_bytes,
        },
        mission_offsets: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn push_chunk_header(buf: &mut Vec<u8>, next_off: u32, idx: u8) {
        buf.extend_from_slice(&[0x02, 0x00, 0x01]);
        buf.extend_from_slice(&next_off.to_le_bytes());
        buf.push(idx);
    }

    #[test]
    fn parses_sa_scm_chunks_and_decodes_instructions() {
        let mut scm = Vec::new();
        // Chunk 's' (globals 0..16)
        push_chunk_header(&mut scm, 16, b's');
        scm.extend_from_slice(&[0u8; 8]);
        // Chunk 0 (used objects = 0)
        push_chunk_header(&mut scm, 28, 0);
        scm.extend_from_slice(&0u32.to_le_bytes());
        // Chunk 1 (script info + 1 mission)
        push_chunk_header(&mut scm, 56, 1);
        scm.extend_from_slice(&64u32.to_le_bytes());
        scm.extend_from_slice(&32u32.to_le_bytes());
        scm.extend_from_slice(&1u16.to_le_bytes());
        scm.extend_from_slice(&0u16.to_le_bytes());
        scm.extend_from_slice(&32u32.to_le_bytes());
        scm.extend_from_slice(&100u32.to_le_bytes());
        // Chunk 2..4 minimal
        push_chunk_header(&mut scm, 72, 2);
        scm.extend_from_slice(&0u32.to_le_bytes());
        scm.extend_from_slice(&0u32.to_le_bytes());
        push_chunk_header(&mut scm, 84, 3);
        scm.extend_from_slice(&0u32.to_le_bytes());
        push_chunk_header(&mut scm, 100, 4);
        scm.extend_from_slice(&16u32.to_le_bytes());
        scm.extend_from_slice(&1u32.to_le_bytes());
        assert_eq!(scm.len(), 100);

        // MAIN entry at 100: 0x03A4 SCRIPT_NAME "MAIN", 0x00C0 SET_TIME_OF_DAY 8 0, 0x0001 WAIT 0
        scm.extend_from_slice(&0x03A4u16.to_le_bytes());
        scm.push(9);
        scm.extend_from_slice(b"MAIN\0\0\0\0");
        scm.extend_from_slice(&0x00C0u16.to_le_bytes());
        scm.extend_from_slice(&[4, 8, 4, 0]);
        scm.extend_from_slice(&0x0001u16.to_le_bytes());
        scm.extend_from_slice(&[4, 0]);

        let img = ScmImage::parse(scm).expect("parse synthetic SCM");
        assert_eq!(img.main_offset, 100);
        assert_eq!(img.mission_offsets, vec![100]);

        let i0 = img.decode_instruction_at(img.main_offset).unwrap();
        assert_eq!(i0.opcode, 0x03A4);
        assert_eq!(i0.args[0].as_str_literal(), Some("MAIN"));

        let i1 = img.decode_instruction_at(i0.next_offset).unwrap();
        assert_eq!(i1.opcode, 0x00C0);
        assert_eq!(i1.args[0].as_i32_literal(), Some(8));
        assert_eq!(i1.args[1].as_i32_literal(), Some(0));

        let i2 = img.decode_instruction_at(i1.next_offset).unwrap();
        assert_eq!(i2.opcode, 0x0001);
        assert_eq!(i2.args[0].as_i32_literal(), Some(0));
    }
}
