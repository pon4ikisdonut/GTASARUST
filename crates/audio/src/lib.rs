//! Audio engine built on `kira` with mixer channels (Master, Music, SFX, Voice),
//! GTA: San Andreas Ogg Vorbis radio streaming (`audio/streams/*`), PCM SFX bank
//! playback (`audio/SFX/*`), and graceful null fallback on headless systems.

use gtasarust_engine::AudioConfig;
use gtasarust_gta_formats::audio_cfg::{AudioStreamIndex, SfxBankIndex, TrackLookupEntry};
use kira::sound::PlaybackState;
use kira::sound::static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings};
use kira::{AudioManager, AudioManagerSettings, Decibels, DefaultBackend, Frame, Tween, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadioStationInfo {
    pub id: u8,
    pub code: String,
    pub display_name: String,
    pub stream_available: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SfxCue {
    Footstep,
    PistolShot,
    HeavyShot,
    Punch,
    CarHorn,
    Crash,
    Explosion,
    RadioClick,
    Respray,
    Siren,
}

struct DecodedRadioTrack {
    generation: u64,
    track_index: usize,
    pack_name: String,
    data: StaticSoundData,
}

pub struct AudioEngine {
    manager: Option<AudioManager<DefaultBackend>>,
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub voice_volume: f32,
    pub current_station_idx: usize,
    pub stations: Vec<RadioStationInfo>,
    pub backend_active: bool,
    audio_root: Option<PathBuf>,
    stream_index: Option<AudioStreamIndex>,
    station_tracks: Vec<Vec<TrackLookupEntry>>,
    station_cursors: Vec<usize>,
    in_vehicle: bool,
    radio_handle: Option<StaticSoundHandle>,
    radio_rx: Receiver<DecodedRadioTrack>,
    radio_tx: Sender<DecodedRadioTrack>,
    decode_generation: u64,
    radio_loading: bool,
    last_radio_db: f32,
    sfx_clips: HashMap<SfxCue, StaticSoundData>,
    footstep_variants: Vec<StaticSoundData>,
    footstep_cursor: usize,
    footstep_cooldown: f32,
    shot_cooldown: f32,
    horn_cooldown: f32,
    crash_cooldown: f32,
    siren_cooldown: f32,
    respray_cooldown: f32,
    engine_handle: Option<StaticSoundHandle>,
    engine_loop_clip: StaticSoundData,
    sa_sfx_loaded: usize,
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
                code: "PLAYBACK".to_string(),
                display_name: "Playback FM".to_string(),
                stream_available: false,
            },
            RadioStationInfo {
                id: 3,
                code: "KJAH".to_string(),
                display_name: "K-JAH West".to_string(),
                stream_available: false,
            },
            RadioStationInfo {
                id: 4,
                code: "CSR".to_string(),
                display_name: "CSR 103.9".to_string(),
                stream_available: false,
            },
            RadioStationInfo {
                id: 5,
                code: "KDST".to_string(),
                display_name: "K-DST".to_string(),
                stream_available: false,
            },
            RadioStationInfo {
                id: 6,
                code: "RADIOX".to_string(),
                display_name: "Radio X".to_string(),
                stream_available: false,
            },
            RadioStationInfo {
                id: 7,
                code: "SA_AMBIENCE".to_string(),
                display_name: "SA Ambience".to_string(),
                stream_available: false,
            },
        ];

        let station_count = stations.len();
        let (radio_tx, radio_rx) = mpsc::channel();
        let mut sfx_clips = HashMap::new();
        for cue in [
            SfxCue::Footstep,
            SfxCue::PistolShot,
            SfxCue::HeavyShot,
            SfxCue::Punch,
            SfxCue::CarHorn,
            SfxCue::Crash,
            SfxCue::Explosion,
            SfxCue::RadioClick,
            SfxCue::Respray,
            SfxCue::Siren,
        ] {
            sfx_clips.insert(cue, synthesize_fallback_sfx(cue));
        }

        Self {
            backend_active: manager.is_some(),
            manager,
            master_volume: config.master_volume,
            music_volume: config.music_volume,
            sfx_volume: config.sfx_volume,
            voice_volume: config.voice_volume,
            current_station_idx: 0,
            stations,
            audio_root: None,
            stream_index: None,
            station_tracks: vec![Vec::new(); station_count],
            station_cursors: vec![0; station_count],
            in_vehicle: false,
            radio_handle: None,
            radio_rx,
            radio_tx,
            decode_generation: 0,
            radio_loading: false,
            last_radio_db: 0.0,
            sfx_clips,
            footstep_variants: Vec::new(),
            footstep_cursor: 0,
            footstep_cooldown: 0.0,
            shot_cooldown: 0.0,
            horn_cooldown: 0.0,
            crash_cooldown: 0.0,
            siren_cooldown: 0.0,
            respray_cooldown: 0.0,
            engine_handle: None,
            engine_loop_clip: synthesize_engine_loop(),
            sa_sfx_loaded: 0,
        }
    }

    /// Indexes real GTA:SA `audio/streams/*` and `audio/SFX/*` from an authorized installation directory.
    pub fn attach_game_dir(&mut self, game_root: &Path) {
        let Some(audio_dir) = find_audio_root(game_root) else {
            return;
        };
        self.audio_root = Some(audio_dir.clone());

        if let Ok(stream_idx) = AudioStreamIndex::from_audio_dir(&audio_dir) {
            let station_pack_prefs: &[&[&str]] = &[
                &[],                      // 0: RADIO_OFF
                &["LS", "BEATS"],         // 1: RLS
                &["CH", "BEATS"],         // 2: PLAYBACK
                &["NJ"],                  // 3: KJAH
                &["CO", "NJ"],            // 4: CSR
                &["DS", "CH", "ADVERTS"], // 5: KDST
                &["RE", "ADVERTS"],       // 6: RADIOX
                &["AMBIENCE", "ADVERTS"], // 7: SA_AMBIENCE
            ];
            let mut active_stations = 0usize;
            for (idx, prefs) in station_pack_prefs.iter().enumerate() {
                if idx == 0 || idx >= self.stations.len() {
                    continue;
                }
                let mut chosen: Vec<TrackLookupEntry> = Vec::new();
                for &pack in *prefs {
                    let tracks = stream_idx.available_tracks_for_pack(&audio_dir, pack);
                    if !tracks.is_empty() {
                        let music_sized: Vec<TrackLookupEntry> = tracks
                            .iter()
                            .filter(|t| t.size >= 250_000)
                            .map(|t| (*t).clone())
                            .collect();
                        chosen = if !music_sized.is_empty() {
                            music_sized
                        } else {
                            tracks.into_iter().cloned().collect()
                        };
                        break;
                    }
                }
                self.stations[idx].stream_available = !chosen.is_empty();
                if !chosen.is_empty() {
                    active_stations += 1;
                }
                self.station_tracks[idx] = chosen;
            }
            let total_tracks: usize = self.station_tracks.iter().map(Vec::len).sum();
            info!(
                active_stations,
                total_tracks, "Indexed GTA:SA Ogg Vorbis audio streams"
            );
            self.stream_index = Some(stream_idx);
        }

        if let Ok(sfx_idx) = SfxBankIndex::from_audio_dir(&audio_dir) {
            self.footstep_variants.clear();
            for sound_idx in 1..=4 {
                if let Ok(clip) = sfx_idx.load_clip(&audio_dir, "FEET", 0, sound_idx) {
                    self.footstep_variants
                        .push(pcm_to_static_sound(clip.sample_rate, &clip.samples));
                    self.sa_sfx_loaded += 1;
                }
            }
            if let Some(first_step) = self.footstep_variants.first() {
                self.sfx_clips.insert(SfxCue::Footstep, first_step.clone());
            }

            let bank_map: &[(SfxCue, &str, usize, usize)] = &[
                (SfxCue::PistolShot, "GENRL", 136, 6),
                (SfxCue::HeavyShot, "GENRL", 136, 21),
                (SfxCue::Punch, "GENRL", 136, 15),
                (SfxCue::CarHorn, "GENRL", 32, 0),
                (SfxCue::Crash, "GENRL", 131, 2),
                (SfxCue::Explosion, "GENRL", 45, 0),
                (SfxCue::RadioClick, "GENRL", 52, 6),
            ];
            for &(cue, pak, bank, snd) in bank_map {
                if let Ok(clip) = sfx_idx.load_clip(&audio_dir, pak, bank, snd) {
                    self.sfx_clips
                        .insert(cue, pcm_to_static_sound(clip.sample_rate, &clip.samples));
                    self.sa_sfx_loaded += 1;
                }
            }
            info!(
                sa_sfx_loaded = self.sa_sfx_loaded,
                footstep_variants = self.footstep_variants.len(),
                "Loaded GTA:SA PCM SFX clips"
            );
        }
    }

    pub fn apply_config(&mut self, config: &AudioConfig) {
        self.master_volume = config.master_volume;
        self.music_volume = config.music_volume;
        self.sfx_volume = config.sfx_volume;
        self.voice_volume = config.voice_volume;
        let db = self.radio_decibels();
        if (db.0 - self.last_radio_db).abs() > 0.1
            && let Some(handle) = &mut self.radio_handle
        {
            handle.set_volume(db, Tween::default());
            self.last_radio_db = db.0;
        }
    }

    pub fn set_in_vehicle(&mut self, in_vehicle: bool) {
        if self.in_vehicle == in_vehicle {
            return;
        }
        self.in_vehicle = in_vehicle;
        if in_vehicle {
            if self.current_station_idx != 0 && self.radio_handle.is_none() && !self.radio_loading {
                self.request_station_playback();
            }
        } else {
            self.stop_radio_playback();
        }
    }

    pub fn cycle_radio_next(&mut self) -> &RadioStationInfo {
        self.current_station_idx = (self.current_station_idx + 1) % self.stations.len();
        self.play_sfx(SfxCue::RadioClick);
        self.stop_radio_playback();
        if self.in_vehicle && self.current_station_idx != 0 {
            self.request_station_playback();
        }
        &self.stations[self.current_station_idx]
    }

    pub fn current_station(&self) -> &RadioStationInfo {
        &self.stations[self.current_station_idx]
    }

    pub fn loaded_sfx_count(&self) -> usize {
        self.sa_sfx_loaded
    }

    pub fn available_radio_count(&self) -> usize {
        self.stations
            .iter()
            .filter(|s| s.id != 0 && s.stream_available)
            .count()
    }

    /// Synchronously decodes one indexed Ogg Vorbis track to PCM frames to validate stream extraction.
    pub fn verify_first_radio_track_decodes(&self) -> Option<(String, usize, u32)> {
        let audio_root = self.audio_root.as_ref()?;
        let stream_idx = self.stream_index.as_ref()?;
        let track = self.station_tracks.iter().flatten().next()?;
        let ogg = stream_idx.extract_ogg_track(audio_root, track).ok()?;
        let data = StaticSoundData::from_cursor(Cursor::new(ogg)).ok()?;
        Some((track.pack_name.clone(), data.frames.len(), data.sample_rate))
    }

    /// Drives background decoded radio track handoff, automatic next-track queueing, and SFX cooldowns.
    pub fn update(&mut self, dt: f32) {
        self.footstep_cooldown = (self.footstep_cooldown - dt).max(0.0);
        self.shot_cooldown = (self.shot_cooldown - dt).max(0.0);
        self.horn_cooldown = (self.horn_cooldown - dt).max(0.0);
        self.crash_cooldown = (self.crash_cooldown - dt).max(0.0);
        self.siren_cooldown = (self.siren_cooldown - dt).max(0.0);
        self.respray_cooldown = (self.respray_cooldown - dt).max(0.0);

        while let Ok(decoded) = self.radio_rx.try_recv() {
            if decoded.generation != self.decode_generation {
                continue;
            }
            self.radio_loading = false;
            if !self.in_vehicle || self.current_station_idx == 0 {
                continue;
            }
            let db = self.radio_decibels();
            let Some(manager) = &mut self.manager else {
                continue;
            };
            if let Some(mut prev) = self.radio_handle.take() {
                prev.stop(Tween::default());
            }
            let data = decoded.data.volume(db);
            match manager.play(data) {
                Ok(handle) => {
                    self.last_radio_db = db.0;
                    self.radio_handle = Some(handle);
                    info!(
                        station = %self.stations[self.current_station_idx].display_name,
                        pack = %decoded.pack_name,
                        track = decoded.track_index,
                        "Playing radio Ogg Vorbis track"
                    );
                }
                Err(err) => {
                    warn!("Failed to start radio playback: {err}");
                }
            }
        }

        if let Some(handle) = &self.radio_handle
            && handle.state() == PlaybackState::Stopped
            && self.in_vehicle
            && self.current_station_idx != 0
            && !self.radio_loading
        {
            self.radio_handle = None;
            self.request_station_playback();
        }
    }

    /// Plays one sound cue with per-cue anti-overlap gating and SFX mixer volume.
    pub fn play_sfx(&mut self, cue: SfxCue) {
        if self.manager.is_none() {
            return;
        }
        match cue {
            SfxCue::Footstep => {
                if self.footstep_cooldown > 0.0 {
                    return;
                }
                self.footstep_cooldown = 0.28;
            }
            SfxCue::PistolShot | SfxCue::HeavyShot | SfxCue::Punch => {
                if self.shot_cooldown > 0.0 {
                    return;
                }
                self.shot_cooldown = 0.08;
            }
            SfxCue::CarHorn => {
                if self.horn_cooldown > 0.0 {
                    return;
                }
                self.horn_cooldown = 0.55;
            }
            SfxCue::Crash => {
                if self.crash_cooldown > 0.0 {
                    return;
                }
                self.crash_cooldown = 0.45;
            }
            SfxCue::Explosion | SfxCue::RadioClick => {}
            SfxCue::Respray => {
                if self.respray_cooldown > 0.0 {
                    return;
                }
                self.respray_cooldown = 1.2;
            }
            SfxCue::Siren => {
                if self.siren_cooldown > 0.0 {
                    return;
                }
                self.siren_cooldown = 0.62;
            }
        }

        let clip = if cue == SfxCue::Footstep && !self.footstep_variants.is_empty() {
            let idx = self.footstep_cursor % self.footstep_variants.len();
            self.footstep_cursor = self.footstep_cursor.wrapping_add(1);
            self.footstep_variants[idx].clone()
        } else if let Some(found) = self.sfx_clips.get(&cue) {
            found.clone()
        } else {
            return;
        };

        let db = self.sfx_decibels(cue);
        if let Some(manager) = &mut self.manager {
            let _ = manager.play(clip.volume(db));
        }
    }

    fn stop_radio_playback(&mut self) {
        self.decode_generation = self.decode_generation.wrapping_add(1);
        self.radio_loading = false;
        if let Some(mut handle) = self.radio_handle.take() {
            handle.stop(Tween::default());
        }
    }

    fn request_station_playback(&mut self) {
        if self.manager.is_none() || self.current_station_idx == 0 {
            return;
        }
        let Some(audio_root) = self.audio_root.clone() else {
            return;
        };
        let Some(stream_idx) = self.stream_index.clone() else {
            return;
        };
        let tracks = &self.station_tracks[self.current_station_idx];
        if tracks.is_empty() {
            return;
        }
        let cursor = self.station_cursors[self.current_station_idx] % tracks.len();
        self.station_cursors[self.current_station_idx] = cursor.wrapping_add(1) % tracks.len();
        let entry = tracks[cursor].clone();

        self.decode_generation = self.decode_generation.wrapping_add(1);
        let generation = self.decode_generation;
        self.radio_loading = true;
        let tx = self.radio_tx.clone();

        std::thread::spawn(
            move || match stream_idx.extract_ogg_track(&audio_root, &entry) {
                Ok(ogg_bytes) => match StaticSoundData::from_cursor(Cursor::new(ogg_bytes)) {
                    Ok(data) => {
                        let _ = tx.send(DecodedRadioTrack {
                            generation,
                            track_index: entry.track_index,
                            pack_name: entry.pack_name,
                            data,
                        });
                    }
                    Err(err) => warn!(
                        track = entry.track_index,
                        "Failed to decode Ogg Vorbis radio stream: {err}"
                    ),
                },
                Err(err) => warn!(
                    track = entry.track_index,
                    "Failed to extract GTA:SA radio stream: {err}"
                ),
            },
        );
    }

    fn radio_decibels(&self) -> Decibels {
        linear_to_decibels(self.master_volume * self.music_volume * 0.65)
    }

    fn sfx_decibels(&self, cue: SfxCue) -> Decibels {
        let scale = match cue {
            SfxCue::Footstep => 0.35,
            SfxCue::RadioClick => 0.30,
            SfxCue::Siren => 0.42,
            _ => 0.70,
        };
        linear_to_decibels(self.master_volume * self.sfx_volume * scale)
    }

    /// Updates continuous vehicle engine RPM pitch/volume and active emergency siren playback.
    pub fn update_vehicle_audio(
        &mut self,
        in_vehicle: bool,
        speed_mps: f32,
        is_bicycle: bool,
        siren_on: bool,
        _dt: f32,
    ) {
        if siren_on && self.siren_cooldown <= 0.0 {
            self.play_sfx(SfxCue::Siren);
        }

        let Some(manager) = self.manager.as_mut() else {
            self.engine_handle = None;
            return;
        };

        if !in_vehicle || is_bicycle {
            if let Some(mut handle) = self.engine_handle.take() {
                handle.stop(kira::Tween::default());
            }
            return;
        }

        let speed_ratio = (speed_mps.abs() / 42.0).clamp(0.0, 1.0);
        let pitch = 0.75 + speed_ratio * 1.45;
        let engine_db =
            linear_to_decibels(self.master_volume * self.sfx_volume * (0.14 + speed_ratio * 0.18));

        if self.engine_handle.is_none() {
            let clip = self
                .engine_loop_clip
                .clone()
                .volume(engine_db)
                .playback_rate(f64::from(pitch));
            if let Ok(handle) = manager.play(clip) {
                self.engine_handle = Some(handle);
            }
        } else if let Some(handle) = self.engine_handle.as_mut() {
            handle.set_volume(engine_db, kira::Tween::default());
            handle.set_playback_rate(f64::from(pitch), kira::Tween::default());
        }
    }
}

fn linear_to_decibels(linear: f32) -> Decibels {
    if linear <= 0.001 {
        Decibels::SILENCE
    } else {
        Decibels(20.0 * linear.clamp(0.001, 1.5).log10())
    }
}

fn pcm_to_static_sound(sample_rate: u32, samples: &[i16]) -> StaticSoundData {
    let frames: Vec<Frame> = samples
        .iter()
        .map(|&s| Frame::from_mono(f32::from(s) / 32768.0))
        .collect();
    StaticSoundData {
        sample_rate: sample_rate.max(8000),
        frames: Arc::from(frames),
        settings: StaticSoundSettings {
            volume: Value::Fixed(Decibels::IDENTITY),
            ..StaticSoundSettings::new()
        },
        slice: None,
    }
}

fn synthesize_fallback_sfx(cue: SfxCue) -> StaticSoundData {
    let sample_rate = 22050u32;
    let duration_secs = match cue {
        SfxCue::Footstep => 0.06,
        SfxCue::PistolShot => 0.14,
        SfxCue::HeavyShot => 0.18,
        SfxCue::Punch => 0.09,
        SfxCue::CarHorn => 0.32,
        SfxCue::Crash => 0.18,
        SfxCue::Explosion => 0.48,
        SfxCue::RadioClick => 0.03,
        SfxCue::Respray => 0.85,
        SfxCue::Siren => 0.60,
    };
    let count = (sample_rate as f32 * duration_secs) as usize;
    let mut frames = Vec::with_capacity(count);
    let mut rng = 0x1234_5678u32;
    for i in 0..count {
        let t = i as f32 / sample_rate as f32;
        let env = (1.0 - (i as f32 / count as f32)).powi(2);
        rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = ((rng >> 16) as f32 / 32768.0) - 1.0;
        let sample = match cue {
            SfxCue::Footstep => {
                (noise * 0.35 + (t * 110.0 * std::f32::consts::TAU).sin() * 0.4) * env
            }
            SfxCue::PistolShot => {
                (noise * 0.8 + (t * 180.0 * std::f32::consts::TAU).sin() * 0.4) * env.powi(2)
            }
            SfxCue::HeavyShot => {
                (noise * 0.9 + (t * 120.0 * std::f32::consts::TAU).sin() * 0.5) * env
            }
            SfxCue::Punch => (noise * 0.5 + (t * 95.0 * std::f32::consts::TAU).sin() * 0.6) * env,
            SfxCue::CarHorn => {
                let s1 = (t * 392.0 * std::f32::consts::TAU).sin();
                let s2 = (t * 493.88 * std::f32::consts::TAU).sin();
                (s1 + s2) * 0.25 * (1.0 - (i as f32 / count as f32).powi(4))
            }
            SfxCue::Crash => noise * 0.75 * env,
            SfxCue::Explosion => {
                (noise * 0.9 + (t * 48.0 * std::f32::consts::TAU).sin() * 0.65) * env
            }
            SfxCue::RadioClick => (t * 1400.0 * std::f32::consts::TAU).sin() * 0.25 * env,
            SfxCue::Respray => {
                let hiss = (((i as f32) * 19.73).sin() * 43_758.547).fract() * 2.0 - 1.0;
                let pulse = (t * 16.0 * std::f32::consts::TAU).sin().abs() * 0.35 + 0.65;
                let attack = (t / 0.06).min(1.0);
                hiss * 0.32 * pulse * attack * env
            }
            SfxCue::Siren => {
                let sweep = 660.0 + ((t / duration_secs) * std::f32::consts::PI).sin() * 440.0;
                (t * sweep * std::f32::consts::TAU).sin() * 0.34 * env.sqrt()
            }
        };
        frames.push(Frame::from_mono(sample.clamp(-0.95, 0.95)));
    }
    StaticSoundData {
        sample_rate,
        frames: Arc::from(frames),
        settings: StaticSoundSettings::new(),
        slice: None,
    }
}

fn synthesize_engine_loop() -> StaticSoundData {
    let sample_rate = 22050u32;
    let duration_secs = 0.25_f32;
    let total = (sample_rate as f32 * duration_secs) as usize;
    let mut frames = Vec::with_capacity(total);
    for i in 0..total {
        let t = i as f32 / sample_rate as f32;
        let fundamental = (t * 56.0 * std::f32::consts::TAU).sin();
        let harmonic2 = (t * 112.0 * std::f32::consts::TAU).sin() * 0.5;
        let harmonic3 = (t * 168.0 * std::f32::consts::TAU).sin() * 0.25;
        let sample = (fundamental + harmonic2 + harmonic3) * 0.22;
        frames.push(Frame::from_mono(sample.clamp(-0.95, 0.95)));
    }
    StaticSoundData {
        sample_rate,
        frames: Arc::from(frames),
        settings: StaticSoundSettings::new().loop_region(0.0..),
        slice: None,
    }
}

fn find_audio_root(game_root: &Path) -> Option<PathBuf> {
    let direct = game_root.join("audio");
    if direct.is_dir() {
        return Some(direct);
    }
    let entries = std::fs::read_dir(game_root).ok()?;
    for entry in entries.flatten() {
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false)
            && entry
                .file_name()
                .to_str()
                .is_some_and(|n| n.eq_ignore_ascii_case("audio"))
        {
            return Some(entry.path());
        }
    }
    None
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
        audio.set_in_vehicle(true);
        audio.play_sfx(SfxCue::PistolShot);
        audio.play_sfx(SfxCue::Footstep);
        audio.play_sfx(SfxCue::Respray);
        audio.play_sfx(SfxCue::Siren);
        audio.update_vehicle_audio(true, 18.0, false, true, 0.016);
        audio.update_vehicle_audio(false, 0.0, false, false, 0.016);
        audio.update(0.016);
        audio.set_in_vehicle(false);
    }
}
