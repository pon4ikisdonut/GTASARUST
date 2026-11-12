//! Complete San Andreas world representation: IDE/IPL registry, spatial sector
//! streaming across Los Santos / San Fierro / Las Venturas / Countryside,
//! zones, water quads, and timecycle/weather atmosphere calculation.

use glam::{Quat, Vec3, Vec4};
use gtasarust_engine::GameDataRoot;
use gtasarust_gta_formats::{
    dat::{GtaDat, TimecycDat, WaterDat, WaterQuad},
    ide::{IdeFile, IdeObjectDef, IdePedDef, IdeVehicleDef},
    img::ImgArchive,
    ipl::{IplCarSpawn, IplEntryExit, IplFile, IplInstance, IplZoneBox},
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::info;

pub const SECTOR_SIZE: f32 = 150.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SectorCoord {
    pub x: i32,
    pub y: i32,
}

impl SectorCoord {
    pub fn from_world_pos(pos: Vec3) -> Self {
        Self {
            x: (pos.x / SECTOR_SIZE).floor() as i32,
            y: (pos.y / SECTOR_SIZE).floor() as i32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WeatherType {
    #[default]
    ExtraSunnyLA,
    SunnyLA,
    ExtraSunnySmog,
    Cloudy,
    Rainy,
    Foggy,
    SandstormDesert,
}

#[derive(Debug, Clone, Copy)]
pub struct AtmosphereState {
    pub sun_dir: Vec3,
    pub sun_color: Vec4,
    pub ambient_color: Vec4,
    pub sky_top: Vec4,
    pub sky_bottom: Vec4,
    pub fog_color: Vec4,
    pub fog_start: f32,
    pub fog_end: f32,
    pub rain_intensity: f32,
}

#[derive(Debug, Clone)]
pub struct WorldPlacement {
    pub model_id: u32,
    pub model_name: String,
    pub txd_name: String,
    pub position: Vec3,
    pub rotation: Quat,
    pub draw_distance: f32,
    pub is_lod: bool,
    pub interior: i32,
}

#[derive(Debug, Default)]
pub struct WorldManager {
    pub objects: HashMap<u32, IdeObjectDef>,
    pub vehicles: HashMap<u32, IdeVehicleDef>,
    pub peds: HashMap<u32, IdePedDef>,
    pub sectors: HashMap<SectorCoord, Vec<WorldPlacement>>,
    pub active_sectors: HashSet<SectorCoord>,
    pub car_spawns: Vec<IplCarSpawn>,
    pub entry_exits: Vec<IplEntryExit>,
    pub zones: Vec<IplZoneBox>,
    pub water_quads: Vec<WaterQuad>,
    pub timecyc: TimecycDat,
    pub weather: WeatherType,
    pub loaded_from_original_data: bool,
}

impl WorldManager {
    /// Load the complete San Andreas world definition (`gta.dat`, `default.dat`, IDEs,
    /// text IPLs, binary stream IPLs from `gta3.img`, zones, water, and timecycle).
    pub fn load_from_game_dir(vfs: &GameDataRoot, include_binary_ipls: bool) -> Self {
        let mut world = Self::default();
        let Ok(gta_dat_text) = vfs.read_text_lossy("data/gta.dat") else {
            return world;
        };
        let default_dat_text = vfs.read_text_lossy("data/default.dat").unwrap_or_default();
        let gta_dat = GtaDat::parse(&gta_dat_text);
        let default_dat = GtaDat::parse(&default_dat_text);

        // 1. Load IDE registries
        for rel in default_dat.ide_files.iter().chain(gta_dat.ide_files.iter()) {
            if let Ok(text) = vfs.read_text_lossy(rel) {
                let ide = IdeFile::parse(&text);
                for obj in ide.objects.into_iter().chain(ide.weapons) {
                    world.objects.insert(obj.id, obj);
                }
                for veh in ide.vehicles {
                    world.vehicles.insert(veh.id, veh);
                }
                for ped in ide.peds {
                    world.peds.insert(ped.id, ped);
                }
            }
        }

        // 2. Load text IPLs & zones
        let mut raw_instances = Vec::new();
        for rel in default_dat.ipl_files.iter().chain(gta_dat.ipl_files.iter()) {
            if let Ok(bytes) = vfs.read_bytes(rel)
                && let Ok(ipl) = IplFile::parse_bytes(&bytes)
            {
                raw_instances.extend(ipl.instances);
                world.car_spawns.extend(ipl.car_spawns);
                world.entry_exits.extend(ipl.entry_exits);
                world.zones.extend(ipl.zones);
            }
        }

        // 3. Load binary streaming IPLs inside `models/gta3.img`
        if include_binary_ipls
            && let Ok(gta3_path) = vfs.resolve("models/gta3.img")
            && let Ok(img) = ImgArchive::open(&gta3_path)
        {
            let ipl_entries: Vec<_> = img
                .entries()
                .iter()
                .filter(|e| e.name.to_ascii_lowercase().ends_with(".ipl"))
                .cloned()
                .collect();
            for entry in ipl_entries {
                if let Ok(bytes) = img.read_entry_data(&entry)
                    && let Ok(ipl) = IplFile::parse_bytes(&bytes)
                {
                    raw_instances.extend(ipl.instances);
                    world.car_spawns.extend(ipl.car_spawns);
                }
            }
        }

        // 4. Index placements into spatial streaming sectors
        for inst in raw_instances {
            world.insert_ipl_instance(inst);
        }

        // 5. Load water and timecycle
        if let Ok(water_txt) = vfs.read_text_lossy("data/water.dat") {
            world.water_quads = WaterDat::parse(&water_txt).quads;
        }
        if let Ok(tc_txt) = vfs.read_text_lossy("data/timecyc.dat") {
            world.timecyc = TimecycDat::parse(&tc_txt);
        }

        world.loaded_from_original_data = !world.sectors.is_empty();
        info!(
            objects = world.objects.len(),
            vehicles = world.vehicles.len(),
            sectors = world.sectors.len(),
            placements = world.total_placements(),
            zones = world.zones.len(),
            water_quads = world.water_quads.len(),
            "World definitions loaded"
        );

        world
    }

    /// Create a synthetic fallback Los Santos / San Andreas slice when running without original files
    /// (e.g., CI smoke tests or unconfigured launches).
    pub fn create_fallback_san_andreas() -> Self {
        let mut world = Self::default();
        // Representative Grove Street / Downtown Los Santos / San Fierro / Las Venturas landmarks
        let landmarks = [
            (1001, "grove_street_hub", [2495.0, -1688.0, 13.5], 300.0),
            (1002, "ls_downtown_tower", [1544.0, -1353.0, 16.0], 500.0),
            (1003, "sf_bridge_pylon", [-2680.0, 1275.0, 18.0], 600.0),
            (1004, "lv_strip_pyramid", [2170.0, 1285.0, 11.0], 500.0),
        ];
        for (id, name, pos, dd) in landmarks {
            world.objects.insert(
                id,
                IdeObjectDef {
                    id,
                    model_name: name.to_string(),
                    txd_name: "generic".to_string(),
                    draw_distance: dd,
                    flags: 0,
                    time_on: None,
                    time_off: None,
                    anim_name: None,
                },
            );
            world.insert_ipl_instance(IplInstance {
                id,
                model_name: name.to_string(),
                interior: 0,
                position: pos,
                rotation: [0.0, 0.0, 0.0, 1.0],
                lod: -1,
            });
        }
        world.zones.push(IplZoneBox {
            name: "GAN".to_string(),
            zone_type: 0,
            min: [2220.0, -1850.0, -50.0],
            max: [2630.0, -1580.0, 200.0],
            level: 1,
            text_label: "Ganton".to_string(),
        });
        world
    }

    fn insert_ipl_instance(&mut self, inst: IplInstance) {
        let def = self.objects.get(&inst.id);
        let model_name = if !inst.model_name.is_empty() {
            inst.model_name
        } else if let Some(d) = def {
            d.model_name.clone()
        } else {
            format!("model_{}", inst.id)
        };
        let txd_name = def
            .map(|d| d.txd_name.clone())
            .unwrap_or_else(|| "generic".to_string());
        let draw_distance = def.map(|d| d.draw_distance).unwrap_or(150.0);
        let is_lod = model_name.to_ascii_lowercase().starts_with("lod");

        let pos = Vec3::from_array(inst.position);
        // GTA IPL quaternions are (x, y, z, w) representing inverse rotation relative to RW;
        // conjugate ensures proper Z-up world orientation.
        let rot = Quat::from_xyzw(
            inst.rotation[0],
            inst.rotation[1],
            inst.rotation[2],
            inst.rotation[3],
        )
        .normalize()
        .conjugate();

        let sector = SectorCoord::from_world_pos(pos);
        self.sectors
            .entry(sector)
            .or_default()
            .push(WorldPlacement {
                model_id: inst.id,
                model_name,
                txd_name,
                position: pos,
                rotation: rot,
                draw_distance,
                is_lod,
                interior: inst.interior,
            });
    }

    pub fn total_placements(&self) -> usize {
        self.sectors.values().map(Vec::len).sum()
    }

    /// Update active spatial streaming sectors around `focus` based on `draw_distance`.
    pub fn update_streaming(&mut self, focus: Vec3, draw_distance: f32) {
        let center = SectorCoord::from_world_pos(focus);
        let radius = ((draw_distance / SECTOR_SIZE).ceil() as i32).clamp(1, 12);
        self.active_sectors.clear();
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let sc = SectorCoord {
                    x: center.x + dx,
                    y: center.y + dy,
                };
                if self.sectors.contains_key(&sc) {
                    self.active_sectors.insert(sc);
                }
            }
        }
    }

    /// Iterate placements currently inside active streaming sectors (matching `interior`).
    pub fn active_placements(&self, interior: i32) -> impl Iterator<Item = &WorldPlacement> {
        self.active_sectors
            .iter()
            .filter_map(|sc| self.sectors.get(sc))
            .flatten()
            .filter(move |p| p.interior == interior)
    }

    /// Resolve current San Andreas region/zone name for a world coordinate.
    pub fn zone_name_at(&self, pos: Vec3) -> &str {
        for z in &self.zones {
            if pos.x >= z.min[0]
                && pos.x <= z.max[0]
                && pos.y >= z.min[1]
                && pos.y <= z.max[1]
                && pos.z >= z.min[2]
                && pos.z <= z.max[2]
            {
                if !z.text_label.is_empty() && z.text_label != "UNUSED" {
                    return &z.text_label;
                }
                return &z.name;
            }
        }
        if pos.x < -750.0 && pos.y > -750.0 {
            "San Fierro"
        } else if pos.x > 650.0 && pos.y > 450.0 {
            "Las Venturas"
        } else if pos.x > 0.0 && pos.y < -700.0 {
            "Los Santos"
        } else {
            "San Andreas Countryside"
        }
    }

    /// Compute sky/sun/ambient/fog parameters for `time_hours` in `[0, 24)` and active weather.
    pub fn atmosphere(&self, time_hours: f32, draw_distance: f32) -> AtmosphereState {
        let angle = ((time_hours - 6.0) / 24.0) * std::f32::consts::TAU;
        let sun_z = angle.sin();
        let sun_x = angle.cos();
        let sun_dir = Vec3::new(sun_x * 0.75, 0.35, sun_z.max(-0.2)).normalize();

        let daylight = ((sun_z + 0.15) / 1.15).clamp(0.0, 1.0);
        let twilight = (1.0 - (sun_z.abs() * 3.0).clamp(0.0, 1.0)) * 0.85;

        let mut sky_top =
            Vec4::new(0.06, 0.09, 0.18, 1.0).lerp(Vec4::new(0.18, 0.44, 0.82, 1.0), daylight);
        let mut sky_bottom =
            Vec4::new(0.12, 0.12, 0.20, 1.0).lerp(Vec4::new(0.68, 0.82, 0.96, 1.0), daylight);
        // Classic Los Santos warm orange/gold dusk & dawn tint
        sky_bottom = sky_bottom.lerp(Vec4::new(0.96, 0.56, 0.24, 1.0), twilight);

        let mut sun_color =
            Vec4::new(0.35, 0.40, 0.60, 0.25).lerp(Vec4::new(1.0, 0.94, 0.82, 1.15), daylight);
        sun_color = sun_color.lerp(Vec4::new(1.0, 0.62, 0.30, 0.95), twilight);

        let ambient_color =
            Vec4::new(0.14, 0.15, 0.22, 1.0).lerp(Vec4::new(0.40, 0.39, 0.36, 1.0), daylight);

        let mut fog_color = sky_bottom;
        let mut fog_start = draw_distance * 0.45;
        let mut fog_end = draw_distance;
        let mut rain_intensity = 0.0;

        match self.weather {
            WeatherType::Rainy => {
                sky_top *= 0.45;
                sky_bottom = Vec4::new(0.32, 0.36, 0.40, 1.0);
                fog_color = sky_bottom;
                fog_start *= 0.45;
                fog_end *= 0.7;
                rain_intensity = 1.0;
            }
            WeatherType::Foggy => {
                fog_start = 40.0;
                fog_end = (draw_distance * 0.35).max(160.0);
                fog_color = Vec4::new(0.65, 0.68, 0.70, 1.0);
            }
            WeatherType::SandstormDesert => {
                fog_start = 60.0;
                fog_end = (draw_distance * 0.45).max(220.0);
                fog_color = Vec4::new(0.76, 0.58, 0.36, 1.0);
                sky_bottom = fog_color;
            }
            _ => {}
        }

        AtmosphereState {
            sun_dir,
            sun_color,
            ambient_color,
            sky_top,
            sky_bottom,
            fog_color,
            fog_start,
            fog_end,
            rain_intensity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_world_streams_grove_street_sector() {
        let mut world = WorldManager::create_fallback_san_andreas();
        let grove = Vec3::new(2495.0, -1688.0, 13.5);
        world.update_streaming(grove, 300.0);
        assert!(world.active_placements(0).count() >= 1);
        assert_eq!(world.zone_name_at(grove), "Ganton");
    }
}
