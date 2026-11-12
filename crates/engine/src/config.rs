use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Supported graphics quality levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum QualityPreset {
    Low,
    #[default]
    Medium,
    High,
    VeryHigh,
}

/// Supported window presentation modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WindowMode {
    #[default]
    Windowed,
    BorderlessFullscreen,
    Fullscreen,
}

/// Configurable rendering and display settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsConfig {
    pub width: u32,
    pub height: u32,
    pub window_mode: WindowMode,
    pub vsync: bool,
    pub msaa_samples: u32,
    pub texture_quality: QualityPreset,
    pub shadow_quality: QualityPreset,
    pub draw_distance: f32,
    pub max_fps: u32,
    pub fov_degrees: f32,
}

impl Default for GraphicsConfig {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            window_mode: WindowMode::Windowed,
            vsync: true,
            msaa_samples: 4,
            texture_quality: QualityPreset::High,
            shadow_quality: QualityPreset::Medium,
            draw_distance: 1200.0,
            max_fps: 120,
            fov_degrees: 70.0,
        }
    }
}

/// Configurable audio mixing settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub voice_volume: f32,
    pub radio_auto_tune: bool,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 0.85,
            music_volume: 0.70,
            sfx_volume: 0.90,
            voice_volume: 0.95,
            radio_auto_tune: true,
        }
    }
}

/// Configurable input sensitivity and controller settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlsConfig {
    pub mouse_sensitivity: f32,
    pub invert_y: bool,
    pub gamepad_deadzone: f32,
}

impl Default for ControlsConfig {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 1.0,
            invert_y: false,
            gamepad_deadzone: 0.15,
        }
    }
}

/// Gameplay and localization preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    pub language: String,
    pub show_hud: bool,
    pub show_radar: bool,
    pub subtitles: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            language: "american".to_string(),
            show_hud: true,
            show_radar: true,
            subtitles: true,
        }
    }
}

/// Root GTASARUST configuration loaded from TOML or defaults.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EngineConfig {
    #[serde(default)]
    pub graphics: GraphicsConfig,
    #[serde(default)]
    pub audio: AudioConfig,
    #[serde(default)]
    pub controls: ControlsConfig,
    #[serde(default)]
    pub game: GameSettings,
    #[serde(default)]
    pub game_data_dir: Option<PathBuf>,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config file '{0}': {1}")]
    Io(PathBuf, #[source] std::io::Error),
    #[error("failed to parse TOML config '{0}': {1}")]
    Parse(PathBuf, #[source] toml::de::Error),
    #[error("failed to serialize TOML config: {0}")]
    Serialize(#[from] toml::ser::Error),
}

impl EngineConfig {
    /// Load configuration from `path`, or return defaults if the file does not exist.
    pub fn load_or_default(path: &Path) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw =
            std::fs::read_to_string(path).map_err(|e| ConfigError::Io(path.to_path_buf(), e))?;
        let mut cfg: Self =
            toml::from_str(&raw).map_err(|e| ConfigError::Parse(path.to_path_buf(), e))?;
        cfg.sanitize();
        Ok(cfg)
    }

    /// Save configuration cleanly to `path`, creating parent directories if needed.
    pub fn save_to_file(&self, path: &Path) -> Result<(), ConfigError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| ConfigError::Io(parent.to_path_buf(), e))?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content).map_err(|e| ConfigError::Io(path.to_path_buf(), e))?;
        Ok(())
    }

    /// Clamp numeric settings into safe, valid ranges.
    pub fn sanitize(&mut self) {
        self.graphics.width = self.graphics.width.max(640);
        self.graphics.height = self.graphics.height.max(480);
        self.graphics.draw_distance = self.graphics.draw_distance.clamp(200.0, 6000.0);
        self.graphics.fov_degrees = self.graphics.fov_degrees.clamp(45.0, 120.0);
        self.graphics.msaa_samples = match self.graphics.msaa_samples {
            1 | 2 | 4 | 8 => self.graphics.msaa_samples,
            _ => 4,
        };
        self.audio.master_volume = self.audio.master_volume.clamp(0.0, 1.0);
        self.audio.music_volume = self.audio.music_volume.clamp(0.0, 1.0);
        self.audio.sfx_volume = self.audio.sfx_volume.clamp(0.0, 1.0);
        self.audio.voice_volume = self.audio.voice_volume.clamp(0.0, 1.0);
        self.controls.mouse_sensitivity = self.controls.mouse_sensitivity.clamp(0.1, 10.0);
        self.controls.gamepad_deadzone = self.controls.gamepad_deadzone.clamp(0.0, 0.5);
    }

    /// Resolve the platform-standard configuration directory (`AppData` on Windows, XDG on Linux).
    pub fn default_config_path() -> PathBuf {
        if let Some(proj) = directories::ProjectDirs::from("org", "gtasarust", "GTASARUST") {
            proj.config_dir().join("config.toml")
        } else {
            PathBuf::from("gtasarust.toml")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_roundtrip_and_sanitize() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cfg.toml");
        let mut cfg = EngineConfig::default();
        cfg.graphics.width = 100;
        cfg.graphics.msaa_samples = 99;
        cfg.audio.master_volume = 2.5;
        cfg.sanitize();

        assert_eq!(cfg.graphics.width, 640);
        assert_eq!(cfg.graphics.msaa_samples, 4);
        assert!((cfg.audio.master_volume - 1.0).abs() < f32::EPSILON);

        cfg.save_to_file(&path).unwrap();
        let loaded = EngineConfig::load_or_default(&path).unwrap();
        assert_eq!(loaded.graphics.width, 640);
        assert_eq!(loaded.game.language, "american");
    }
}
