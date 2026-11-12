use bytemuck::{Pod, Zeroable};
use glam::{Mat3, Mat4, Quat, Vec3};
use gtasarust_gta_formats::{
    dat::WaterQuad,
    rw::{DffClump, DffFrame, DffGeometry},
};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GpuMeshHandle(pub usize);

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex3D {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex3D {
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex3D>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![
            0 => Float32x3,
            1 => Float32x3,
            2 => Float32x2,
            3 => Float32x4
        ],
    };
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct InstanceData {
    pub model_c0: [f32; 4],
    pub model_c1: [f32; 4],
    pub model_c2: [f32; 4],
    pub model_c3: [f32; 4],
    pub tint: [f32; 4],
}

impl InstanceData {
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<InstanceData>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &wgpu::vertex_attr_array![
            4 => Float32x4,
            5 => Float32x4,
            6 => Float32x4,
            7 => Float32x4,
            8 => Float32x4
        ],
    };

    pub fn from_transform(pos: Vec3, rot: Quat, scale: Vec3, tint: [f32; 4]) -> Self {
        let m = Mat4::from_scale_rotation_translation(scale, rot, pos);
        Self {
            model_c0: m.x_axis.to_array(),
            model_c1: m.y_axis.to_array(),
            model_c2: m.z_axis.to_array(),
            model_c3: m.w_axis.to_array(),
            tint,
        }
    }
}

/// CPU-side skinning bind-pose arrays and bone influences for a `DffSubMesh`.
#[derive(Debug, Clone)]
pub struct SkinnedVertexWeights {
    pub base_positions: Vec<[f32; 3]>,
    pub base_normals: Vec<[f32; 3]>,
    pub bone_indices: Vec<[u8; 4]>,
    pub bone_weights: Vec<[f32; 4]>,
}

/// CPU-side textured sub-mesh extracted from a RenderWare `.dff` clump.
pub struct DffSubMesh {
    pub texture_name: Option<String>,
    pub material_color: [f32; 4],
    pub cpu_mesh: CpuMeshData,
    pub skin_weights: Option<SkinnedVertexWeights>,
}

impl DffSubMesh {
    /// Evaluate linear-blend CPU skinning for all vertices using `skin_matrices`, writing
    /// deformed `Vertex3D` records into `out`.
    pub fn compute_skinned_vertices(&self, skin_matrices: &[Mat4], out: &mut Vec<Vertex3D>) {
        out.clear();
        let Some(sw) = &self.skin_weights else {
            out.extend_from_slice(&self.cpu_mesh.vertices);
            return;
        };
        if skin_matrices.is_empty() {
            out.extend_from_slice(&self.cpu_mesh.vertices);
            return;
        }
        out.reserve(self.cpu_mesh.vertices.len());
        for (i, v) in self.cpu_mesh.vertices.iter().enumerate() {
            let base_p = Vec3::from_array(sw.base_positions[i]);
            let base_n = Vec3::from_array(sw.base_normals[i]);
            let idxs = sw.bone_indices[i];
            let wgts = sw.bone_weights[i];
            let mut pos_acc = Vec3::ZERO;
            let mut nrm_acc = Vec3::ZERO;
            let mut total_w = 0.0f32;
            for k in 0..4 {
                let w = wgts[k];
                if w <= 1e-5 {
                    continue;
                }
                if let Some(&mat) = skin_matrices.get(usize::from(idxs[k])) {
                    pos_acc += mat.transform_point3(base_p) * w;
                    nrm_acc += (Mat3::from_mat4(mat) * base_n) * w;
                    total_w += w;
                }
            }
            let (position, normal) = if total_w > 1e-4 {
                (
                    (pos_acc / total_w).to_array(),
                    nrm_acc.try_normalize().unwrap_or(base_n).to_array(),
                )
            } else {
                (base_p.to_array(), base_n.to_array())
            };
            out.push(Vertex3D {
                position,
                normal,
                uv: v.uv,
                color: v.color,
            });
        }
    }
}

#[derive(Default)]
struct SubMeshAccum {
    texture_name: Option<String>,
    material_color: [f32; 4],
    vertices: Vec<Vertex3D>,
    indices: Vec<u32>,
    skin_weights: Option<SkinnedVertexWeights>,
}

pub struct CpuMeshData {
    pub vertices: Vec<Vertex3D>,
    pub indices: Vec<u32>,
}

impl CpuMeshData {
    /// Generate a textured unit cube centered at `(0,0,0)` with extent `[-0.5, 0.5]`.
    pub fn unit_box() -> Self {
        let faces: [([f32; 3], [[f32; 3]; 4]); 6] = [
            (
                [0.0, 0.0, 1.0],
                [
                    [-0.5, -0.5, 0.5],
                    [0.5, -0.5, 0.5],
                    [0.5, 0.5, 0.5],
                    [-0.5, 0.5, 0.5],
                ],
            ),
            (
                [0.0, 0.0, -1.0],
                [
                    [-0.5, 0.5, -0.5],
                    [0.5, 0.5, -0.5],
                    [0.5, -0.5, -0.5],
                    [-0.5, -0.5, -0.5],
                ],
            ),
            (
                [1.0, 0.0, 0.0],
                [
                    [0.5, -0.5, -0.5],
                    [0.5, 0.5, -0.5],
                    [0.5, 0.5, 0.5],
                    [0.5, -0.5, 0.5],
                ],
            ),
            (
                [-1.0, 0.0, 0.0],
                [
                    [-0.5, 0.5, -0.5],
                    [-0.5, -0.5, -0.5],
                    [-0.5, -0.5, 0.5],
                    [-0.5, 0.5, 0.5],
                ],
            ),
            (
                [0.0, 1.0, 0.0],
                [
                    [0.5, 0.5, -0.5],
                    [-0.5, 0.5, -0.5],
                    [-0.5, 0.5, 0.5],
                    [0.5, 0.5, 0.5],
                ],
            ),
            (
                [0.0, -1.0, 0.0],
                [
                    [-0.5, -0.5, -0.5],
                    [0.5, -0.5, -0.5],
                    [0.5, -0.5, 0.5],
                    [-0.5, -0.5, 0.5],
                ],
            ),
        ];
        let uvs = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];
        let mut vertices = Vec::with_capacity(24);
        let mut indices = Vec::with_capacity(36);
        for (normal, corners) in faces {
            let base = vertices.len() as u32;
            for i in 0..4 {
                vertices.push(Vertex3D {
                    position: corners[i],
                    normal,
                    uv: uvs[i],
                    color: [1.0, 1.0, 1.0, 1.0],
                });
            }
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
        Self { vertices, indices }
    }

    pub fn ground_quad(half_size: f32, uv_repeat: f32) -> Self {
        let vertices = vec![
            Vertex3D {
                position: [-half_size, -half_size, 0.0],
                normal: [0.0, 0.0, 1.0],
                uv: [0.0, uv_repeat],
                color: [1.0; 4],
            },
            Vertex3D {
                position: [half_size, -half_size, 0.0],
                normal: [0.0, 0.0, 1.0],
                uv: [uv_repeat, uv_repeat],
                color: [1.0; 4],
            },
            Vertex3D {
                position: [half_size, half_size, 0.0],
                normal: [0.0, 0.0, 1.0],
                uv: [uv_repeat, 0.0],
                color: [1.0; 4],
            },
            Vertex3D {
                position: [-half_size, half_size, 0.0],
                normal: [0.0, 0.0, 1.0],
                uv: [0.0, 0.0],
                color: [1.0; 4],
            },
        ];
        Self {
            vertices,
            indices: vec![0, 1, 2, 0, 2, 3],
        }
    }

    /// Build a combined water surface mesh from parsed `water.dat` quads.
    pub fn from_water_quads(quads: &[WaterQuad]) -> Option<Self> {
        if quads.is_empty() {
            return None;
        }
        let mut vertices = Vec::with_capacity(quads.len() * 4);
        let mut indices = Vec::with_capacity(quads.len() * 6);
        for q in quads {
            let base = vertices.len() as u32;
            let corners = q.corners;
            for c in corners {
                vertices.push(Vertex3D {
                    position: c,
                    normal: [0.0, 0.0, 1.0],
                    uv: [c[0] / 24.0, c[1] / 24.0],
                    color: [0.10, 0.42, 0.62, 0.72],
                });
            }
            let p0 = Vec3::from_array(corners[0]);
            let p1 = Vec3::from_array(corners[1]);
            let p3 = Vec3::from_array(corners[3]);
            // Ensure upward-facing (+Z) winding for backface culling
            if (p1 - p0).cross(p3 - p0).z >= 0.0 {
                indices.extend_from_slice(&[base, base + 1, base + 3, base, base + 3, base + 2]);
            } else {
                indices.extend_from_slice(&[base, base + 3, base + 1, base, base + 2, base + 3]);
            }
        }
        Some(Self { vertices, indices })
    }

    /// Build a downward-pointing GTA:SA entrance/mission cone marker (`apex` at `z = 0`).
    pub fn marker_cone(radius: f32, height: f32) -> Self {
        const SEGS: usize = 16;
        let mut vertices = Vec::with_capacity(SEGS + 2);
        let mut indices = Vec::with_capacity(SEGS * 6);

        // Vertex 0: downward tip
        vertices.push(Vertex3D {
            position: [0.0, 0.0, 0.0],
            normal: [0.0, 0.0, -1.0],
            uv: [0.5, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
        });
        // Vertex 1: top center
        vertices.push(Vertex3D {
            position: [0.0, 0.0, height],
            normal: [0.0, 0.0, 1.0],
            uv: [0.5, 0.0],
            color: [1.0, 1.0, 1.0, 1.0],
        });

        let ring_start = 2u32;
        for i in 0..SEGS {
            let angle = (i as f32 / SEGS as f32) * std::f32::consts::TAU;
            let (s, c) = angle.sin_cos();
            let nrm = Vec3::new(c * height, s * height, -radius)
                .try_normalize()
                .unwrap_or(Vec3::X)
                .to_array();
            vertices.push(Vertex3D {
                position: [c * radius, s * radius, height],
                normal: nrm,
                uv: [i as f32 / SEGS as f32, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            });
        }

        for i in 0..SEGS {
            let curr = ring_start + i as u32;
            let next = ring_start + ((i + 1) % SEGS) as u32;
            // Outer cone side (CCW looking from outside)
            indices.extend_from_slice(&[0, next, curr]);
            // Top cap (+Z normal)
            indices.extend_from_slice(&[1, curr, next]);
        }

        Self { vertices, indices }
    }

    /// Compute minimum vertex Z height (useful for aligning multi-part vehicle meshes to ground).
    pub fn min_z(&self) -> f32 {
        self.vertices
            .iter()
            .map(|v| v.position[2])
            .fold(f32::INFINITY, f32::min)
    }

    /// Convert a RenderWare `DffClump` into material-grouped textured sub-meshes with
    /// hierarchical `DffFrame` transforms and corrected triangle-strip winding.
    pub fn dff_clump_submeshes(clump: &DffClump) -> Vec<DffSubMesh> {
        let frame_mats = compute_frame_world_matrices(&clump.frames);
        let mut groups: HashMap<String, SubMeshAccum> = HashMap::new();
        let mut order: Vec<String> = Vec::new();
        let filter_vehicle_variants = clump.atomics.len() > 2;

        if clump.atomics.is_empty() {
            for geom in &clump.geometries {
                append_geometry_submeshes(geom, Mat4::IDENTITY, &mut groups, &mut order);
            }
        } else {
            for atomic in &clump.atomics {
                let geom_idx = atomic.geometry_index as usize;
                let frame_idx = atomic.frame_index as usize;
                let Some(geom) = clump.geometries.get(geom_idx) else {
                    continue;
                };
                if filter_vehicle_variants && let Some(frame) = clump.frames.get(frame_idx) {
                    let lower = frame.name.to_ascii_lowercase();
                    if lower.ends_with("_dam") || lower.ends_with("_vlo") {
                        continue;
                    }
                }
                let xform = frame_mats.get(frame_idx).copied().unwrap_or(Mat4::IDENTITY);
                append_geometry_submeshes(geom, xform, &mut groups, &mut order);
            }
        }

        order
            .into_iter()
            .filter_map(|key| {
                let acc = groups.remove(&key)?;
                if acc.vertices.is_empty() || acc.indices.is_empty() {
                    return None;
                }
                Some(DffSubMesh {
                    texture_name: acc.texture_name,
                    material_color: acc.material_color,
                    cpu_mesh: CpuMeshData {
                        vertices: acc.vertices,
                        indices: acc.indices,
                    },
                    skin_weights: acc.skin_weights,
                })
            })
            .collect()
    }

    /// Convert a parsed RenderWare `DffClump` into a single combined GPU-uploadable mesh.
    pub fn from_dff_clump(clump: &DffClump) -> Option<Self> {
        let subs = Self::dff_clump_submeshes(clump);
        if subs.is_empty() {
            return None;
        }
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        for sub in subs {
            let base = vertices.len() as u32;
            vertices.extend(sub.cpu_mesh.vertices);
            indices.extend(sub.cpu_mesh.indices.into_iter().map(|idx| base + idx));
        }
        Some(Self { vertices, indices })
    }
}

fn compute_frame_world_matrices(frames: &[DffFrame]) -> Vec<Mat4> {
    let locals: Vec<Mat4> = frames
        .iter()
        .map(|f| {
            Mat4::from_cols(
                Vec3::from_array(f.rotation[0]).extend(0.0),
                Vec3::from_array(f.rotation[1]).extend(0.0),
                Vec3::from_array(f.rotation[2]).extend(0.0),
                Vec3::from_array(f.position).extend(1.0),
            )
        })
        .collect();

    let mut world = Vec::with_capacity(frames.len());
    for idx in 0..frames.len() {
        let mut acc = locals[idx];
        let mut cur = frames[idx].parent;
        let mut depth = 0usize;
        while cur >= 0 && (cur as usize) < frames.len() && depth < frames.len() {
            let p_idx = cur as usize;
            acc = locals[p_idx] * acc;
            cur = frames[p_idx].parent;
            depth += 1;
        }
        world.push(acc);
    }
    world
}

fn append_geometry_submeshes(
    geom: &DffGeometry,
    xform: Mat4,
    groups: &mut HashMap<String, SubMeshAccum>,
    order: &mut Vec<String>,
) {
    if geom.positions.is_empty() {
        return;
    }
    let normal_mat = Mat3::from_mat4(xform);

    let emit_split = |mat_idx: usize,
                      tri_indices: Vec<[u32; 3]>,
                      groups: &mut HashMap<String, SubMeshAccum>,
                      order: &mut Vec<String>| {
        if tri_indices.is_empty() {
            return;
        }
        let mat = geom.materials.get(mat_idx);
        let tex_name = mat
            .and_then(|m| m.texture_name.as_ref())
            .map(|s| s.to_ascii_lowercase());
        let mat_color = mat
            .map(|m| {
                [
                    f32::from(m.color[0]) / 255.0,
                    f32::from(m.color[1]) / 255.0,
                    f32::from(m.color[2]) / 255.0,
                    f32::from(m.color[3]) / 255.0,
                ]
            })
            .unwrap_or([1.0, 1.0, 1.0, 1.0]);

        let key = format!(
            "{}|{:02x}{:02x}{:02x}{:02x}",
            tex_name.as_deref().unwrap_or(""),
            (mat_color[0] * 255.0) as u8,
            (mat_color[1] * 255.0) as u8,
            (mat_color[2] * 255.0) as u8,
            (mat_color[3] * 255.0) as u8
        );

        let entry = groups.entry(key.clone()).or_insert_with(|| {
            order.push(key);
            SubMeshAccum {
                texture_name: tex_name,
                material_color: mat_color,
                ..Default::default()
            }
        });

        let mut local_remap: HashMap<u32, u32> = HashMap::new();
        for [a, b, c] in tri_indices {
            for orig_v in [a, b, c] {
                let idx = orig_v as usize;
                if idx >= geom.positions.len() {
                    continue;
                }
                let mapped = *local_remap.entry(orig_v).or_insert_with(|| {
                    let pos = xform
                        .transform_point3(Vec3::from_array(geom.positions[idx]))
                        .to_array();
                    let raw_n = geom.normals.get(idx).copied().unwrap_or([0.0, 0.0, 1.0]);
                    let normal = (normal_mat * Vec3::from_array(raw_n))
                        .try_normalize()
                        .unwrap_or(Vec3::Z)
                        .to_array();
                    let uv = geom.uvs.get(idx).copied().unwrap_or([0.0, 0.0]);
                    let vcol = geom
                        .colors
                        .get(idx)
                        .map(|c| {
                            [
                                f32::from(c[0]) / 255.0,
                                f32::from(c[1]) / 255.0,
                                f32::from(c[2]) / 255.0,
                                f32::from(c[3]) / 255.0,
                            ]
                        })
                        .unwrap_or([1.0, 1.0, 1.0, 1.0]);
                    let new_idx = entry.vertices.len() as u32;
                    entry.vertices.push(Vertex3D {
                        position: pos,
                        normal,
                        uv,
                        color: vcol,
                    });
                    if let Some(skin) = &geom.skin {
                        let sw = entry.skin_weights.get_or_insert_with(|| {
                            let existing = entry.vertices.len().saturating_sub(1);
                            SkinnedVertexWeights {
                                base_positions: entry.vertices[..existing]
                                    .iter()
                                    .map(|ev| ev.position)
                                    .collect(),
                                base_normals: entry.vertices[..existing]
                                    .iter()
                                    .map(|ev| ev.normal)
                                    .collect(),
                                bone_indices: vec![[0; 4]; existing],
                                bone_weights: vec![[1.0, 0.0, 0.0, 0.0]; existing],
                            }
                        });
                        sw.base_positions.push(geom.positions[idx]);
                        sw.base_normals.push(raw_n);
                        sw.bone_indices
                            .push(skin.bone_indices.get(idx).copied().unwrap_or([0; 4]));
                        sw.bone_weights.push(
                            skin.bone_weights
                                .get(idx)
                                .copied()
                                .unwrap_or([1.0, 0.0, 0.0, 0.0]),
                        );
                    }
                    new_idx
                });
                entry.indices.push(mapped);
            }
        }
    };

    if !geom.splits.is_empty() {
        for split in &geom.splits {
            let mut tris = Vec::new();
            if geom.is_tristrip {
                for (step, win) in split.indices.windows(3).enumerate() {
                    let (a, b, c) = if step % 2 == 1 {
                        (win[0], win[2], win[1])
                    } else {
                        (win[0], win[1], win[2])
                    };
                    if a != b && b != c && a != c {
                        tris.push([a, b, c]);
                    }
                }
            } else {
                for &[a, b, c] in split.indices.as_chunks::<3>().0 {
                    if a != b && b != c && a != c {
                        tris.push([a, b, c]);
                    }
                }
            }
            emit_split(split.material_index as usize, tris, groups, order);
        }
    } else {
        let mut per_mat: HashMap<usize, Vec<[u32; 3]>> = HashMap::new();
        for &[v0, v1, v2, mat_id] in &geom.triangles {
            per_mat.entry(usize::from(mat_id)).or_default().push([
                u32::from(v0),
                u32::from(v1),
                u32::from(v2),
            ]);
        }
        for (mat_idx, tris) in per_mat {
            emit_split(mat_idx, tris, groups, order);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gtasarust_gta_formats::rw::{DffAtomic, DffMaterial, DffMeshSplit};

    #[test]
    fn dff_submeshes_apply_parent_transform_and_tristrip_winding() {
        let clump = DffClump {
            rw_version: 0x36003,
            frames: vec![
                DffFrame {
                    rotation: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
                    position: [10.0, 0.0, 0.0],
                    parent: -1,
                    flags: 0,
                    name: "root".into(),
                    hanim_node_id: None,
                    hanim_nodes: Vec::new(),
                },
                DffFrame {
                    rotation: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
                    position: [0.0, 5.0, 0.0],
                    parent: 0,
                    flags: 0,
                    name: "child".into(),
                    hanim_node_id: None,
                    hanim_nodes: Vec::new(),
                },
            ],
            geometries: vec![DffGeometry {
                flags: 0,
                positions: vec![
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0],
                    [1.0, 1.0, 0.0],
                ],
                normals: vec![[0.0, 0.0, 1.0]; 4],
                uvs: vec![[0.0, 0.0]; 4],
                colors: vec![[255; 4]; 4],
                triangles: vec![],
                materials: vec![DffMaterial {
                    color: [200, 100, 50, 255],
                    texture_name: Some("wall_tex".into()),
                    alpha_name: None,
                    ambient: 1.0,
                    specular: 0.0,
                    diffuse: 1.0,
                }],
                splits: vec![DffMeshSplit {
                    material_index: 0,
                    indices: vec![0, 1, 2, 3],
                }],
                is_tristrip: true,
                bounding_sphere: [0.0; 4],
                skin: None,
            }],
            atomics: vec![DffAtomic {
                frame_index: 1,
                geometry_index: 0,
                flags: 0,
            }],
        };

        let subs = CpuMeshData::dff_clump_submeshes(&clump);
        assert_eq!(subs.len(), 1);
        assert_eq!(subs[0].texture_name.as_deref(), Some("wall_tex"));
        // Verify hierarchical translation (10 + 0, 0 + 5, 0)
        assert!((subs[0].cpu_mesh.vertices[0].position[0] - 10.0).abs() < 1e-4);
        assert!((subs[0].cpu_mesh.vertices[0].position[1] - 5.0).abs() < 1e-4);
        // Verify strip produces 2 triangles with consistent +Z normals (odd triangle reversed!)
        assert_eq!(subs[0].cpu_mesh.indices.len(), 6);
        let p = &subs[0].cpu_mesh.vertices;
        let idx = &subs[0].cpu_mesh.indices;
        let n0 = (Vec3::from_array(p[idx[1] as usize].position)
            - Vec3::from_array(p[idx[0] as usize].position))
        .cross(
            Vec3::from_array(p[idx[2] as usize].position)
                - Vec3::from_array(p[idx[0] as usize].position),
        );
        let n1 = (Vec3::from_array(p[idx[4] as usize].position)
            - Vec3::from_array(p[idx[3] as usize].position))
        .cross(
            Vec3::from_array(p[idx[5] as usize].position)
                - Vec3::from_array(p[idx[3] as usize].position),
        );
        assert!(n0.z > 0.0 && n1.z > 0.0);
    }

    #[test]
    fn water_quad_builds_upward_facing_triangles() {
        let quads = vec![WaterQuad {
            corners: [
                [-10.0, -10.0, 0.0],
                [10.0, -10.0, 0.0],
                [-10.0, 10.0, 0.0],
                [10.0, 10.0, 0.0],
            ],
            flags: 1,
        }];
        let mesh = CpuMeshData::from_water_quads(&quads).unwrap();
        assert_eq!(mesh.indices.len(), 6);
        let v = &mesh.vertices;
        let idx = &mesh.indices;
        let n0 = (Vec3::from_array(v[idx[1] as usize].position)
            - Vec3::from_array(v[idx[0] as usize].position))
        .cross(
            Vec3::from_array(v[idx[2] as usize].position)
                - Vec3::from_array(v[idx[0] as usize].position),
        );
        assert!(n0.z > 0.0);
    }

    #[test]
    fn skinned_submesh_deforms_vertices_from_bone_matrix() {
        use gtasarust_gta_formats::rw::{DffAtomic, DffMaterial, DffMeshSplit, DffSkin};

        let clump = DffClump {
            rw_version: 0x36003,
            frames: vec![DffFrame {
                rotation: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
                position: [0.0, 0.0, 0.0],
                parent: -1,
                flags: 0,
                name: "root".into(),
                hanim_node_id: Some(0),
                hanim_nodes: Vec::new(),
            }],
            geometries: vec![DffGeometry {
                flags: 0,
                positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                normals: vec![[0.0, 0.0, 1.0]; 3],
                uvs: vec![[0.0, 0.0]; 3],
                colors: vec![[255; 4]; 3],
                triangles: Vec::new(),
                materials: vec![DffMaterial {
                    color: [255, 255, 255, 255],
                    texture_name: Some("cj_body".into()),
                    alpha_name: None,
                    ambient: 1.0,
                    specular: 0.0,
                    diffuse: 1.0,
                }],
                splits: vec![DffMeshSplit {
                    material_index: 0,
                    indices: vec![0, 1, 2],
                }],
                is_tristrip: false,
                bounding_sphere: [0.0; 4],
                skin: Some(DffSkin {
                    num_bones: 2,
                    bone_indices: vec![[1, 0, 0, 0]; 3],
                    bone_weights: vec![[1.0, 0.0, 0.0, 0.0]; 3],
                    inv_bind_matrices: vec![Mat4::IDENTITY.to_cols_array(); 2],
                }),
            }],
            atomics: vec![DffAtomic {
                frame_index: 0,
                geometry_index: 0,
                flags: 0,
            }],
        };

        let subs = CpuMeshData::dff_clump_submeshes(&clump);
        assert_eq!(subs.len(), 1);
        assert!(subs[0].skin_weights.is_some());

        let mats = vec![
            Mat4::IDENTITY,
            Mat4::from_translation(Vec3::new(0.0, 0.0, 2.5)),
        ];
        let mut out = Vec::new();
        subs[0].compute_skinned_vertices(&mats, &mut out);
        assert_eq!(out.len(), 3);
        assert!((out[0].position[2] - 2.5).abs() < 1e-4);
        assert!((out[1].position[0] - 1.0).abs() < 1e-4);
        assert!((out[1].position[2] - 2.5).abs() < 1e-4);
    }

    #[test]
    fn marker_cone_has_outward_normals_and_valid_indices() {
        let cone = CpuMeshData::marker_cone(0.65, 1.1);
        assert!(!cone.vertices.is_empty());
        assert_eq!(cone.indices.len() % 3, 0);
        for &idx in &cone.indices {
            assert!((idx as usize) < cone.vertices.len());
        }
        assert!((cone.vertices[0].position[2] - 0.0).abs() < 1e-4);
        assert!((cone.vertices[1].position[2] - 1.1).abs() < 1e-4);
    }
}
