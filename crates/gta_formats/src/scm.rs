use crate::error::FormatError;
use crate::reader::BinReader;
use serde::{Deserialize, Serialize};

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

/// Inspect and decode the header tables of a compiled GTA:SA `main.scm` bytecode image.
pub fn inspect_scm_header(data: &[u8]) -> Result<ScmSummary, FormatError> {
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

    // Locate the mission offset table by scanning the `0002` / `01` segment pointers after globals
    // In SA `main.scm`, globals end at `main_jump_target`, followed by model table + mission table.
    let mut mission_offsets = Vec::new();
    let mut largest_mission_size = 0u32;

    if global_var_bytes + 8 <= data.len() {
        r.seek(global_var_bytes)?;
        let maybe_seg = r.read_u8()?;
        if maybe_seg == 0x73 || maybe_seg == 0x02 {
            // Best-effort read of subsequent segment header if present
            let _ = r.read_u8();
        }
    }

    // Also probe standard SA offset `main_jump_target` -> skip segments 1..3 by reading segment sizes
    // Segment 1: objects table, Segment 2: mission offsets
    if let Ok((m_offsets, max_sz)) = scan_sa_scm_segments(data, global_var_bytes) {
        mission_offsets = m_offsets;
        largest_mission_size = max_sz;
    }

    Ok(ScmSummary {
        header: ScmHeader {
            total_bytes: data.len(),
            main_jump_target,
            largest_mission_size,
            mission_count: mission_offsets.len(),
            global_var_bytes,
        },
        mission_offsets,
    })
}

fn scan_sa_scm_segments(data: &[u8], start: usize) -> Result<(Vec<u32>, u32), FormatError> {
    let mut r = BinReader::new(data);
    r.seek(start)?;
    // Skip padding / initial segment byte (`0x73` = 's' or jump)
    if r.remaining() < 12 {
        return Ok((Vec::new(), 0));
    }
    // Walk until first `0x02 0x00 0x01 <target>` jump to segment 2
    let limit = (start + 64).min(data.len().saturating_sub(7));
    let mut seg2_start = None;
    for pos in start..limit {
        if data[pos] == 0x02 && data[pos + 1] == 0x00 && data[pos + 2] == 0x01 {
            let target =
                u32::from_le_bytes([data[pos + 3], data[pos + 4], data[pos + 5], data[pos + 6]])
                    as usize;
            if target > pos && target < data.len() {
                seg2_start = Some(target);
                break;
            }
        }
    }

    let Some(s2) = seg2_start else {
        return Ok((Vec::new(), 0));
    };
    r.seek(s2)?;
    if r.remaining() < 20 {
        return Ok((Vec::new(), 0));
    }
    // Segment 2 starts with jump to segment 3, padding, then `main_size`, `largest_mission_size`, `mission_count`, ...
    let _op = r.read_u16_le()?;
    let _t = r.read_u8()?;
    let seg3 = r.read_u32_le()? as usize;
    if r.remaining() < 12 {
        return Ok((Vec::new(), 0));
    }
    let _padding = r.read_u8()?;
    let _main_size = r.read_u32_le()?;
    let largest_mission = r.read_u32_le()?;
    let mission_count = (r.read_u16_le()? as usize).min(512);
    let _exclusive_count = r.read_u16_le()?;

    let mut offsets = Vec::with_capacity(mission_count);
    for _ in 0..mission_count {
        if r.position() + 4 > seg3.min(data.len()) {
            break;
        }
        offsets.push(r.read_u32_le()?);
    }

    Ok((offsets, largest_mission))
}
