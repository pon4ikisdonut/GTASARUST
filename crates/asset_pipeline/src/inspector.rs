use crate::manifest::{ArchiveSummary, AssetManifest, AssetWarning, ManifestValidationReport};
use gtasarust_engine::GameDataRoot;
use gtasarust_gta_formats::{
    audio_cfg::AudioStreamStatus,
    col::parse_col_archive,
    dat::{GtaDat, HandlingCfg, TimecycDat, WaterDat, WeaponDat},
    gxt::GxtArchive,
    ide::IdeFile,
    ifp::IfpArchive,
    img::ImgArchive,
    ipl::IplFile,
    scm::inspect_scm_header,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("VFS error: {0}")]
    Vfs(#[from] gtasarust_engine::VfsError),
    #[error("format error: {0}")]
    Format(#[from] gtasarust_gta_formats::FormatError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationReport {
    pub manifest: AssetManifest,
    pub validation: ManifestValidationReport,
    pub audio_streams: Vec<AudioStreamStatus>,
}

/// Inspect an authorized GTA: San Andreas directory, parse core metadata,
/// verify archives and cross-references, and return a validated manifest report.
pub fn inspect_installation(game_dir: &Path) -> Result<InstallationReport, PipelineError> {
    let vfs = GameDataRoot::open(game_dir)?;
    let mut warnings = Vec::new();
    let mut missing_assets = Vec::new();

    // Fingerprint executable if present (read-only metadata only)
    let game_version = match vfs.resolve("gta_sa.exe") {
        Ok(exe_path) => {
            let bytes = std::fs::read(&exe_path)?;
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            let digest = hasher.finalize();
            let hash: String = digest.iter().map(|b| format!("{b:02X}")).collect();
            format!("PC ({} bytes, SHA256:{}..)", bytes.len(), &hash[..12])
        }
        Err(_) => "Unknown (gta_sa.exe absent)".to_string(),
    };

    let gta_dat_text = vfs.read_text_lossy("data/gta.dat")?;
    let default_dat_text = vfs.read_text_lossy("data/default.dat")?;
    let gta_dat = GtaDat::parse(&gta_dat_text);
    let default_dat = GtaDat::parse(&default_dat_text);

    // Inspect IMG archives
    let mut img_paths = vec![
        "models/gta3.img".to_string(),
        "models/gta_int.img".to_string(),
        "models/player.img".to_string(),
        "anim/anim.img".to_string(),
        "anim/cuts.img".to_string(),
    ];
    img_paths.extend(gta_dat.img_archives.clone());

    let mut archives = Vec::new();
    let mut img_entry_set: HashSet<String> = HashSet::new();
    let mut img_ipl_instance_count = 0usize;
    let mut img_ipl_car_spawn_count = 0usize;

    for rel in img_paths {
        match vfs.resolve(&rel) {
            Ok(resolved) => match ImgArchive::open(&resolved) {
                Ok(img) => {
                    let mut dff_count = 0usize;
                    let mut txd_count = 0usize;
                    let mut col_count = 0usize;
                    let mut ipl_count = 0usize;
                    let mut binary_ipl_entries = Vec::new();
                    for entry in img.entries() {
                        let lower = entry.name.to_ascii_lowercase();
                        img_entry_set.insert(lower.clone());
                        if lower.ends_with(".dff") {
                            dff_count += 1;
                        } else if lower.ends_with(".txd") {
                            txd_count += 1;
                        } else if lower.ends_with(".col") {
                            col_count += 1;
                        } else if lower.ends_with(".ipl") {
                            ipl_count += 1;
                            binary_ipl_entries.push(entry.clone());
                        }
                    }
                    for entry in binary_ipl_entries {
                        if let Ok(bytes) = img.read_entry_data(&entry)
                            && let Ok(ipl) = IplFile::parse_bytes(&bytes)
                        {
                            img_ipl_instance_count += ipl.instances.len();
                            img_ipl_car_spawn_count += ipl.car_spawns.len();
                        }
                    }
                    archives.push(ArchiveSummary {
                        relative_path: rel,
                        entry_count: img.entries().len(),
                        dff_count,
                        txd_count,
                        col_count,
                        ipl_count,
                    });
                }
                Err(err) => warnings.push(AssetWarning {
                    category: "archive".to_string(),
                    path: rel,
                    message: err.to_string(),
                }),
            },
            Err(_) => missing_assets.push(rel),
        }
    }

    // Parse all IDE definitions
    let mut ide_files = default_dat.ide_files;
    ide_files.extend(gta_dat.ide_files);
    let mut ide_object_count = 0usize;
    let mut ide_vehicle_count = 0usize;
    let mut ide_ped_count = 0usize;
    let mut referenced_models = Vec::new();
    let mut referenced_txds = Vec::new();

    for rel in &ide_files {
        match vfs.read_text_lossy(rel) {
            Ok(text) => {
                let ide = IdeFile::parse(&text);
                ide_object_count += ide.objects.len() + ide.weapons.len();
                ide_vehicle_count += ide.vehicles.len();
                ide_ped_count += ide.peds.len();
                for obj in &ide.objects {
                    if !obj.model_name.eq_ignore_ascii_case("null") {
                        referenced_models
                            .push(format!("{}.dff", obj.model_name.to_ascii_lowercase()));
                    }
                    if !obj.txd_name.eq_ignore_ascii_case("null")
                        && !obj.txd_name.eq_ignore_ascii_case("generic")
                    {
                        referenced_txds.push(format!("{}.txd", obj.txd_name.to_ascii_lowercase()));
                    }
                }
                for veh in &ide.vehicles {
                    referenced_models.push(format!("{}.dff", veh.model_name.to_ascii_lowercase()));
                    referenced_txds.push(format!("{}.txd", veh.txd_name.to_ascii_lowercase()));
                }
            }
            Err(_) => missing_assets.push(rel.clone()),
        }
    }

    // Parse all text IPL placements + binary streaming IPLs indexed above
    let mut ipl_files = default_dat.ipl_files;
    ipl_files.extend(gta_dat.ipl_files);
    let mut ipl_instance_count = img_ipl_instance_count;
    let mut ipl_car_spawn_count = img_ipl_car_spawn_count;
    for rel in &ipl_files {
        match vfs.read_bytes(rel) {
            Ok(bytes) => {
                if let Ok(ipl) = IplFile::parse_bytes(&bytes) {
                    ipl_instance_count += ipl.instances.len();
                    ipl_car_spawn_count += ipl.car_spawns.len();
                }
            }
            Err(_) => missing_assets.push(rel.clone()),
        }
    }

    // Check handling, weapon, timecyc, water, GXT, IFP, SCM, COL
    let handling_count = vfs
        .read_text_lossy("data/handling.cfg")
        .map(|t| HandlingCfg::parse(&t).vehicles.len())
        .unwrap_or(0);
    let weapon_count = vfs
        .read_text_lossy("data/weapon.dat")
        .map(|t| WeaponDat::parse(&t).weapons.len())
        .unwrap_or(0);
    let timecyc_entries = vfs
        .read_text_lossy("data/timecyc.dat")
        .map(|t| TimecycDat::parse(&t).entries.len())
        .unwrap_or(0);
    let water_quads = vfs
        .read_text_lossy("data/water.dat")
        .map(|t| WaterDat::parse(&t).quads.len())
        .unwrap_or(0);

    let gxt_entries = vfs
        .read_bytes("text/american.gxt")
        .ok()
        .and_then(|b| GxtArchive::parse(&b).ok())
        .map(|g| g.total_entries())
        .unwrap_or(0);

    let ifp_clips = vfs
        .read_bytes("anim/ped.ifp")
        .ok()
        .and_then(|b| IfpArchive::parse_header(&b).ok())
        .map(|i| i.clips.len())
        .unwrap_or(0);

    let scm_missions = vfs
        .read_bytes("data/script/main.scm")
        .ok()
        .and_then(|b| inspect_scm_header(&b).ok())
        .map(|s| s.header.mission_count)
        .unwrap_or(0);

    let mut col_models = 0usize;
    for col_rel in [
        "models/coll/peds.col",
        "models/coll/vehicles.col",
        "models/coll/weapons.col",
    ] {
        if let Ok(bytes) = vfs.read_bytes(col_rel)
            && let Ok(parsed) = parse_col_archive(&bytes)
        {
            col_models += parsed.len();
        }
    }

    // Check audio streams (gracefully detect 0-byte rips)
    let mut audio_streams = Vec::new();
    if let Ok(streams_dir) = vfs.resolve("audio/streams")
        && let Ok(read_dir) = std::fs::read_dir(streams_dir)
    {
        for entry in read_dir.flatten() {
            if let Ok(meta) = entry.metadata()
                && meta.is_file()
            {
                let name = entry.file_name().to_string_lossy().into_owned();
                let status = AudioStreamStatus::from_metadata(name.clone(), meta.len());
                if !status.is_present {
                    warnings.push(AssetWarning {
                        category: "audio_stream".to_string(),
                        path: format!("audio/streams/{name}"),
                        message: "0-byte placeholder stream file (missing station audio)"
                            .to_string(),
                    });
                }
                audio_streams.push(status);
            }
        }
        audio_streams.sort_by(|a, b| a.name.cmp(&b.name));
    }

    // Cross-reference missing models/TXDs against IMG archives
    let mut missing_dff_refs = 0usize;
    for m in &referenced_models {
        if !img_entry_set.contains(m) {
            missing_dff_refs += 1;
        }
    }
    let mut missing_txd_refs = 0usize;
    for t in &referenced_txds {
        if !img_entry_set.contains(t) {
            missing_txd_refs += 1;
        }
    }
    if missing_dff_refs > 0 {
        warnings.push(AssetWarning {
            category: "cross_ref".to_string(),
            path: "IDE->IMG".to_string(),
            message: format!("{missing_dff_refs} IDE object/vehicle model references not found in scanned IMG archives"),
        });
    }
    if missing_txd_refs > 0 {
        warnings.push(AssetWarning {
            category: "cross_ref".to_string(),
            path: "IDE->TXD".to_string(),
            message: format!(
                "{missing_txd_refs} IDE TXD references not found in scanned IMG archives"
            ),
        });
    }

    let total_models: usize = archives.iter().map(|a| a.dff_count).sum();
    let total_textures: usize = archives.iter().map(|a| a.txd_count).sum();
    let total_collisions: usize = archives.iter().map(|a| a.col_count).sum::<usize>() + col_models;

    let manifest = AssetManifest {
        schema_version: 1,
        game_version,
        archives,
        models: total_models,
        textures: total_textures,
        collisions: total_collisions,
        ide_objects: ide_object_count,
        ide_vehicles: ide_vehicle_count,
        ide_peds: ide_ped_count,
        ipl_instances: ipl_instance_count,
        ipl_car_spawns: ipl_car_spawn_count,
        handling_entries: handling_count,
        weapon_entries: weapon_count,
        timecyc_entries,
        water_quads,
        gxt_entries,
        ifp_clips,
        scm_missions,
        warnings: warnings.clone(),
    };

    let validation = ManifestValidationReport {
        valid: missing_assets.is_empty() && manifest.models > 0,
        missing_files: missing_assets,
        missing_dff_refs,
        missing_txd_refs,
        warning_count: warnings.len(),
    };

    Ok(InstallationReport {
        manifest,
        validation,
        audio_streams,
    })
}
