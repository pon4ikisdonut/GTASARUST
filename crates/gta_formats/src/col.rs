use crate::error::FormatError;
use crate::reader::BinReader;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColVersion {
    Col1,
    Col2,
    Col3,
    Col4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColBounds {
    pub radius: f32,
    pub center: [f32; 3],
    pub min: [f32; 3],
    pub max: [f32; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColSphere {
    pub center: [f32; 3],
    pub radius: f32,
    pub material: u8,
    pub flags: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColBox {
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub material: u8,
    pub flags: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColFace {
    pub indices: [u16; 3],
    pub material: u8,
    pub light: u8,
}

/// Parsed collision volume for a single GTA:SA model (`COLL`, `COL2`, `COL3`, `COL4`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColModel {
    pub version: ColVersion,
    pub name: String,
    pub model_id: i16,
    pub bounds: ColBounds,
    pub spheres: Vec<ColSphere>,
    pub boxes: Vec<ColBox>,
    pub vertices: Vec<[f32; 3]>,
    pub faces: Vec<ColFace>,
}

/// Parse one or more collision models concatenated in a `.col` buffer or DFF `Collision` section.
pub fn parse_col_archive(data: &[u8]) -> Result<Vec<ColModel>, FormatError> {
    let mut models = Vec::new();
    let mut offset = 0usize;

    while offset + 8 <= data.len() {
        let fourcc = &data[offset..offset + 4];
        let version = match fourcc {
            b"COLL" => ColVersion::Col1,
            b"COL2" => ColVersion::Col2,
            b"COL3" => ColVersion::Col3,
            b"COL4" => ColVersion::Col4,
            _ => {
                if models.is_empty() {
                    return Err(FormatError::InvalidMagic {
                        expected: "COLL/COL2/COL3/COL4".to_string(),
                        actual: String::from_utf8_lossy(fourcc).into_owned(),
                    });
                }
                break;
            }
        };

        let body_size = u32::from_le_bytes([
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]) as usize;
        let total_end = offset
            .checked_add(8)
            .and_then(|v| v.checked_add(body_size))
            .ok_or_else(|| FormatError::Malformed {
                context: "COL size".to_string(),
                detail: "overflow".to_string(),
            })?;
        if total_end > data.len() {
            return Err(FormatError::UnexpectedEof {
                offset,
                needed: body_size + 8,
                remaining: data.len() - offset,
            });
        }

        let model = parse_single_col(version, &data[offset..total_end])?;
        models.push(model);
        offset = total_end;
    }

    Ok(models)
}

fn parse_single_col(version: ColVersion, record: &[u8]) -> Result<ColModel, FormatError> {
    let mut r = BinReader::new(record);
    r.skip(8)?; // FourCC + size
    let name = r.read_fixed_str(22)?;
    let model_id = r.read_i16_le()?;

    match version {
        ColVersion::Col1 => {
            let radius = r.read_f32_le()?;
            let center = r.read_vec3_le()?;
            let min = r.read_vec3_le()?;
            let max = r.read_vec3_le()?;
            let bounds = ColBounds {
                radius,
                center,
                min,
                max,
            };

            let num_spheres = r.read_u32_le()? as usize;
            let mut spheres = Vec::with_capacity(num_spheres.min(4096));
            for _ in 0..num_spheres {
                let sr = r.read_f32_le()?;
                let sc = r.read_vec3_le()?;
                let material = r.read_u8()?;
                let flags = r.read_u8()?;
                r.skip(2)?;
                spheres.push(ColSphere {
                    center: sc,
                    radius: sr,
                    material,
                    flags,
                });
            }

            let _num_unknown = r.read_u32_le()?;
            let num_boxes = r.read_u32_le()? as usize;
            let mut boxes = Vec::with_capacity(num_boxes.min(4096));
            for _ in 0..num_boxes {
                let bmin = r.read_vec3_le()?;
                let bmax = r.read_vec3_le()?;
                let material = r.read_u8()?;
                let flags = r.read_u8()?;
                r.skip(2)?;
                boxes.push(ColBox {
                    min: bmin,
                    max: bmax,
                    material,
                    flags,
                });
            }

            let num_verts = r.read_u32_le()? as usize;
            let mut vertices = Vec::with_capacity(num_verts.min(65536));
            for _ in 0..num_verts {
                vertices.push(r.read_vec3_le()?);
            }

            let num_faces = r.read_u32_le()? as usize;
            let mut faces = Vec::with_capacity(num_faces.min(65536));
            for _ in 0..num_faces {
                let a = r.read_u32_le()? as u16;
                let b = r.read_u32_le()? as u16;
                let c = r.read_u32_le()? as u16;
                let material = r.read_u8()?;
                let light = r.read_u8()?;
                r.skip(2)?;
                faces.push(ColFace {
                    indices: [a, b, c],
                    material,
                    light,
                });
            }

            Ok(ColModel {
                version,
                name,
                model_id,
                bounds,
                spheres,
                boxes,
                vertices,
                faces,
            })
        }
        ColVersion::Col2 | ColVersion::Col3 | ColVersion::Col4 => {
            let min = r.read_vec3_le()?;
            let max = r.read_vec3_le()?;
            let center = r.read_vec3_le()?;
            let radius = r.read_f32_le()?;
            let bounds = ColBounds {
                radius,
                center,
                min,
                max,
            };

            let num_spheres = r.read_u16_le()? as usize;
            let num_boxes = r.read_u16_le()? as usize;
            let num_faces = r.read_u16_le()? as usize;
            let _num_lines = r.read_u8()?;
            let _padding = r.read_u8()?;
            let flags = r.read_u32_le()?;

            let spheres_off = r.read_u32_le()? as usize;
            let boxes_off = r.read_u32_le()? as usize;
            let _lines_off = r.read_u32_le()? as usize;
            let verts_off = r.read_u32_le()? as usize;
            let faces_off = r.read_u32_le()? as usize;
            let _tri_planes_off = r.read_u32_le()? as usize;

            if matches!(version, ColVersion::Col3 | ColVersion::Col4) {
                let _num_shadow_faces = r.read_u32_le()?;
                let _shadow_verts_off = r.read_u32_le()?;
                let _shadow_faces_off = r.read_u32_le()?;
            }
            if matches!(version, ColVersion::Col4) {
                let _unk = r.read_u32_le()?;
            }

            // Offsets in COL2/3/4 are relative to the start of the bounding box + 4? Actually relative to +4 (after FourCC).
            let base = 4usize;

            let mut spheres = Vec::with_capacity(num_spheres);
            if num_spheres > 0 && spheres_off > 0 {
                r.seek(base + spheres_off)?;
                for _ in 0..num_spheres {
                    let sc = r.read_vec3_le()?;
                    let sr = r.read_f32_le()?;
                    let material = r.read_u8()?;
                    let sflags = r.read_u8()?;
                    r.skip(2)?;
                    spheres.push(ColSphere {
                        center: sc,
                        radius: sr,
                        material,
                        flags: sflags,
                    });
                }
            }

            let mut boxes = Vec::with_capacity(num_boxes);
            if num_boxes > 0 && boxes_off > 0 {
                r.seek(base + boxes_off)?;
                for _ in 0..num_boxes {
                    let bmin = r.read_vec3_le()?;
                    let bmax = r.read_vec3_le()?;
                    let material = r.read_u8()?;
                    let bflags = r.read_u8()?;
                    r.skip(2)?;
                    boxes.push(ColBox {
                        min: bmin,
                        max: bmax,
                        material,
                        flags: bflags,
                    });
                }
            }

            let mut faces = Vec::with_capacity(num_faces);
            let mut max_vert_index = 0usize;
            if num_faces > 0 && faces_off > 0 && (flags & 0x02) != 0 {
                r.seek(base + faces_off)?;
                for _ in 0..num_faces {
                    let a = r.read_u16_le()?;
                    let b = r.read_u16_le()?;
                    let c = r.read_u16_le()?;
                    let material = r.read_u8()?;
                    let light = r.read_u8()?;
                    max_vert_index = max_vert_index
                        .max(a as usize)
                        .max(b as usize)
                        .max(c as usize);
                    faces.push(ColFace {
                        indices: [a, b, c],
                        material,
                        light,
                    });
                }
            }

            let mut vertices = Vec::new();
            if !faces.is_empty() && verts_off > 0 {
                let num_verts = max_vert_index + 1;
                r.seek(base + verts_off)?;
                vertices.reserve(num_verts);
                for _ in 0..num_verts {
                    if r.remaining() < 6 {
                        break;
                    }
                    // COL2/3/4 vertices are signed 16-bit fixed-point divided by 128.0
                    let x = f32::from(r.read_i16_le()?) / 128.0;
                    let y = f32::from(r.read_i16_le()?) / 128.0;
                    let z = f32::from(r.read_i16_le()?) / 128.0;
                    vertices.push([x, y, z]);
                }
            }

            Ok(ColModel {
                version,
                name,
                model_id,
                bounds,
                spheres,
                boxes,
                vertices,
                faces,
            })
        }
    }
}
