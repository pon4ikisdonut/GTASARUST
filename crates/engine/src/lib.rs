//! Core engine foundation for GTASARUST: platform checks, configuration,
//! logging, case-insensitive cross-platform VFS, and fixed-timestep clock.

#[cfg(not(target_arch = "x86_64"))]
compile_error!("GTASARUST only supports x86_64 / AMD64 targets.");

pub mod config;
pub mod logging;
pub mod platform;
pub mod time;
pub mod vfs;

pub use config::{
    AudioConfig, ControlsConfig, EngineConfig, GameSettings, GraphicsConfig, QualityPreset,
    WindowMode,
};
pub use logging::init_logging;
pub use platform::{HostInfo, HostOs};
pub use time::{FixedTimestep, GameClock};
pub use vfs::{GameDataRoot, VfsError};
