//! `wgpu` + WGSL 3D renderer for GTASARUST (Z-up San Andreas world coordinates),
//! supporting sky/timecycle atmosphere, instanced meshes, frustum culling,
//! RenderWare DFF/TXD GPU upload, and `egui-wgpu` UI overlay composition.

pub mod camera;
pub mod mesh;
pub mod renderer;

pub use camera::{CameraState, Frustum};
pub use mesh::{GpuMeshHandle, InstanceData, Vertex3D};
pub use renderer::{RenderInstanceBatch, SceneRenderer};
