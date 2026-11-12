//! Audio engine built on `kira` with mixer channels (Master, Music, SFX, Voice),
//! radio station cycling, and graceful null fallback on headless CI systems.

use gtasarust_engine::AudioConfig;
use kira::{AudioManager, AudioManagerSettings, DefaultBackend};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadioStationInfo {
    pub id: u8,
    pub code: String,
    pub display_name: String,
    pub stream_available: bool,
}

pub struct AudioEngine {
    _manager: Option<AudioManager<DefaultBackend>>,
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub voice_volume: f32,
    pub current_station_idx: usize,
    pub stations: Vec<RadioStationInfo>,
    pub backend_active: bool,
}

impl AudioEngine {
    pub fn new(config: &AudioConfig, headless: bool) -> Self {
        let manager = if headless {
            None
        } else {
            match AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()) {
                Ok(m) => {
                    info!("Kira audio backend initialized");
                    Some(m)
                }
                Err(err) => {
                    warn!("Audio output device unavailable ({err}); running with silent mixer");
                    None
                }
            }
        };

        let stations = vec![
            RadioStationInfo {
                id: 0,
                code: "RADIO_OFF".to_string(),
                display_name: "Radio Off".to_string(),
                stream_available: true,
            },
            RadioStationInfo {
                id: 1,
                code: "RLS".to_string(),
                display_name: "Radio Los Santos".to_string(),
                stream_available: false,
            },
            RadioStationInfo {
                id: 2,
                code: "CSR".to_string(),
                display_name: "CSR 103.9".to_string(),
                stream_available: false,
            },
            RadioStationInfo {
                id: 3,
                code: "KDST".to_string(),
                display_name: "K-DST".to_string(),
                stream_available: false,
            },
            RadioStationInfo {
                id: 4,
                code: "RADIOX".to_string(),
                display_name: "Radio X".to_string(),
                stream_available: false,
            },
        ];

        Self {
            backend_active: manager.is_some(),
            _manager: manager,
            master_volume: config.master_volume,
            music_volume: config.music_volume,
            sfx_volume: config.sfx_volume,
            voice_volume: config.voice_volume,
            current_station_idx: 0,
            stations,
        }
    }

    pub fn apply_config(&mut self, config: &AudioConfig) {
        self.master_volume = config.master_volume;
        self.music_volume = config.music_volume;
        self.sfx_volume = config.sfx_volume;
        self.voice_volume = config.voice_volume;
    }

    pub fn cycle_radio_next(&mut self) -> &RadioStationInfo {
        self.current_station_idx = (self.current_station_idx + 1) % self.stations.len();
        &self.stations[self.current_station_idx]
    }

    pub fn current_station(&self) -> &RadioStationInfo {
        &self.stations[self.current_station_idx]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn radio_cycles_cleanly_even_headless() {
        let mut audio = AudioEngine::new(&AudioConfig::default(), true);
        assert_eq!(audio.current_station().code, "RADIO_OFF");
        let next = audio.cycle_radio_next().code.clone();
        assert_eq!(next, "RLS");
    }
}
