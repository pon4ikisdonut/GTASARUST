use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Quat, Vec3};
use gtasarust_gta_formats::rw::DffClump;

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

    /// Generate a tiled Z=0 ground quad (`size x size` meters).
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

    /// Convert a parsed RenderWare `DffClump` into a GPU-uploadable mesh.
    pub fn from_dff_clump(clump: &DffClump) -> Option<Self> {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for geom in &clump.geometries {
            let base = vertices.len() as u32;
            for i in 0..geom.positions.len() {
                let normal = geom.normals.get(i).copied().unwrap_or([0.0, 0.0, 1.0]);
                let uv = geom.uvs.get(i).copied().unwrap_or([0.0, 0.0]);
                let col = geom
                    .colors
                    .get(i)
                    .map(|c| {
                        [
                            f32::from(c[0]) / 255.0,
                            f32::from(c[1]) / 255.0,
                            f32::from(c[2]) / 255.0,
                            f32::from(c[3]) / 255.0,
                        ]
                    })
                    .unwrap_or([1.0, 1.0, 1.0, 1.0]);
                vertices.push(Vertex3D {
                    position: geom.positions[i],
                    normal,
                    uv,
                    color: col,
                });
            }
            if !geom.splits.is_empty() {
                for split in &geom.splits {
                    if geom.is_tristrip {
                        for win in split.indices.windows(3) {
                            let (a, b, c) = (win[0], win[1], win[2]);
                            if a != b && b != c && a != c {
                                indices.extend_from_slice(&[base + a, base + b, base + c]);
                            }
                        }
                    } else {
                        for &[a, b, c] in split.indices.as_chunks::<3>().0 {
                            indices.extend_from_slice(&[base + a, base + b, base + c]);
                        }
                    }
                }
            } else {
                for tri in &geom.triangles {
                    indices.extend_from_slice(&[
                        base + u32::from(tri[0]),
                        base + u32::from(tri[1]),
                        base + u32::from(tri[2]),
                    ]);
                }
            }
        }

        if vertices.is_empty() || indices.is_empty() {
            None
        } else {
            Some(Self { vertices, indices })
        }
    }
}
