//! Versioned, checksummed, atomic 8-slot save game persistence for GTASARUST.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const CURRENT_SAVE_VERSION: u32 = 1;
pub const MAX_SAVE_SLOTS: usize = 8;
const SAVE_MAGIC: &[u8; 8] = b"GTASAV01";

#[derive(Debug, Error)]
pub enum SaveError {
    #[error("invalid save slot {0} (valid range: 1..={MAX_SAVE_SLOTS})")]
    InvalidSlot(usize),
    #[error("save file '{0}' is corrupted (checksum mismatch or invalid header)")]
    Corrupted(PathBuf),
    #[error("unsupported future save schema version {0}")]
    UnsupportedVersion(u32),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedWeaponSlot {
    pub weapon_name: String,
    pub ammo: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedVehicleState {
    pub model_id: u32,
    pub position: [f32; 3],
    pub yaw: f32,
    pub health: f32,
    pub color1: u8,
    pub color2: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveGameData {
    pub schema_version: u32,
    pub slot_title: String,
    pub game_hour: u8,
    pub game_minute: u8,
    pub game_day: u32,
    pub player_position: [f32; 3],
    pub player_yaw: f32,
    #[serde(default)]
    pub interior: i32,
    pub health: f32,
    pub max_health: f32,
    pub armor: f32,
    pub money: i64,
    pub respect: i32,
    pub wanted_stars: u8,
    pub weapons: Vec<SavedWeaponSlot>,
    pub completed_missions: Vec<String>,
    pub owned_properties: Vec<String>,
    pub garage_vehicles: Vec<SavedVehicleState>,
    pub total_playtime_seconds: f64,
}

impl Default for SaveGameData {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SAVE_VERSION,
            slot_title: "Grove Street - Home".to_string(),
            game_hour: 8,
            game_minute: 0,
            game_day: 1,
            // Grove Street cul-de-sac coordinates in Los Santos
            player_position: [2495.3, -1688.2, 13.5],
            player_yaw: 0.0,
            interior: 0,
            health: 100.0,
            max_health: 100.0,
            armor: 0.0,
            money: 350,
            respect: 0,
            wanted_stars: 0,
            weapons: vec![SavedWeaponSlot {
                weapon_name: "UNARMED".to_string(),
                ammo: 0,
            }],
            completed_missions: Vec::new(),
            owned_properties: vec!["johnson_house".to_string()],
            garage_vehicles: Vec::new(),
            total_playtime_seconds: 0.0,
        }
    }
}

pub fn slot_path(save_dir: &Path, slot: usize) -> Result<PathBuf, SaveError> {
    if !(1..=MAX_SAVE_SLOTS).contains(&slot) {
        return Err(SaveError::InvalidSlot(slot));
    }
    Ok(save_dir.join(format!("gtasarust_slot_{slot}.sav")))
}

/// Atomically write a checksummed, versioned save file (write temp file, then rename).
pub fn write_save_slot(
    save_dir: &Path,
    slot: usize,
    data: &SaveGameData,
) -> Result<PathBuf, SaveError> {
    let target = slot_path(save_dir, slot)?;
    std::fs::create_dir_all(save_dir)?;

    let payload = serde_json::to_vec_pretty(data)?;
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(&payload);
    let crc = hasher.finalize();

    let mut out = Vec::with_capacity(16 + payload.len());
    out.extend_from_slice(SAVE_MAGIC);
    out.extend_from_slice(&data.schema_version.to_le_bytes());
    out.extend_from_slice(&crc.to_le_bytes());
    out.extend_from_slice(&payload);

    let tmp_path = save_dir.join(format!("gtasarust_slot_{slot}.sav.tmp"));
    std::fs::write(&tmp_path, &out)?;
    std::fs::rename(&tmp_path, &target)?;
    Ok(target)
}

/// Read, verify CRC32 checksum, and migrate (if older version) a save slot.
pub fn read_save_slot(save_dir: &Path, slot: usize) -> Result<SaveGameData, SaveError> {
    let target = slot_path(save_dir, slot)?;
    let bytes = std::fs::read(&target)?;
    if bytes.len() < 16 || &bytes[0..8] != SAVE_MAGIC {
        return Err(SaveError::Corrupted(target));
    }

    let version = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
    if version > CURRENT_SAVE_VERSION {
        return Err(SaveError::UnsupportedVersion(version));
    }
    let expected_crc = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
    let payload = &bytes[16..];

    let mut hasher = crc32fast::Hasher::new();
    hasher.update(payload);
    if hasher.finalize() != expected_crc {
        return Err(SaveError::Corrupted(target));
    }

    let mut parsed: SaveGameData =
        serde_json::from_slice(payload).map_err(|_| SaveError::Corrupted(target.clone()))?;
    parsed.schema_version = CURRENT_SAVE_VERSION;
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_roundtrip_and_corruption_detection() {
        let dir = tempfile::tempdir().unwrap();
        let save = SaveGameData {
            money: 12_500,
            completed_missions: vec!["BIG_SMOKE".to_string()],
            ..Default::default()
        };

        let path = write_save_slot(dir.path(), 1, &save).unwrap();
        let loaded = read_save_slot(dir.path(), 1).unwrap();
        assert_eq!(loaded.money, 12_500);
        assert_eq!(loaded.completed_missions, vec!["BIG_SMOKE"]);

        // Flip a payload byte -> must detect corruption
        let mut raw = std::fs::read(&path).unwrap();
        let last = raw.len() - 2;
        raw[last] ^= 0xFF;
        std::fs::write(&path, raw).unwrap();
        assert!(matches!(
            read_save_slot(dir.path(), 1),
            Err(SaveError::Corrupted(_))
        ));
    }
}
