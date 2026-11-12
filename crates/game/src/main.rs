mod simulation;

use anyhow::Result;
use clap::Parser;
use egui_winit::State as EguiWinitState;
use gtasarust_engine::{EngineConfig, HostInfo, init_logging};
use gtasarust_renderer::{RenderInstanceBatch, SceneRenderer};
use gtasarust_ui::{ScreenState, build_game_ui};
use simulation::GameWorldState;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tracing::{error, info, warn};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

#[derive(Debug, Parser)]
#[command(
    name = "gtasarust",
    about = "GTASARUST — Cross-platform Rust recreation of Grand Theft Auto: San Andreas"
)]
struct CliArgs {
    /// Path to user's authorized GTA: San Andreas installation directory.
    #[arg(long, env = "GTASA_DATA_DIR")]
    game_dir: Option<PathBuf>,

    /// Path to TOML configuration file.
    #[arg(long)]
    config: Option<PathBuf>,

    /// Run without creating a window or GPU surface (for CI / automated testing).
    #[arg(long)]
    headless: bool,

    /// Initialize subsystems, run deterministic simulation ticks, verify state, and exit.
    #[arg(long)]
    smoke_test: bool,

    /// Number of simulation frames to execute in headless mode.
    #[arg(long, default_value_t = 60)]
    frames: u32,

    /// Enable verbose debug logging.
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = CliArgs::parse();
    init_logging(args.verbose);

    let host = HostInfo::detect();
    info!(
        os = %host.os_name,
        arch = %host.arch,
        cores = host.cpu_cores,
        "Starting GTASARUST"
    );

    let config_path = args
        .config
        .unwrap_or_else(EngineConfig::default_config_path);
    let mut config = EngineConfig::load_or_default(&config_path)?;
    if let Some(dir) = args.game_dir {
        config.game_data_dir = Some(dir);
    }

    let mut game = GameWorldState::initialize(config, host, args.headless || args.smoke_test);

    if args.headless || args.smoke_test {
        info!(
            frames = args.frames,
            "Running headless simulation smoke check"
        );
        game.run_headless_smoke(args.frames)?;
        info!("Headless simulation completed cleanly");
        return Ok(());
    }

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = WindowedApp {
        game,
        window: None,
        renderer: None,
        egui_ctx: egui::Context::default(),
        egui_winit: None,
        last_frame: Instant::now(),
        fps_smooth: 60.0,
    };
    event_loop.run_app(&mut app)?;
    Ok(())
}

struct WindowedApp {
    game: GameWorldState,
    window: Option<Arc<Window>>,
    renderer: Option<SceneRenderer>,
    egui_ctx: egui::Context,
    egui_winit: Option<EguiWinitState>,
    last_frame: Instant,
    fps_smooth: f32,
}

impl ApplicationHandler for WindowedApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title("GTASARUST — Grand Theft Auto: San Andreas")
            .with_inner_size(LogicalSize::new(
                self.game.config.graphics.width,
                self.game.config.graphics.height,
            ));

        match event_loop.create_window(attrs) {
            Ok(win) => {
                let win = Arc::new(win);
                match SceneRenderer::new(win.clone(), &self.game.config.graphics) {
                    Ok(mut renderer) => {
                        self.game.preload_render_meshes(&mut renderer);
                        let egui_winit = EguiWinitState::new(
                            self.egui_ctx.clone(),
                            egui::ViewportId::ROOT,
                            win.as_ref(),
                            Some(win.scale_factor() as f32),
                            None,
                            None,
                        );
                        self.renderer = Some(renderer);
                        self.egui_winit = Some(egui_winit);
                        self.window = Some(win);
                        self.last_frame = Instant::now();
                    }
                    Err(err) => {
                        error!("Failed to initialize wgpu renderer: {err}");
                        event_loop.exit();
                    }
                }
            }
            Err(err) => {
                error!("Failed to create application window: {err}");
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let mut consumed_by_ui = false;
        if let (Some(win), Some(egui_winit)) = (&self.window, &mut self.egui_winit) {
            let resp = egui_winit.on_window_event(win.as_ref(), &event);
            consumed_by_ui = resp.consumed;
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if let Some(r) = &mut self.renderer {
                    r.resize(new_size.width, new_size.height);
                    self.game.camera.aspect = r.aspect();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if !consumed_by_ui || self.game.ui_state.screen == ScreenState::Playing {
                    self.game
                        .input
                        .handle_key_event(event.physical_key, event.state);
                }
            }
            WindowEvent::MouseInput { button, state, .. } => {
                if !consumed_by_ui && self.game.ui_state.screen == ScreenState::Playing {
                    self.game.input.handle_mouse_button(button, state);
                }
            }
            WindowEvent::RedrawRequested => {
                self.tick_and_render(event_loop);
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if let DeviceEvent::MouseMotion { delta } = event
            && self.game.ui_state.screen == ScreenState::Playing
            && !self.game.ui_state.show_dev_overlay
        {
            self.game
                .input
                .add_mouse_delta(delta.0 as f32, delta.1 as f32);
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(win) = &self.window {
            win.request_redraw();
        }
    }
}

impl WindowedApp {
    fn tick_and_render(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        let dt = (now - self.last_frame).as_secs_f32().clamp(0.0001, 0.1);
        self.last_frame = now;
        let inst_fps = 1.0 / dt;
        self.fps_smooth = self.fps_smooth * 0.92 + inst_fps * 0.08;

        self.game.step_frame(dt);

        let Some(win) = self.window.as_ref() else {
            return;
        };
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };
        let Some(egui_winit) = self.egui_winit.as_mut() else {
            return;
        };

        self.game.camera.aspect = renderer.aspect();
        self.game.camera.fov_radians = self.game.config.graphics.fov_degrees.to_radians();
        self.game.camera.far = self.game.config.graphics.draw_distance;

        let raw_input = egui_winit.take_egui_input(win.as_ref());
        let backend_name = renderer.backend_name.clone();
        let adapter_name = renderer.adapter_name.clone();
        let radio_name = self.game.audio.current_station().display_name.clone();

        let mut ui_actions = Default::default();
        let full_output = self.egui_ctx.run_pass(raw_input, |ctx| {
            ui_actions = build_game_ui(
                ctx,
                &mut self.game.ui_state,
                &mut self.game.config,
                &self.game.clock,
                &self.game.player,
                &self.game.vehicles,
                &self.game.world,
                &self.game.missions,
                &self.game.localization,
                self.fps_smooth,
                &backend_name,
                &adapter_name,
                &radio_name,
            );
        });

        egui_winit.handle_platform_output(win.as_ref(), full_output.platform_output.clone());
        let pixels_per_point = win.scale_factor() as f32;
        let primitives = self
            .egui_ctx
            .tessellate(full_output.shapes.clone(), pixels_per_point);

        self.game.apply_ui_actions(ui_actions);
        if self.game.quit_requested {
            event_loop.exit();
            return;
        }

        self.game.stream_nearby_render_assets(renderer, 6);
        self.game.update_skinned_meshes(renderer);
        let atmo = self.game.world.atmosphere(
            self.game.clock.hours_f32(),
            self.game.config.graphics.draw_distance,
        );
        let batches: Vec<RenderInstanceBatch> = self.game.collect_render_batches(renderer);

        if let Err(err) = renderer.render_frame(
            &self.game.camera,
            &atmo,
            self.game.clock.hours_f32(),
            &batches,
            Some((full_output, primitives, pixels_per_point)),
            &self.egui_ctx,
        ) {
            warn!("Render frame warning: {err}");
        }
    }
}
