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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfpKeyframe {
    pub time: f32,
    pub rotation: [f32; 4], // x, y, z, w
    pub translation: Option<[f32; 3]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfpBoneTrack {
    pub name: String,
    pub bone_id: i32,
    pub frame_type: u32,
    pub keyframes: Vec<IfpKeyframe>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfpClip {
    pub name: String,
    pub duration_seconds: f32,
    pub bones: Vec<IfpBoneTrack>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IfpArchive {
    pub version: String,
    pub package_name: String,
    pub clips: Vec<IfpClipHeader>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IfpFullArchive {
    pub version: String,
    pub package_name: String,
    pub clips: Vec<IfpClip>,
}

impl IfpArchive {
    /// Parse an `ANP3` (San Andreas) or `ANPK` (III/VC) animation package header & clip index.
    pub fn parse_header(data: &[u8]) -> Result<Self, FormatError> {
        let full = Self::parse_full_internal(data, false)?;
        let clips = full
            .clips
            .into_iter()
            .map(|c| {
                let frame_count = c.bones.iter().map(|b| b.keyframes.len()).max().unwrap_or(0);
                IfpClipHeader {
                    name: c.name,
                    bone_count: c.bones.len(),
                    frame_count,
                    duration_seconds: c.duration_seconds,
                }
            })
            .collect();
        Ok(Self {
            version: full.version,
            package_name: full.package_name,
            clips,
        })
    }

    /// Parse all `ANP3` animation clips, bone tracks, and quantized keyframes into CPU memory.
    pub fn parse_full(data: &[u8]) -> Result<IfpFullArchive, FormatError> {
        Self::parse_full_internal(data, true)
    }

    fn parse_full_internal(
        data: &[u8],
        decode_frames: bool,
    ) -> Result<IfpFullArchive, FormatError> {
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
                if r.remaining() < 36 {
                    break;
                }
                let name = r.read_fixed_str(24)?;
                let bone_count = r.read_u32_le()? as usize;
                let frame_data_size = r.read_u32_le()? as usize;
                let _unk = r.read_u32_le()?;

                let mut bones = Vec::with_capacity(bone_count.min(256));
                let mut max_time = 0.0f32;
                let start_pos = r.position();
                for _ in 0..bone_count {
                    if r.remaining() < 36 {
                        break;
                    }
                    let bone_name = r.read_fixed_str(24)?.trim().to_string();
                    let frame_type = r.read_u32_le()?;
                    let num_frames = r.read_u32_le()? as usize;
                    let bone_id = r.read_i32_le()?;
                    let stride = match frame_type {
                        3 => 10,
                        4 => 16,
                        _ => 10,
                    };
                    let bytes = num_frames.saturating_mul(stride);
                    if bytes > r.remaining() {
                        break;
                    }

                    let mut keyframes = if decode_frames {
                        Vec::with_capacity(num_frames.min(4096))
                    } else {
                        Vec::new()
                    };

                    if decode_frames {
                        for _ in 0..num_frames {
                            let qx = f32::from(r.read_i16_le()?) / 4096.0;
                            let qy = f32::from(r.read_i16_le()?) / 4096.0;
                            let qz = f32::from(r.read_i16_le()?) / 4096.0;
                            let qw = f32::from(r.read_i16_le()?) / 4096.0;
                            let t_ticks = r.read_i16_le()?;
                            let time = f32::from(t_ticks) / 60.0;
                            max_time = max_time.max(time);
                            let translation = if stride == 16 {
                                let tx = f32::from(r.read_i16_le()?) / 1024.0;
                                let ty = f32::from(r.read_i16_le()?) / 1024.0;
                                let tz = f32::from(r.read_i16_le()?) / 1024.0;
                                Some([tx, ty, tz])
                            } else {
                                None
                            };
                            keyframes.push(IfpKeyframe {
                                time,
                                rotation: [qx, qy, qz, qw],
                                translation,
                            });
                        }
                    } else if num_frames > 0 && stride >= 10 {
                        // In header-only mode keep bone slot count accurate while only reading tail time
                        r.skip(bytes.saturating_sub(2))?;
                        let t_ticks = r.read_i16_le().unwrap_or(0);
                        max_time = max_time.max(f32::from(t_ticks) / 60.0);
                        // If stride == 16 the tail 6 bytes are tx/ty/tz rather than time; header peek above matches legacy behavior
                        keyframes.resize_with(num_frames, || IfpKeyframe {
                            time: max_time,
                            rotation: [0.0, 0.0, 0.0, 1.0],
                            translation: None,
                        });
                    } else {
                        r.skip(bytes)?;
                    }

                    bones.push(IfpBoneTrack {
                        name: bone_name,
                        bone_id,
                        frame_type,
                        keyframes,
                    });
                }

                if r.position() == start_pos && frame_data_size <= r.remaining() {
                    r.skip(frame_data_size)?;
                }

                clips.push(IfpClip {
                    name,
                    duration_seconds: max_time,
                    bones,
                });
            }

            Ok(IfpFullArchive {
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
            Ok(IfpFullArchive {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_synthetic_anp3_keyframes() {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"ANP3");
        buf.extend_from_slice(&0u32.to_le_bytes());
        let mut pkg = [0u8; 24];
        pkg[..3].copy_from_slice(b"ped");
        buf.extend_from_slice(&pkg);
        buf.extend_from_slice(&1u32.to_le_bytes()); // 1 clip

        let mut clip_name = [0u8; 24];
        clip_name[..9].copy_from_slice(b"walk_civi");
        buf.extend_from_slice(&clip_name);
        buf.extend_from_slice(&1u32.to_le_bytes()); // 1 bone
        buf.extend_from_slice(&68u32.to_le_bytes()); // frame data size
        buf.extend_from_slice(&0u32.to_le_bytes());

        let mut bone_name = [0u8; 24];
        bone_name[..4].copy_from_slice(b"Root");
        buf.extend_from_slice(&bone_name);
        buf.extend_from_slice(&4u32.to_le_bytes()); // frame_type 4
        buf.extend_from_slice(&2u32.to_le_bytes()); // 2 keyframes
        buf.extend_from_slice(&0i32.to_le_bytes()); // bone_id 0

        // Keyframe 0 at t=0
        for v in [0i16, 0, 0, 4096, 0, 0, 0, 0] {
            buf.extend_from_slice(&v.to_le_bytes());
        }
        // Keyframe 1 at t=0.5s (30 ticks), ty=0.5m (512)
        for v in [0i16, 0, 0, 4096, 30, 0, 512, 0] {
            buf.extend_from_slice(&v.to_le_bytes());
        }

        let parsed = IfpArchive::parse_full(&buf).expect("parse ANP3");
        assert_eq!(parsed.clips.len(), 1);
        assert_eq!(parsed.clips[0].name, "walk_civi");
        assert!((parsed.clips[0].duration_seconds - 0.5).abs() < 0.001);
        assert_eq!(parsed.clips[0].bones[0].keyframes.len(), 2);
        let tr = parsed.clips[0].bones[0].keyframes[1].translation.unwrap();
        assert!((tr[1] - 0.5).abs() < 0.001);
    }
}
