use crate::error::FormatError;
use crate::reader::BinReader;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfpClipHeader {
    pub name: String,
    pub bone_count: usize,
    pub frame_count: usize,
    pub duration_seconds: f32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IfpArchive {
    pub version: String,
    pub package_name: String,
    pub clips: Vec<IfpClipHeader>,
}

impl IfpArchive {
    /// Parse an `ANP3` (San Andreas) or `ANPK` (III/VC) animation package header & clip index.
    pub fn parse_header(data: &[u8]) -> Result<Self, FormatError> {
        let mut r = BinReader::new(data);
        if data.len() < 8 {
            return Err(FormatError::UnexpectedEof {
                offset: 0,
                needed: 8,
                remaining: data.len(),
            });
        }
        let magic = r.read_bytes(4)?;
        let _total_size = r.read_u32_le()?;

        if magic == b"ANP3" {
            let package_name = r.read_fixed_str(24)?;
            let clip_count = r.read_u32_le()? as usize;
            let mut clips = Vec::with_capacity(clip_count.min(4096));

            for _ in 0..clip_count {
                if r.remaining() < 32 {
                    break;
                }
                let name = r.read_fixed_str(24)?;
                let bone_count = r.read_u32_le()? as usize;
                let frame_data_size = r.read_u32_le()? as usize;
                let _unk = r.read_u32_le()?;
                clips.push(IfpClipHeader {
                    name,
                    bone_count,
                    frame_count: 0,
                    duration_seconds: 0.0,
                });

                // Skip over bone headers + frame data safely
                let mut total_frames = 0usize;
                let mut max_time = 0.0f32;
                let start_pos = r.position();
                for _ in 0..bone_count {
                    if r.remaining() < 36 {
                        break;
                    }
                    let _bone_name = r.read_fixed_str(24)?;
                    let frame_type = r.read_u32_le()?;
                    let num_frames = r.read_u32_le()? as usize;
                    let _bone_id = r.read_i32_le()?;
                    total_frames = total_frames.max(num_frames);
                    let stride = match frame_type {
                        3 => 10, // compressed quat + time
                        4 => 16, // compressed quat + translation + time
                        _ => 10,
                    };
                    let bytes = num_frames.saturating_mul(stride);
                    if bytes > r.remaining() {
                        break;
                    }
                    if num_frames > 0 && stride >= 10 {
                        // Read last keyframe's timestamp (i16 / 60.0) before skipping
                        r.skip(bytes.saturating_sub(2))?;
                        let t_ticks = r.read_i16_le().unwrap_or(0);
                        max_time = max_time.max((f32::from(t_ticks)) / 60.0);
                    } else {
                        r.skip(bytes)?;
                    }
                }
                if let Some(last) = clips.last_mut() {
                    last.frame_count = total_frames;
                    last.duration_seconds = max_time;
                }
                if r.position() == start_pos && frame_data_size <= r.remaining() {
                    r.skip(frame_data_size)?;
                }
            }

            Ok(Self {
                version: "ANP3".to_string(),
                package_name,
                clips,
            })
        } else if magic == b"ANPK" {
            let info_tag = r.read_bytes(4)?;
            let package_name = if info_tag == b"INFO" {
                let sz = r.read_u32_le()? as usize;
                let _cnt = r.read_u32_le()?;
                r.read_fixed_str(sz.saturating_sub(4))?
            } else {
                String::new()
            };
            Ok(Self {
                version: "ANPK".to_string(),
                package_name,
                clips: Vec::new(),
            })
        } else {
            Err(FormatError::InvalidMagic {
                expected: "ANP3 or ANPK".to_string(),
                actual: String::from_utf8_lossy(magic).into_owned(),
            })
        }
    }
}
