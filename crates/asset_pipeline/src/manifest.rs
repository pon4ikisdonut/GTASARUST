use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveSummary {
    pub relative_path: String,
    pub entry_count: usize,
    pub dff_count: usize,
    pub txd_count: usize,
    pub col_count: usize,
    pub ipl_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetWarning {
    pub category: String,
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetManifest {
    pub schema_version: u32,
    pub game_version: String,
    pub archives: Vec<ArchiveSummary>,
    pub models: usize,
    pub textures: usize,
    pub collisions: usize,
    pub ide_objects: usize,
    pub ide_vehicles: usize,
    pub ide_peds: usize,
    pub ipl_instances: usize,
    pub ipl_car_spawns: usize,
    pub handling_entries: usize,
    pub weapon_entries: usize,
    pub timecyc_entries: usize,
    pub water_quads: usize,
    pub gxt_entries: usize,
    pub ifp_clips: usize,
    pub scm_missions: usize,
    pub warnings: Vec<AssetWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestValidationReport {
    pub valid: bool,
    pub missing_files: Vec<String>,
    pub missing_dff_refs: usize,
    pub missing_txd_refs: usize,
    pub warning_count: usize,
}
