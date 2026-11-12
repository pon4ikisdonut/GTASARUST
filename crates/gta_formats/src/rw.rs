use crate::error::FormatError;
use crate::reader::BinReader;
use serde::{Deserialize, Serialize};

pub const RW_STRUCT: u32 = 0x0001;
pub const RW_STRING: u32 = 0x0002;
pub const RW_EXTENSION: u32 = 0x0003;
pub const RW_TEXTURE: u32 = 0x0006;
pub const RW_MATERIAL: u32 = 0x0007;
pub const RW_MATERIAL_LIST: u32 = 0x0008;
pub const RW_FRAME_LIST: u32 = 0x000E;
pub const RW_GEOMETRY: u32 = 0x000F;
pub const RW_CLUMP: u32 = 0x0010;
pub const RW_ATOMIC: u32 = 0x0014;
pub const RW_TEXTURE_NATIVE: u32 = 0x0015;
pub const RW_TEX_DICTIONARY: u32 = 0x0016;
pub const RW_GEOMETRY_LIST: u32 = 0x001A;
pub const RW_SKIN_PLG: u32 = 0x0116;
pub const RW_HANIM_PLG: u32 = 0x011E;
pub const RW_BIN_MESH_PLG: u32 = 0x050E;
/// Rockstar SA CustomBuildingDNPipeline extra pre-lit night vertex color plugin chunk.
pub const RW_EXTRA_VERT_COLOUR_PLG: u32 = 0x0253_F2F9;

#[derive(Debug, Clone, Copy)]
pub struct RwChunkHeader {
    pub chunk_type: u32,
    pub size: usize,
    pub lib_id: u32,
}

impl RwChunkHeader {
    pub fn read(r: &mut BinReader<'_>) -> Result<Self, FormatError> {
        let chunk_type = r.read_u32_le()?;
        let size = r.read_u32_le()? as usize;
        let lib_id = r.read_u32_le()?;
        Ok(Self {
            chunk_type,
            size,
            lib_id,
        })
    }

    pub fn version(&self) -> u32 {
        let id = self.lib_id;
        if (id & 0xFFFF_0000) == 0 {
            id << 8
        } else {
            (((id >> 14) & 0x3FF00) + 0x30000) | ((id >> 16) & 0x3F)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DffHAnimNode {
    pub node_id: i32,
    pub node_index: i32,
    pub flags: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DffFrame {
    pub rotation: [[f32; 3]; 3],
    pub position: [f32; 3],
    pub parent: i32,
    pub flags: u32,
    pub name: String,
    #[serde(default)]
    pub hanim_node_id: Option<i32>,
    #[serde(default)]
    pub hanim_nodes: Vec<DffHAnimNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DffSkin {
    pub num_bones: u8,
    pub bone_indices: Vec<[u8; 4]>,
    pub bone_weights: Vec<[f32; 4]>,
    pub inv_bind_matrices: Vec<[f32; 16]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DffMaterial {
    pub color: [u8; 4],
    pub texture_name: Option<String>,
    pub alpha_name: Option<String>,
    pub ambient: f32,
    pub specular: f32,
    pub diffuse: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DffMeshSplit {
    pub material_index: u32,
    pub indices: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DffGeometry {
    pub flags: u32,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub colors: Vec<[u8; 4]>,
    #[serde(default)]
    pub night_colors: Vec<[u8; 4]>,
    pub triangles: Vec<[u16; 4]>, // [v1, v2, v3, material_id]
    pub materials: Vec<DffMaterial>,
    pub splits: Vec<DffMeshSplit>,
    pub is_tristrip: bool,
    pub bounding_sphere: [f32; 4],
    #[serde(default)]
    pub skin: Option<DffSkin>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DffAtomic {
    pub frame_index: u32,
    pub geometry_index: u32,
    pub flags: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DffClump {
    pub rw_version: u32,
    pub frames: Vec<DffFrame>,
    pub geometries: Vec<DffGeometry>,
    pub atomics: Vec<DffAtomic>,
}

impl DffClump {
    /// Parse a RenderWare `.dff` Clump binary buffer.
    pub fn parse(data: &[u8]) -> Result<Self, FormatError> {
        let mut r = BinReader::new(data);
        let root = RwChunkHeader::read(&mut r)?;
        if root.chunk_type != RW_CLUMP {
            return Err(FormatError::InvalidMagic {
                expected: "RW Clump (0x10)".to_string(),
                actual: format!("0x{:04X}", root.chunk_type),
            });
        }
        let end = (r.position() + root.size).min(data.len());
        let mut clump = DffClump {
            rw_version: root.version(),
            ..Default::default()
        };

        while r.position() + 12 <= end {
            let hdr = RwChunkHeader::read(&mut r)?;
            let chunk_end = (r.position() + hdr.size).min(end);
            match hdr.chunk_type {
                RW_STRUCT => {
                    // Clump struct: atomic count (1 or 3 u32s)
                    r.seek(chunk_end)?;
                }
                RW_FRAME_LIST => {
                    clump.frames = parse_frame_list(&data[r.position()..chunk_end])?;
                    r.seek(chunk_end)?;
                }
                RW_GEOMETRY_LIST => {
                    clump.geometries = parse_geometry_list(&data[r.position()..chunk_end])?;
                    r.seek(chunk_end)?;
                }
                RW_ATOMIC => {
                    if let Some(atomic) = parse_atomic(&data[r.position()..chunk_end])? {
                        clump.atomics.push(atomic);
                    }
                    r.seek(chunk_end)?;
                }
                _ => {
                    r.seek(chunk_end)?;
                }
            }
        }

        Ok(clump)
    }
}

fn parse_frame_list(data: &[u8]) -> Result<Vec<DffFrame>, FormatError> {
    let mut r = BinReader::new(data);
    let mut frames = Vec::new();
    let mut ext_frame_idx = 0usize;
    while r.remaining() >= 12 {
        let hdr = RwChunkHeader::read(&mut r)?;
        let chunk_end = (r.position() + hdr.size).min(data.len());
        if hdr.chunk_type == RW_STRUCT && r.remaining() >= 4 {
            let count = r.read_u32_le()? as usize;
            frames.reserve(count.min(1024));
            for _ in 0..count {
                if r.remaining() < 56 {
                    break;
                }
                let rot = [r.read_vec3_le()?, r.read_vec3_le()?, r.read_vec3_le()?];
                let position = r.read_vec3_le()?;
                let parent = r.read_i32_le()?;
                let flags = r.read_u32_le()?;
                frames.push(DffFrame {
                    rotation: rot,
                    position,
                    parent,
                    flags,
                    name: String::new(),
                    hanim_node_id: None,
                    hanim_nodes: Vec::new(),
                });
            }
        } else if hdr.chunk_type == RW_EXTENSION {
            let mut er = BinReader::new(&data[r.position()..chunk_end]);
            while er.remaining() >= 12 {
                let eh = RwChunkHeader::read(&mut er)?;
                let e_end = (er.position() + eh.size).min(chunk_end - r.position());
                if eh.chunk_type == 0x0253_F2FE && ext_frame_idx < frames.len() {
                    let s = er.read_fixed_str(eh.size)?;
                    frames[ext_frame_idx].name = s;
                } else if eh.chunk_type == RW_HANIM_PLG
                    && ext_frame_idx < frames.len()
                    && eh.size >= 12
                {
                    let _ver = er.read_u32_le()?;
                    let node_id = er.read_i32_le()?;
                    let num_nodes = er.read_u32_le()? as usize;
                    frames[ext_frame_idx].hanim_node_id = Some(node_id);
                    if num_nodes > 0 && er.remaining() >= 8 {
                        let _flags = er.read_u32_le()?;
                        let _key_frame_size = er.read_u32_le()?;
                        let mut nodes = Vec::with_capacity(num_nodes.min(256));
                        for _ in 0..num_nodes {
                            if er.remaining() < 12 {
                                break;
                            }
                            nodes.push(DffHAnimNode {
                                node_id: er.read_i32_le()?,
                                node_index: er.read_i32_le()?,
                                flags: er.read_u32_le()?,
                            });
                        }
                        frames[ext_frame_idx].hanim_nodes = nodes;
                    }
                }
                er.seek(e_end)?;
            }
            ext_frame_idx += 1;
        }
        r.seek(chunk_end)?;
    }
    Ok(frames)
}

fn parse_geometry_list(data: &[u8]) -> Result<Vec<DffGeometry>, FormatError> {
    let mut r = BinReader::new(data);
    let mut geometries = Vec::new();
    while r.remaining() >= 12 {
        let hdr = RwChunkHeader::read(&mut r)?;
        let chunk_end = (r.position() + hdr.size).min(data.len());
        if hdr.chunk_type == RW_GEOMETRY {
            geometries.push(parse_geometry(
                &data[r.position()..chunk_end],
                hdr.version(),
            )?);
        }
        r.seek(chunk_end)?;
    }
    Ok(geometries)
}

fn parse_geometry(data: &[u8], rw_version: u32) -> Result<DffGeometry, FormatError> {
    let mut r = BinReader::new(data);
    let mut geom = DffGeometry {
        flags: 0,
        positions: Vec::new(),
        normals: Vec::new(),
        uvs: Vec::new(),
        colors: Vec::new(),
        night_colors: Vec::new(),
        triangles: Vec::new(),
        materials: Vec::new(),
        splits: Vec::new(),
        is_tristrip: false,
        bounding_sphere: [0.0; 4],
        skin: None,
    };
    let mut struct_num_verts = 0usize;

    while r.remaining() >= 12 {
        let hdr = RwChunkHeader::read(&mut r)?;
        let chunk_end = (r.position() + hdr.size).min(data.len());
        match hdr.chunk_type {
            RW_STRUCT => {
                let mut sr = BinReader::new(&data[r.position()..chunk_end]);
                let flags = sr.read_u32_le()?;
                let num_tris = sr.read_u32_le()? as usize;
                let num_verts = sr.read_u32_le()? as usize;
                struct_num_verts = num_verts;
                let _num_morphs = sr.read_u32_le()?;
                geom.flags = flags;
                geom.is_tristrip = (flags & 0x01) != 0;

                if rw_version < 0x34000 {
                    sr.skip(12)?; // ambient, specular, diffuse
                }

                let is_native = (flags & 0x0100_0000) != 0;
                if !is_native {
                    if (flags & 0x08) != 0 {
                        geom.colors.reserve(num_verts.min(100_000));
                        for _ in 0..num_verts {
                            if sr.remaining() < 4 {
                                break;
                            }
                            geom.colors.push(sr.read_array::<4>()?);
                        }
                    }

                    let mut uv_layers = ((flags >> 16) & 0xFF) as usize;
                    if uv_layers == 0 {
                        if (flags & 0x80) != 0 {
                            uv_layers = 2;
                        } else if (flags & 0x04) != 0 {
                            uv_layers = 1;
                        }
                    }
                    for layer in 0..uv_layers {
                        for _ in 0..num_verts {
                            if sr.remaining() < 8 {
                                break;
                            }
                            let u = sr.read_f32_le()?;
                            let v = sr.read_f32_le()?;
                            if layer == 0 {
                                geom.uvs.push([u, v]);
                            }
                        }
                    }

                    geom.triangles.reserve(num_tris.min(200_000));
                    for _ in 0..num_tris {
                        if sr.remaining() < 8 {
                            break;
                        }
                        let v2 = sr.read_u16_le()?;
                        let v1 = sr.read_u16_le()?;
                        let mat = sr.read_u16_le()?;
                        let v3 = sr.read_u16_le()?;
                        geom.triangles.push([v1, v2, v3, mat]);
                    }
                }

                if sr.remaining() >= 24 {
                    geom.bounding_sphere = [
                        sr.read_f32_le()?,
                        sr.read_f32_le()?,
                        sr.read_f32_le()?,
                        sr.read_f32_le()?,
                    ];
                    let has_verts = sr.read_u32_le()? != 0;
                    let has_norms = sr.read_u32_le()? != 0;
                    if has_verts {
                        geom.positions.reserve(num_verts.min(100_000));
                        for _ in 0..num_verts {
                            if sr.remaining() < 12 {
                                break;
                            }
                            geom.positions.push(sr.read_vec3_le()?);
                        }
                    }
                    if has_norms {
                        geom.normals.reserve(num_verts.min(100_000));
                        for _ in 0..num_verts {
                            if sr.remaining() < 12 {
                                break;
                            }
                            geom.normals.push(sr.read_vec3_le()?);
                        }
                    }
                }
            }
            RW_MATERIAL_LIST => {
                geom.materials = parse_material_list(&data[r.position()..chunk_end])?;
            }
            RW_EXTENSION => {
                let mut er = BinReader::new(&data[r.position()..chunk_end]);
                while er.remaining() >= 12 {
                    let eh = RwChunkHeader::read(&mut er)?;
                    let e_end = (er.position() + eh.size).min(chunk_end - r.position());
                    if eh.chunk_type == RW_BIN_MESH_PLG && er.remaining() >= 12 {
                        let split_flags = er.read_u32_le()?;
                        let num_splits = er.read_u32_le()? as usize;
                        let _total_indices = er.read_u32_le()?;
                        geom.is_tristrip = (split_flags & 0x01) != 0;
                        for _ in 0..num_splits {
                            if er.remaining() < 8 {
                                break;
                            }
                            let idx_count = er.read_u32_le()? as usize;
                            let mat_idx = er.read_u32_le()?;
                            let mut indices = Vec::with_capacity(idx_count.min(200_000));
                            for _ in 0..idx_count {
                                if er.remaining() < 4 {
                                    break;
                                }
                                indices.push(er.read_u32_le()?);
                            }
                            geom.splits.push(DffMeshSplit {
                                material_index: mat_idx,
                                indices,
                            });
                        }
                    } else if eh.chunk_type == RW_SKIN_PLG && er.remaining() >= 4 {
                        let num_bones = er.read_u8()?;
                        let num_used = er.read_u8()? as usize;
                        let _max_weights = er.read_u8()?;
                        let _pad = er.read_u8()?;
                        if er.remaining() >= num_used {
                            er.skip(num_used)?;
                        }
                        let v_count = if !geom.positions.is_empty() {
                            geom.positions.len()
                        } else {
                            struct_num_verts
                        };
                        let mut bone_indices = Vec::with_capacity(v_count);
                        for _ in 0..v_count {
                            if er.remaining() < 4 {
                                break;
                            }
                            bone_indices.push(er.read_array::<4>()?);
                        }
                        let mut bone_weights = Vec::with_capacity(v_count);
                        for _ in 0..v_count {
                            if er.remaining() < 16 {
                                break;
                            }
                            bone_weights.push([
                                er.read_f32_le()?,
                                er.read_f32_le()?,
                                er.read_f32_le()?,
                                er.read_f32_le()?,
                            ]);
                        }
                        let mut inv_bind_matrices = Vec::with_capacity(usize::from(num_bones));
                        for _ in 0..num_bones {
                            if er.remaining() < 64 {
                                break;
                            }
                            let mut m = [0.0f32; 16];
                            for elem in &mut m {
                                *elem = er.read_f32_le()?;
                            }
                            // Normalize RenderWare matrix padding column elements to valid affine column-major mat4
                            m[3] = 0.0;
                            m[7] = 0.0;
                            m[11] = 0.0;
                            m[15] = 1.0;
                            inv_bind_matrices.push(m);
                        }
                        if bone_indices.len() == v_count
                            && bone_weights.len() == v_count
                            && !inv_bind_matrices.is_empty()
                        {
                            geom.skin = Some(DffSkin {
                                num_bones,
                                bone_indices,
                                bone_weights,
                                inv_bind_matrices,
                            });
                        }
                    } else if eh.chunk_type == RW_EXTRA_VERT_COLOUR_PLG && er.remaining() >= 4 {
                        let magic = er.read_u32_le()?;
                        if magic != 0 {
                            let v_count = if !geom.positions.is_empty() {
                                geom.positions.len()
                            } else {
                                struct_num_verts
                            };
                            geom.night_colors.reserve(v_count.min(100_000));
                            for _ in 0..v_count {
                                if er.remaining() < 4 {
                                    break;
                                }
                                geom.night_colors.push(er.read_array::<4>()?);
                            }
                        }
                    }
                    er.seek(e_end)?;
                }
            }
            _ => {}
        }
        r.seek(chunk_end)?;
    }

    Ok(geom)
}

fn parse_material_list(data: &[u8]) -> Result<Vec<DffMaterial>, FormatError> {
    let mut r = BinReader::new(data);
    let mut mats = Vec::new();
    while r.remaining() >= 12 {
        let hdr = RwChunkHeader::read(&mut r)?;
        let chunk_end = (r.position() + hdr.size).min(data.len());
        if hdr.chunk_type == RW_MATERIAL {
            mats.push(parse_material(&data[r.position()..chunk_end])?);
        }
        r.seek(chunk_end)?;
    }
    Ok(mats)
}

fn parse_material(data: &[u8]) -> Result<DffMaterial, FormatError> {
    let mut r = BinReader::new(data);
    let mut mat = DffMaterial {
        color: [255, 255, 255, 255],
        texture_name: None,
        alpha_name: None,
        ambient: 1.0,
        specular: 0.0,
        diffuse: 1.0,
    };

    while r.remaining() >= 12 {
        let hdr = RwChunkHeader::read(&mut r)?;
        let chunk_end = (r.position() + hdr.size).min(data.len());
        match hdr.chunk_type {
            RW_STRUCT if hdr.size >= 28 => {
                let mut sr = BinReader::new(&data[r.position()..chunk_end]);
                let _flags = sr.read_u32_le()?;
                mat.color = sr.read_array::<4>()?;
                let _unused = sr.read_u32_le()?;
                let _is_textured = sr.read_u32_le()?;
                mat.ambient = sr.read_f32_le()?;
                mat.specular = sr.read_f32_le()?;
                mat.diffuse = sr.read_f32_le()?;
            }
            RW_TEXTURE => {
                let (tex, alpha) = parse_texture_ref(&data[r.position()..chunk_end])?;
                mat.texture_name = tex;
                mat.alpha_name = alpha;
            }
            _ => {}
        }
        r.seek(chunk_end)?;
    }

    Ok(mat)
}

fn parse_texture_ref(data: &[u8]) -> Result<(Option<String>, Option<String>), FormatError> {
    let mut r = BinReader::new(data);
    let mut strings = Vec::new();
    while r.remaining() >= 12 {
        let hdr = RwChunkHeader::read(&mut r)?;
        let chunk_end = (r.position() + hdr.size).min(data.len());
        if hdr.chunk_type == RW_STRING {
            let s = r.read_fixed_str(hdr.size)?;
            strings.push(if s.is_empty() { None } else { Some(s) });
        }
        r.seek(chunk_end)?;
    }
    Ok((
        strings.first().cloned().flatten(),
        strings.get(1).cloned().flatten(),
    ))
}

fn parse_atomic(data: &[u8]) -> Result<Option<DffAtomic>, FormatError> {
    let mut r = BinReader::new(data);
    while r.remaining() >= 12 {
        let hdr = RwChunkHeader::read(&mut r)?;
        let chunk_end = (r.position() + hdr.size).min(data.len());
        if hdr.chunk_type == RW_STRUCT && hdr.size >= 16 {
            let mut sr = BinReader::new(&data[r.position()..chunk_end]);
            let frame_index = sr.read_u32_le()?;
            let geometry_index = sr.read_u32_le()?;
            let flags = sr.read_u32_le()?;
            return Ok(Some(DffAtomic {
                frame_index,
                geometry_index,
                flags,
            }));
        }
        r.seek(chunk_end)?;
    }
    Ok(None)
}

/// Decoded RGBA8 texture extracted from a RenderWare `.txd` archive.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxdTexture {
    pub name: String,
    pub alpha_name: String,
    pub width: u32,
    pub height: u32,
    pub rgba8: Vec<u8>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TxdArchive {
    pub textures: Vec<TxdTexture>,
}

impl TxdArchive {
    /// Parse a RenderWare `.txd` TextureDictionary and decode textures to RGBA8.
    pub fn parse(data: &[u8]) -> Result<Self, FormatError> {
        let mut r = BinReader::new(data);
        let root = RwChunkHeader::read(&mut r)?;
        if root.chunk_type != RW_TEX_DICTIONARY {
            return Err(FormatError::InvalidMagic {
                expected: "RW TexDictionary (0x16)".to_string(),
                actual: format!("0x{:04X}", root.chunk_type),
            });
        }
        let end = (r.position() + root.size).min(data.len());
        let mut textures = Vec::new();

        while r.position() + 12 <= end {
            let hdr = RwChunkHeader::read(&mut r)?;
            let chunk_end = (r.position() + hdr.size).min(end);
            if hdr.chunk_type == RW_TEXTURE_NATIVE
                && let Ok(Some(tex)) = parse_texture_native(&data[r.position()..chunk_end])
            {
                textures.push(tex);
            }
            r.seek(chunk_end)?;
        }

        Ok(Self { textures })
    }

    pub fn find(&self, name: &str) -> Option<&TxdTexture> {
        self.textures
            .iter()
            .find(|t| t.name.eq_ignore_ascii_case(name))
    }
}

fn parse_texture_native(data: &[u8]) -> Result<Option<TxdTexture>, FormatError> {
    let mut r = BinReader::new(data);
    while r.remaining() >= 12 {
        let hdr = RwChunkHeader::read(&mut r)?;
        let chunk_end = (r.position() + hdr.size).min(data.len());
        if hdr.chunk_type == RW_STRUCT {
            let mut sr = BinReader::new(&data[r.position()..chunk_end]);
            if sr.remaining() < 88 {
                return Ok(None);
            }
            let platform_id = sr.read_u32_le()?;
            if platform_id != 8 && platform_id != 9 {
                return Ok(None);
            }
            let _filter_flags = sr.read_u32_le()?;
            let name = sr.read_fixed_str(32)?;
            let alpha_name = sr.read_fixed_str(32)?;
            let raster_format = sr.read_u32_le()?;
            let d3d_or_alpha = sr.read_u32_le()?;
            let width = u32::from(sr.read_u16_le()?);
            let height = u32::from(sr.read_u16_le()?);
            let depth = sr.read_u8()?;
            let _num_levels = sr.read_u8()?;
            let _raster_type = sr.read_u8()?;
            let compression = sr.read_u8()?;

            if width == 0 || height == 0 || width > 8192 || height > 8192 {
                return Ok(None);
            }

            let is_pal8 = (raster_format & 0x2000) != 0;
            let is_pal4 = (raster_format & 0x4000) != 0;
            let palette = if is_pal8 {
                Some(sr.read_bytes(256 * 4)?)
            } else if is_pal4 {
                Some(sr.read_bytes(16 * 4)?)
            } else {
                None
            };

            if sr.remaining() < 4 {
                return Ok(None);
            }
            let mip0_size = sr.read_u32_le()? as usize;
            if sr.remaining() < mip0_size {
                return Ok(None);
            }
            let raw = sr.read_bytes(mip0_size)?;

            let rgba8 = decode_txd_raster(
                platform_id,
                raster_format,
                d3d_or_alpha,
                compression,
                depth,
                width,
                height,
                palette,
                raw,
            );
            return Ok(Some(TxdTexture {
                name,
                alpha_name,
                width,
                height,
                rgba8,
            }));
        }
        r.seek(chunk_end)?;
    }
    Ok(None)
}

#[allow(clippy::too_many_arguments)]
fn decode_txd_raster(
    platform_id: u32,
    raster_format: u32,
    d3d_or_alpha: u32,
    compression: u8,
    depth: u8,
    width: u32,
    height: u32,
    palette: Option<&[u8]>,
    raw: &[u8],
) -> Vec<u8> {
    let pixel_count = (width as usize) * (height as usize);

    // Check FourCC compression (`DXT1`, `DXT3`, `DXT5`) for D3D9 or compression byte for D3D8
    let fourcc = d3d_or_alpha.to_le_bytes();
    let dxt_kind = if platform_id == 9 {
        match &fourcc {
            b"DXT1" => Some(1),
            b"DXT3" => Some(3),
            b"DXT5" => Some(5),
            _ => None,
        }
    } else {
        match compression {
            1 => Some(1),
            3 => Some(3),
            5 => Some(5),
            _ => None,
        }
    };

    let allow_dxt1_alpha = (raster_format & 0x0F00) != 0x0200;
    if let Some(dxt) = dxt_kind {
        return decode_dxt(dxt, width, height, raw, allow_dxt1_alpha);
    }

    if let Some(pal) = palette {
        let mut out = vec![255u8; pixel_count * 4];
        if (raster_format & 0x2000) != 0 {
            for (i, &idx) in raw.iter().take(pixel_count).enumerate() {
                let p = (idx as usize) * 4;
                if p + 3 < pal.len() {
                    out[i * 4..i * 4 + 4].copy_from_slice(&pal[p..p + 4]);
                }
            }
        } else {
            for i in 0..pixel_count {
                let byte = raw.get(i / 2).copied().unwrap_or(0);
                let idx = if i % 2 == 0 { byte & 0x0F } else { byte >> 4 };
                let p = (idx as usize) * 4;
                if p + 3 < pal.len() {
                    out[i * 4..i * 4 + 4].copy_from_slice(&pal[p..p + 4]);
                }
            }
        }
        return out;
    }

    let mut out = vec![255u8; pixel_count * 4];
    let base_fmt = raster_format & 0x0F00;
    if depth == 32 && raw.len() >= pixel_count * 4 {
        let no_alpha = base_fmt == 0x0600 || d3d_or_alpha == 22; // 888 / X8R8G8B8
        for i in 0..pixel_count {
            let b = raw[i * 4];
            let g = raw[i * 4 + 1];
            let r = raw[i * 4 + 2];
            let a = if no_alpha { 255 } else { raw[i * 4 + 3] };
            out[i * 4..i * 4 + 4].copy_from_slice(&[r, g, b, a]);
        }
    } else if depth == 16 && raw.len() >= pixel_count * 2 {
        for i in 0..pixel_count {
            let v = u16::from_le_bytes([raw[i * 2], raw[i * 2 + 1]]);
            let (r, g, b, a) = match base_fmt {
                0x0100 => {
                    // 1555
                    let a = if (v & 0x8000) != 0 { 255 } else { 0 };
                    let r = (((v >> 10) & 0x1F) * 255 / 31) as u8;
                    let g = (((v >> 5) & 0x1F) * 255 / 31) as u8;
                    let b = ((v & 0x1F) * 255 / 31) as u8;
                    (r, g, b, a)
                }
                0x0300 => {
                    // 4444
                    let a = (((v >> 12) & 0x0F) * 17) as u8;
                    let r = (((v >> 8) & 0x0F) * 17) as u8;
                    let g = (((v >> 4) & 0x0F) * 17) as u8;
                    let b = ((v & 0x0F) * 17) as u8;
                    (r, g, b, a)
                }
                _ => {
                    // 565
                    let r = (((v >> 11) & 0x1F) * 255 / 31) as u8;
                    let g = (((v >> 5) & 0x3F) * 255 / 63) as u8;
                    let b = ((v & 0x1F) * 255 / 31) as u8;
                    (r, g, b, 255)
                }
            };
            out[i * 4..i * 4 + 4].copy_from_slice(&[r, g, b, a]);
        }
    } else if depth == 24 && raw.len() >= pixel_count * 3 {
        for i in 0..pixel_count {
            out[i * 4] = raw[i * 3 + 2];
            out[i * 4 + 1] = raw[i * 3 + 1];
            out[i * 4 + 2] = raw[i * 3];
            out[i * 4 + 3] = 255;
        }
    } else if depth == 8 && raw.len() >= pixel_count {
        for (i, &lum) in raw.iter().take(pixel_count).enumerate() {
            out[i * 4..i * 4 + 4].copy_from_slice(&[lum, lum, lum, 255]);
        }
    }

    out
}

/// Composite an external grayscale/alpha TXD mask texture onto a decoded diffuse RGBA8 image.
pub fn merge_rgba_alpha_mask(
    diffuse_rgba: &[u8],
    dw: u32,
    dh: u32,
    mask_rgba: &[u8],
    mw: u32,
    mh: u32,
) -> Vec<u8> {
    let mut out = diffuse_rgba.to_vec();
    if dw == 0 || dh == 0 || mw == 0 || mh == 0 || mask_rgba.len() < (mw * mh * 4) as usize {
        return out;
    }
    let dw_u = dw as usize;
    let dh_u = dh as usize;
    let mw_u = mw as usize;
    let mh_u = mh as usize;
    if out.len() < dw_u * dh_u * 4 {
        return out;
    }
    for y in 0..dh_u {
        let sy = (y * mh_u) / dh_u;
        for x in 0..dw_u {
            let sx = (x * mw_u) / dw_u;
            let m_idx = (sy * mw_u + sx) * 4;
            let lum = mask_rgba[m_idx]; // grayscale masks use R=G=B
            out[(y * dw_u + x) * 4 + 3] = lum;
        }
    }
    out
}

fn decode_dxt(kind: u8, width: u32, height: u32, raw: &[u8], allow_dxt1_alpha: bool) -> Vec<u8> {
    let w = width as usize;
    let h = height as usize;
    let mut out = vec![255u8; w * h * 4];
    let bx_count = w.div_ceil(4);
    let by_count = h.div_ceil(4);
    let block_stride = if kind == 1 { 8 } else { 16 };

    for by in 0..by_count {
        for bx in 0..bx_count {
            let block_idx = (by * bx_count + bx) * block_stride;
            if block_idx + block_stride > raw.len() {
                break;
            }
            let block = &raw[block_idx..block_idx + block_stride];
            let color_off = if kind == 1 { 0 } else { 8 };
            let c0 = u16::from_le_bytes([block[color_off], block[color_off + 1]]);
            let c1 = u16::from_le_bytes([block[color_off + 2], block[color_off + 3]]);
            let lut = u32::from_le_bytes([
                block[color_off + 4],
                block[color_off + 5],
                block[color_off + 6],
                block[color_off + 7],
            ]);
            let colors = dxt_color_table(c0, c1, kind == 1 && allow_dxt1_alpha);

            for py in 0..4 {
                for px in 0..4 {
                    let x = bx * 4 + px;
                    let y = by * 4 + py;
                    if x >= w || y >= h {
                        continue;
                    }
                    let bit_idx = (py * 4 + px) * 2;
                    let code = ((lut >> bit_idx) & 0x03) as usize;
                    let mut rgba = colors[code];

                    if kind == 3 {
                        let a_byte = block[py * 2 + (px / 2)];
                        let nibble = if px % 2 == 0 {
                            a_byte & 0x0F
                        } else {
                            a_byte >> 4
                        };
                        rgba[3] = nibble * 17;
                    } else if kind == 5 {
                        rgba[3] = dxt5_alpha(&block[0..8], py * 4 + px);
                    }

                    let dst = (y * w + x) * 4;
                    out[dst..dst + 4].copy_from_slice(&rgba);
                }
            }
        }
    }

    out
}

fn rgb565_to_rgba(c: u16) -> [u8; 4] {
    let r = (((c >> 11) & 0x1F) * 255 / 31) as u8;
    let g = (((c >> 5) & 0x3F) * 255 / 63) as u8;
    let b = ((c & 0x1F) * 255 / 31) as u8;
    [r, g, b, 255]
}

fn dxt_color_table(c0: u16, c1: u16, allow_punchthrough: bool) -> [[u8; 4]; 4] {
    let p0 = rgb565_to_rgba(c0);
    let p1 = rgb565_to_rgba(c1);
    if c0 > c1 || !allow_punchthrough {
        let p2 = [
            ((2 * u16::from(p0[0]) + u16::from(p1[0])) / 3) as u8,
            ((2 * u16::from(p0[1]) + u16::from(p1[1])) / 3) as u8,
            ((2 * u16::from(p0[2]) + u16::from(p1[2])) / 3) as u8,
            255,
        ];
        let p3 = [
            ((u16::from(p0[0]) + 2 * u16::from(p1[0])) / 3) as u8,
            ((u16::from(p0[1]) + 2 * u16::from(p1[1])) / 3) as u8,
            ((u16::from(p0[2]) + 2 * u16::from(p1[2])) / 3) as u8,
            255,
        ];
        [p0, p1, p2, p3]
    } else {
        let p2 = [
            ((u16::from(p0[0]) + u16::from(p1[0])) / 2) as u8,
            ((u16::from(p0[1]) + u16::from(p1[1])) / 2) as u8,
            ((u16::from(p0[2]) + u16::from(p1[2])) / 2) as u8,
            255,
        ];
        [p0, p1, p2, [0, 0, 0, 0]]
    }
}

fn dxt5_alpha(alpha_block: &[u8], texel_idx: usize) -> u8 {
    let a0 = alpha_block[0];
    let a1 = alpha_block[1];
    let mut bits: u64 = 0;
    for i in 0..6 {
        bits |= u64::from(alpha_block[2 + i]) << (i * 8);
    }
    let code = ((bits >> (texel_idx * 3)) & 0x07) as u16;
    if code == 0 {
        return a0;
    }
    if code == 1 {
        return a1;
    }
    let f0 = u16::from(a0);
    let f1 = u16::from(a1);
    if a0 > a1 {
        (((8 - code) * f0 + (code - 1) * f1) / 7) as u8
    } else if code < 6 {
        (((6 - code) * f0 + (code - 1) * f1) / 5) as u8
    } else if code == 6 {
        0
    } else {
        255
    }
}
