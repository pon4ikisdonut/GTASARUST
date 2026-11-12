use crate::camera::CameraState;
use crate::mesh::{CpuMeshData, GpuMeshHandle, InstanceData, Vertex3D};
use bytemuck::{Pod, Zeroable};
use egui_wgpu::ScreenDescriptor;
use gtasarust_engine::{GraphicsConfig, WindowMode};
use gtasarust_world::AtmosphereState;
use std::sync::Arc;
use tracing::info;
use wgpu::util::DeviceExt;
use winit::window::{Fullscreen, Window};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct FrameUniforms {
    view_proj: [[f32; 4]; 4],
    camera_pos: [f32; 4],
    sun_dir: [f32; 4],
    sun_color: [f32; 4],
    ambient_color: [f32; 4],
    fog_color: [f32; 4],
    fog_params: [f32; 4],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct SkyUniforms {
    inv_view_proj: [[f32; 4]; 4],
    sky_top: [f32; 4],
    sky_bottom: [f32; 4],
    sun_dir: [f32; 4],
    sun_color: [f32; 4],
}

struct GpuMesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
}

pub struct RenderInstanceBatch {
    pub mesh: GpuMeshHandle,
    pub texture_idx: usize,
    pub instances: Vec<InstanceData>,
}

pub struct SceneRenderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    depth_view: wgpu::TextureView,
    sky_pipeline: wgpu::RenderPipeline,
    world_pipeline: wgpu::RenderPipeline,
    frame_uniform_buf: wgpu::Buffer,
    frame_bind_group: wgpu::BindGroup,
    sky_uniform_buf: wgpu::Buffer,
    sky_bind_group: wgpu::BindGroup,
    tex_bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    textures: Vec<wgpu::BindGroup>,
    meshes: Vec<GpuMesh>,
    pub box_mesh: GpuMeshHandle,
    pub ground_mesh: GpuMeshHandle,
    pub checker_tex: usize,
    pub white_tex: usize,
    pub egui_renderer: egui_wgpu::Renderer,
    pub backend_name: String,
    pub adapter_name: String,
}

impl SceneRenderer {
    pub fn new(window: Arc<Window>, gfx: &GraphicsConfig) -> anyhow::Result<Self> {
        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::from_env().unwrap_or(wgpu::Backends::PRIMARY),
            flags: wgpu::InstanceFlags::from_build_config(),
            backend_options: wgpu::BackendOptions::from_env_or_default(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            display: None,
        });

        let surface = instance.create_surface(window.clone())?;

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;

        let (adapter, device, queue) = rt.block_on(async {
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                    apply_limit_buckets: false,
                })
                .await?;

            let (device, queue) = adapter
                .request_device(&wgpu::DeviceDescriptor {
                    label: Some("gtasarust_device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                    trace: wgpu::Trace::Off,
                    experimental_features: wgpu::ExperimentalFeatures::disabled(),
                })
                .await?;

            Ok::<_, anyhow::Error>((adapter, device, queue))
        })?;

        let info = adapter.get_info();
        let backend_name = format!("{:?}", info.backend);
        let adapter_name = info.name.clone();
        info!(backend = %backend_name, adapter = %adapter_name, "wgpu graphics adapter initialized");

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| !f.is_srgb())
            .unwrap_or(caps.formats[0]);

        let present_mode = if gfx.vsync {
            wgpu::PresentMode::AutoVsync
        } else {
            wgpu::PresentMode::AutoNoVsync
        };

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            color_space: wgpu::SurfaceColorSpace::Auto,
            width,
            height,
            present_mode,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        match gfx.window_mode {
            WindowMode::Windowed => window.set_fullscreen(None),
            WindowMode::BorderlessFullscreen => {
                window.set_fullscreen(Some(Fullscreen::Borderless(None)));
            }
            WindowMode::Fullscreen => {
                window.set_fullscreen(Some(Fullscreen::Borderless(None)));
            }
        }

        let depth_view = create_depth_view(&device, width, height);

        let frame_uniform_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("frame_uniform_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let tex_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("tex_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let frame_uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("frame_uniforms"),
            contents: bytemuck::bytes_of(&FrameUniforms::zeroed()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let frame_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("frame_bg"),
            layout: &frame_uniform_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: frame_uniform_buf.as_entire_binding(),
            }],
        });

        let sky_uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sky_uniforms"),
            contents: bytemuck::bytes_of(&SkyUniforms::zeroed()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let sky_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sky_bg"),
            layout: &frame_uniform_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: sky_uniform_buf.as_entire_binding(),
            }],
        });

        let sky_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sky_shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../../../assets/shaders/sky.wgsl").into(),
            ),
        });
        let world_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("world_shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../../../assets/shaders/world.wgsl").into(),
            ),
        });

        let sky_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sky_pipeline_layout"),
            bind_group_layouts: &[Some(&frame_uniform_layout)],
            immediate_size: 0,
        });
        let sky_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sky_pipeline"),
            layout: Some(&sky_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &sky_shader,
                entry_point: Some("vs_sky"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &sky_shader,
                entry_point: Some("fs_sky"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview_mask: None,
            cache: None,
        });

        let world_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("world_pipeline_layout"),
                bind_group_layouts: &[Some(&frame_uniform_layout), Some(&tex_bind_group_layout)],
                immediate_size: 0,
            });
        let world_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("world_pipeline"),
            layout: Some(&world_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &world_shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(Vertex3D::LAYOUT), Some(InstanceData::LAYOUT)],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Back),
                front_face: wgpu::FrontFace::Ccw,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &world_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview_mask: None,
            cache: None,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("world_sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        let egui_renderer = egui_wgpu::Renderer::new(
            &device,
            format,
            egui_wgpu::RendererOptions {
                msaa_samples: 1,
                depth_stencil_format: None,
                dithering: true,
                predictable_texture_filtering: false,
            },
        );

        let mut renderer = Self {
            surface,
            device,
            queue,
            config,
            depth_view,
            sky_pipeline,
            world_pipeline,
            frame_uniform_buf,
            frame_bind_group,
            sky_uniform_buf,
            sky_bind_group,
            tex_bind_group_layout,
            sampler,
            textures: Vec::new(),
            meshes: Vec::new(),
            box_mesh: GpuMeshHandle(0),
            ground_mesh: GpuMeshHandle(0),
            checker_tex: 0,
            white_tex: 0,
            egui_renderer,
            backend_name,
            adapter_name,
        };

        renderer.white_tex = renderer.upload_rgba8_texture("white", 1, 1, &[255, 255, 255, 255]);
        renderer.checker_tex = renderer.upload_checker_ground_texture();
        renderer.box_mesh = renderer.upload_mesh("unit_box", &CpuMeshData::unit_box());
        renderer.ground_mesh =
            renderer.upload_mesh("ground_quad", &CpuMeshData::ground_quad(2500.0, 250.0));

        Ok(renderer)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.depth_view = create_depth_view(&self.device, width, height);
    }

    pub fn aspect(&self) -> f32 {
        self.config.width as f32 / self.config.height.max(1) as f32
    }

    pub fn surface_size(&self) -> [u32; 2] {
        [self.config.width, self.config.height]
    }

    pub fn upload_mesh(&mut self, label: &str, cpu: &CpuMeshData) -> GpuMeshHandle {
        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: bytemuck::cast_slice(&cpu.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: bytemuck::cast_slice(&cpu.indices),
                usage: wgpu::BufferUsages::INDEX,
            });
        let handle = GpuMeshHandle(self.meshes.len());
        self.meshes.push(GpuMesh {
            vertex_buffer,
            index_buffer,
            index_count: cpu.indices.len() as u32,
        });
        handle
    }

    pub fn upload_rgba8_texture(
        &mut self,
        label: &str,
        width: u32,
        height: u32,
        rgba8: &[u8],
    ) -> usize {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba8,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(label),
            layout: &self.tex_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });
        let idx = self.textures.len();
        self.textures.push(bg);
        idx
    }

    fn upload_checker_ground_texture(&mut self) -> usize {
        let mut pixels = vec![0u8; 64 * 64 * 4];
        for y in 0..64usize {
            for x in 0..64usize {
                let border = x < 2 || y < 2;
                let shade = if border {
                    [78, 82, 74, 255]
                } else if ((x / 16) + (y / 16)) % 2 == 0 {
                    [112, 118, 104, 255]
                } else {
                    [102, 108, 96, 255]
                };
                let dst = (y * 64 + x) * 4;
                pixels[dst..dst + 4].copy_from_slice(&shade);
            }
        }
        self.upload_rgba8_texture("ground_grid", 64, 64, &pixels)
    }

    pub fn render_frame(
        &mut self,
        camera: &CameraState,
        atmo: &AtmosphereState,
        time_hours: f32,
        batches: &[RenderInstanceBatch],
        egui_output: Option<(egui::FullOutput, Vec<egui::ClippedPrimitive>, f32)>,
        _egui_ctx: &egui::Context,
    ) -> anyhow::Result<()> {
        let view_proj = camera.view_proj_matrix();
        let inv_vp = view_proj.inverse();

        let frame_u = FrameUniforms {
            view_proj: view_proj.to_cols_array_2d(),
            camera_pos: [camera.position.x, camera.position.y, camera.position.z, 1.0],
            sun_dir: [atmo.sun_dir.x, atmo.sun_dir.y, atmo.sun_dir.z, 0.0],
            sun_color: atmo.sun_color.to_array(),
            ambient_color: atmo.ambient_color.to_array(),
            fog_color: atmo.fog_color.to_array(),
            fog_params: [
                atmo.fog_start,
                atmo.fog_end,
                time_hours,
                atmo.rain_intensity,
            ],
        };
        self.queue
            .write_buffer(&self.frame_uniform_buf, 0, bytemuck::bytes_of(&frame_u));

        let sky_u = SkyUniforms {
            inv_view_proj: inv_vp.to_cols_array_2d(),
            sky_top: atmo.sky_top.to_array(),
            sky_bottom: atmo.sky_bottom.to_array(),
            sun_dir: [atmo.sun_dir.x, atmo.sun_dir.y, atmo.sun_dir.z, 0.0],
            sun_color: atmo.sun_color.to_array(),
        };
        self.queue
            .write_buffer(&self.sky_uniform_buf, 0, bytemuck::bytes_of(&sky_u));

        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(tex) => tex,
            wgpu::CurrentSurfaceTexture::Suboptimal(tex) => {
                self.surface.configure(&self.device, &self.config);
                tex
            }
            wgpu::CurrentSurfaceTexture::Lost | wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(());
            }
            other => anyhow::bail!("surface acquisition error: {other:?}"),
        };

        let color_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Pre-upload instance buffers before render pass
        let mut gpu_batches = Vec::with_capacity(batches.len());
        for b in batches {
            if b.instances.is_empty() {
                continue;
            }
            let buf = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("instances"),
                    contents: bytemuck::cast_slice(&b.instances),
                    usage: wgpu::BufferUsages::VERTEX,
                });
            gpu_batches.push((b.mesh, b.texture_idx, buf, b.instances.len() as u32));
        }

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("main_encoder"),
            });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("world_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &color_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: f64::from(atmo.sky_bottom.x),
                            g: f64::from(atmo.sky_bottom.y),
                            b: f64::from(atmo.sky_bottom.z),
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            // 1. Sky dome pass
            rpass.set_pipeline(&self.sky_pipeline);
            rpass.set_bind_group(0, &self.sky_bind_group, &[]);
            rpass.draw(0..3, 0..1);

            // 2. Instanced world geometry pass
            rpass.set_pipeline(&self.world_pipeline);
            rpass.set_bind_group(0, &self.frame_bind_group, &[]);
            for (mesh_h, tex_idx, inst_buf, count) in &gpu_batches {
                if let (Some(mesh), Some(tex_bg)) =
                    (self.meshes.get(mesh_h.0), self.textures.get(*tex_idx))
                {
                    rpass.set_bind_group(1, tex_bg, &[]);
                    rpass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                    rpass.set_vertex_buffer(1, inst_buf.slice(..));
                    rpass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    rpass.draw_indexed(0..mesh.index_count, 0, 0..*count);
                }
            }
        }

        // 3. Egui HUD / Menu / Dev Inspector composition pass
        let mut extra_cmds = Vec::new();
        if let Some((full_output, primitives, pixels_per_point)) = egui_output {
            for (id, image_deltas) in &full_output.textures_delta.set {
                for image_delta in image_deltas {
                    self.egui_renderer
                        .update_texture(&self.device, &self.queue, *id, image_delta);
                }
            }
            let screen_desc = ScreenDescriptor {
                size_in_pixels: [self.config.width, self.config.height],
                pixels_per_point,
            };
            extra_cmds = self.egui_renderer.update_buffers(
                &self.device,
                &self.queue,
                &mut encoder,
                &primitives,
                &screen_desc,
            );
            {
                let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("ui_pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &color_view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                let mut static_pass = pass.forget_lifetime();
                self.egui_renderer
                    .render(&mut static_pass, &primitives, &screen_desc);
            }
            for id in &full_output.textures_delta.free {
                self.egui_renderer.free_texture(id);
            }
        }

        extra_cmds.push(encoder.finish());
        self.queue.submit(extra_cmds);
        self.queue.present(frame);

        Ok(())
    }
}

fn create_depth_view(device: &wgpu::Device, width: u32, height: u32) -> wgpu::TextureView {
    let tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth_tex"),
        size: wgpu::Extent3d {
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    tex.create_view(&wgpu::TextureViewDescriptor::default())
}
