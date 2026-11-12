//! Complete San Andreas world representation: IDE/IPL registry, spatial sector
//! streaming across Los Santos / San Fierro / Las Venturas / Countryside,
//! zones, water quads, and timecycle/weather atmosphere calculation.

use glam::{Quat, Vec3, Vec4};
use gtasarust_engine::GameDataRoot;
use gtasarust_gta_formats::{
    col::{ColModel, parse_col_archive},
    dat::{GtaDat, TimecycDat, WaterDat, WaterQuad},
    ide::{IdeFile, IdeObjectDef, IdePedDef, IdeVehicleDef},
    img::ImgArchive,
    ipl::{IplCarSpawn, IplEntryExit, IplFile, IplInstance, IplZoneBox},
    paths::PathsFile,
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
    pub ambient_obj_color: Vec4,
    pub sky_top: Vec4,
    pub sky_bottom: Vec4,
    pub fog_color: Vec4,
    pub water_color: Vec4,
    pub fog_start: f32,
    pub fog_end: f32,
    pub rain_intensity: f32,
    /// Building day/night blend (`0.0 = day prelit`, `1.0 = night prelit`) matching SA `CustomBuildingDNPipeline`.
    pub dn_balance: f32,
}

#[derive(Debug, Clone)]
pub struct WorldPlacement {
    pub model_id: u32,
    pub model_name: String,
    pub txd_name: String,
    pub position: Vec3,
    pub rotation: Quat,
    pub draw_distance: f32,
    pub flags: u32,
    pub time_on: Option<u8>,
    pub time_off: Option<u8>,
    pub is_lod: bool,
    pub interior: i32,
}

/// Directed navigation waypoint in San Andreas road/pedestrian path graph.
#[derive(Debug, Clone)]
pub struct WorldPathNode {
    pub position: Vec3,
    pub neighbors: Vec<(u16, u16)>,
    pub is_ped: bool,
    pub is_water: bool,
    pub is_highway: bool,
}

/// Complete San Andreas vehicle & pedestrian navigation graph parsed from `NODES0.DAT`..`NODES63.DAT`.
#[derive(Debug, Default)]
pub struct PathGraph {
    pub nodes: HashMap<(u16, u16), WorldPathNode>,
    pub areas: HashMap<u16, Vec<(u16, u16)>>,
    pub total_links: usize,
}

impl PathGraph {
    pub fn area_xy(pos: Vec3) -> (i32, i32) {
        let ax = ((pos.x + 3000.0) / 750.0).floor().clamp(0.0, 7.0) as i32;
        let ay = ((pos.y + 3000.0) / 750.0).floor().clamp(0.0, 7.0) as i32;
        (ax, ay)
    }

    pub fn insert_paths_file(&mut self, pf: PathsFile) {
        for n in pf.nodes {
            let start = n.link_start as usize;
            let end = start
                .saturating_add(n.link_count as usize)
                .min(pf.links.len());
            let mut neighbors = Vec::with_capacity(end.saturating_sub(start));
            if start < end {
                for l in &pf.links[start..end] {
                    neighbors.push((l.target_area_id, l.target_node_id));
                }
            }
            self.total_links += neighbors.len();
            let key = (n.area_id, n.node_id);
            self.nodes.insert(
                key,
                WorldPathNode {
                    position: Vec3::from_array(n.position),
                    neighbors,
                    is_ped: n.is_ped,
                    is_water: n.is_water,
                    is_highway: n.is_highway,
                },
            );
            self.areas.entry(n.area_id).or_default().push(key);
        }
    }

    /// Collect candidate waypoint keys within `radius` around `pos` (filtering road vs sidewalk).
    pub fn nodes_near(&self, pos: Vec3, radius: f32, want_ped: bool) -> Vec<(u16, u16)> {
        let (cx, cy) = Self::area_xy(pos);
        let r2 = radius * radius;
        let mut out = Vec::new();
        for dy in -1..=1 {
            for dx in -1..=1 {
                let ax = cx + dx;
                let ay = cy + dy;
                if !(0..8).contains(&ax) || !(0..8).contains(&ay) {
                    continue;
                }
                let area_id = (ay * 8 + ax) as u16;
                if let Some(bucket) = self.areas.get(&area_id) {
                    for &k in bucket {
                        if let Some(node) = self.nodes.get(&k)
                            && node.is_ped == want_ped
                            && !node.is_water
                            && node.position.truncate().distance_squared(pos.truncate()) <= r2
                        {
                            out.push(k);
                        }
                    }
                }
            }
        }
        out
    }

    /// Find the closest path node within `max_dist` of `pos`.
    pub fn nearest_node(&self, pos: Vec3, want_ped: bool, max_dist: f32) -> Option<(u16, u16)> {
        let (cx, cy) = Self::area_xy(pos);
        let mut best_d2 = max_dist * max_dist;
        let mut best = None;
        for dy in -1..=1 {
            for dx in -1..=1 {
                let ax = cx + dx;
                let ay = cy + dy;
                if !(0..8).contains(&ax) || !(0..8).contains(&ay) {
                    continue;
                }
                let area_id = (ay * 8 + ax) as u16;
                if let Some(bucket) = self.areas.get(&area_id) {
                    for &k in bucket {
                        if let Some(node) = self.nodes.get(&k)
                            && node.is_ped == want_ped
                            && !node.is_water
                        {
                            let d2 = node.position.distance_squared(pos);
                            if d2 < best_d2 {
                                best_d2 = d2;
                                best = Some(k);
                            }
                        }
                    }
                }
            }
        }
        best
    }

    /// Choose next connected waypoint matching `want_ped`, avoiding immediate U-turn `prev` when possible.
    pub fn pick_next_node(
        &self,
        current: (u16, u16),
        prev: Option<(u16, u16)>,
        seed: u64,
        want_ped: bool,
    ) -> Option<(u16, u16)> {
        let node = self.nodes.get(&current)?;
        let mut forward: Vec<(u16, u16)> = Vec::new();
        let mut fallback: Vec<(u16, u16)> = Vec::new();
        for &nbr in &node.neighbors {
            if let Some(target) = self.nodes.get(&nbr)
                && target.is_ped == want_ped
                && !target.is_water
            {
                if Some(nbr) != prev {
                    forward.push(nbr);
                } else {
                    fallback.push(nbr);
                }
            }
        }
        let choices = if !forward.is_empty() {
            &forward
        } else {
            &fallback
        };
        if choices.is_empty() {
            return None;
        }
        Some(choices[(seed as usize) % choices.len()])
    }
}

#[derive(Debug, Default)]
pub struct WorldManager {
    pub objects: HashMap<u32, IdeObjectDef>,
    pub vehicles: HashMap<u32, IdeVehicleDef>,
    pub peds: HashMap<u32, IdePedDef>,
    /// Parent TXD lookup table populated from IDE `txdp` sections (child -> parent, lowercase).
    pub txd_parents: HashMap<String, String>,
    pub sectors: HashMap<SectorCoord, Vec<WorldPlacement>>,
    pub active_sectors: HashSet<SectorCoord>,
    pub car_spawns: Vec<IplCarSpawn>,
    pub entry_exits: Vec<IplEntryExit>,
    pub zones: Vec<IplZoneBox>,
    pub water_quads: Vec<WaterQuad>,
    pub timecyc: TimecycDat,
    pub weather: WeatherType,
    pub col_models: HashMap<String, ColModel>,
    pub path_graph: PathGraph,
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

        // 1. Load IDE registries (including `txdp` parent chains and `data/maps/txd.ide`)
        for rel in default_dat
            .ide_files
            .iter()
            .chain(gta_dat.ide_files.iter())
            .chain(std::iter::once(&"data/maps/txd.ide".to_string()))
        {
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
                for tp in ide.txd_parents {
                    let child = tp.child_txd.trim().to_ascii_lowercase();
                    let parent = tp.parent_txd.trim().to_ascii_lowercase();
                    if !child.is_empty() && !parent.is_empty() && child != parent {
                        world.txd_parents.insert(child, parent);
                    }
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

        // 3. Load binary streaming IPLs and collision archives inside `models/gta3.img` & `models/coll/*.col`
        if include_binary_ipls {
            for col_rel in default_dat.col_files.iter().chain(gta_dat.col_files.iter()) {
                if let Ok(bytes) = vfs.read_bytes(col_rel)
                    && let Ok(parsed) = parse_col_archive(&bytes)
                {
                    for model in parsed {
                        insert_col_model(&mut world.col_models, model);
                    }
                }
            }

            if let Ok(gta3_path) = vfs.resolve("models/gta3.img")
                && let Ok(img) = ImgArchive::open(&gta3_path)
            {
                let mut ipl_entries = Vec::new();
                let mut col_entries = Vec::new();
                let mut node_entries = Vec::new();
                for entry in img.entries() {
                    let lower = entry.name.to_ascii_lowercase();
                    if lower.ends_with(".ipl") {
                        ipl_entries.push(entry.clone());
                    } else if lower.ends_with(".col") {
                        col_entries.push(entry.clone());
                    } else if lower.starts_with("nodes") && lower.ends_with(".dat") {
                        node_entries.push(entry.clone());
                    }
                }
                for entry in ipl_entries {
                    if let Ok(bytes) = img.read_entry_data(&entry)
                        && let Ok(ipl) = IplFile::parse_bytes(&bytes)
                    {
                        raw_instances.extend(ipl.instances);
                        world.car_spawns.extend(ipl.car_spawns);
                    }
                }
                for entry in col_entries {
                    if let Ok(bytes) = img.read_entry_data(&entry)
                        && let Ok(parsed) = parse_col_archive(&bytes)
                    {
                        for model in parsed {
                            insert_col_model(&mut world.col_models, model);
                        }
                    }
                }
                for entry in node_entries {
                    if let Ok(bytes) = img.read_entry_data(&entry)
                        && let Ok(pf) = PathsFile::parse_bytes(&bytes)
                    {
                        world.path_graph.insert_paths_file(pf);
                    }
                }
            }

            if let Ok(gta_int_path) = vfs.resolve("models/gta_int.img")
                && let Ok(img) = ImgArchive::open(&gta_int_path)
            {
                let mut int_ipls = Vec::new();
                let mut int_cols = Vec::new();
                for e in img.entries() {
                    let lower = e.name.to_ascii_lowercase();
                    if lower.ends_with(".ipl") {
                        int_ipls.push(e.clone());
                    } else if lower.ends_with(".col") {
                        int_cols.push(e.clone());
                    }
                }
                for entry in int_ipls {
                    if let Ok(bytes) = img.read_entry_data(&entry)
                        && let Ok(ipl) = IplFile::parse_bytes(&bytes)
                    {
                        raw_instances.extend(ipl.instances);
                        world.car_spawns.extend(ipl.car_spawns);
                        world.entry_exits.extend(ipl.entry_exits);
                    }
                }
                for entry in int_cols {
                    if let Ok(bytes) = img.read_entry_data(&entry)
                        && let Ok(parsed) = parse_col_archive(&bytes)
                    {
                        for model in parsed {
                            insert_col_model(&mut world.col_models, model);
                        }
                    }
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
            car_spawns = world.car_spawns.len(),
            col_models = world.col_models.len(),
            path_nodes = world.path_graph.nodes.len(),
            path_links = world.path_graph.total_links,
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
        let flags = def.map(|d| d.flags).unwrap_or(0);
        let time_on = def.and_then(|d| d.time_on);
        let time_off = def.and_then(|d| d.time_off);
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
                flags,
                time_on,
                time_off,
                is_lod,
                interior: inst.interior,
            });
    }

    /// Resolve a TXD name followed by its `txdp` parent chain (up to 8 hops, cycle-safe).
    pub fn resolve_txd_chain(&self, txd_name: &str) -> Vec<String> {
        let mut chain = Vec::with_capacity(4);
        let mut cur = txd_name.trim().to_ascii_lowercase();
        for _ in 0..8 {
            if cur.is_empty() || chain.contains(&cur) {
                break;
            }
            let next = self.txd_parents.get(&cur).cloned();
            chain.push(cur);
            match next {
                Some(p) => cur = p,
                None => break,
            }
        }
        chain
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

    /// Iterate `ENEX` entry/exit portals near `pos` within `radius` matching `current_interior`.
    pub fn nearby_enex(&self, pos: Vec3, current_interior: i32, radius: f32) -> Vec<&IplEntryExit> {
        let r2 = radius * radius;
        self.entry_exits
            .iter()
            .filter(|e| {
                e.target_interior == current_interior
                    && Vec3::from_array(e.entrance).distance_squared(pos) <= r2
            })
            .collect()
    }

    /// Find the paired exit marker matching `source.name` across interior boundary.
    pub fn find_paired_enex(
        &self,
        source: &IplEntryExit,
        current_interior: i32,
    ) -> Option<&IplEntryExit> {
        self.entry_exits.iter().find(|candidate| {
            !source.name.is_empty()
                && candidate.name.eq_ignore_ascii_case(&source.name)
                && candidate.target_interior != current_interior
        })
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

    /// Compute sky/sun/ambient/fog parameters for `time_hours` in `[0, 24)` and active weather,
    /// using parsed `data/timecyc.dat` entries and `CustomBuildingRenderer` day/night balance when loaded.
    pub fn atmosphere(&self, time_hours: f32, draw_distance: f32) -> AtmosphereState {
        let h = time_hours.rem_euclid(24.0);
        let minutes = h * 60.0;
        let dn_balance = if minutes < 360.0 {
            1.0
        } else if minutes < 420.0 {
            (420.0 - minutes) / 60.0
        } else if minutes < 1200.0 {
            0.0
        } else if minutes < 1260.0 {
            1.0 - (1260.0 - minutes) / 60.0
        } else {
            1.0
        };

        let angle = ((h - 6.0) / 24.0) * std::f32::consts::TAU;
        let sun_z = angle.sin();
        let sun_x = angle.cos();
        let sun_dir = Vec3::new(sun_x * 0.72, 0.38, sun_z.max(-0.22)).normalize();

        let daylight = ((sun_z + 0.15) / 1.15).clamp(0.0, 1.0);
        let twilight = (1.0 - (sun_z.abs() * 3.0).clamp(0.0, 1.0)) * 0.85;

        // Interpolate SA `timecyc.dat` rows (8 time slots per weather block: 0, 5, 6, 7, 12, 19, 20, 22)
        let weather_block = match self.weather {
            WeatherType::ExtraSunnyLA => 0usize,
            WeatherType::SunnyLA => 1,
            WeatherType::ExtraSunnySmog => 2,
            WeatherType::Cloudy => 4,
            WeatherType::Rainy => 5,
            WeatherType::Foggy => 6,
            WeatherType::SandstormDesert => 19,
        };

        let mut sky_top =
            Vec4::new(0.06, 0.09, 0.18, 1.0).lerp(Vec4::new(0.24, 0.52, 0.88, 1.0), daylight);
        let mut sky_bottom =
            Vec4::new(0.12, 0.12, 0.20, 1.0).lerp(Vec4::new(0.68, 0.82, 0.96, 1.0), daylight);
        sky_bottom = sky_bottom.lerp(Vec4::new(0.96, 0.56, 0.24, 1.0), twilight);

        let mut sun_color =
            Vec4::new(0.35, 0.40, 0.60, 0.35).lerp(Vec4::new(1.0, 0.95, 0.85, 1.1), daylight);
        sun_color = sun_color.lerp(Vec4::new(1.0, 0.65, 0.34, 0.95), twilight);

        let mut ambient_color =
            Vec4::new(0.08, 0.08, 0.12, 1.0).lerp(Vec4::new(0.14, 0.12, 0.10, 1.0), daylight);
        let mut ambient_obj_color =
            Vec4::new(0.45, 0.46, 0.52, 1.0).lerp(Vec4::new(0.84, 0.78, 0.72, 1.0), daylight);
        let mut water_color = Vec4::new(0.18, 0.42, 0.52, 0.82);

        let mut fog_start = draw_distance * 0.45;
        let mut fog_end = draw_distance;

        if self.timecyc.entries.len() >= 8 {
            let max_blocks = self.timecyc.entries.len() / 8;
            let block = weather_block.min(max_blocks.saturating_sub(1));
            let base = block * 8;
            const ANCHORS: [f32; 8] = [0.0, 5.0, 6.0, 7.0, 12.0, 19.0, 20.0, 22.0];
            let (i0, i1, t) = if h >= 22.0 {
                (7usize, 0usize, (h - 22.0) / 2.0)
            } else {
                let mut pair = (0, 1, 0.0);
                for k in 0..7 {
                    if h >= ANCHORS[k] && h <= ANCHORS[k + 1] {
                        let span = (ANCHORS[k + 1] - ANCHORS[k]).max(0.001);
                        pair = (k, k + 1, (h - ANCHORS[k]) / span);
                        break;
                    }
                }
                pair
            };
            let a = &self.timecyc.entries[base + i0];
            let b = &self.timecyc.entries[base + i1];
            let rgb_lerp = |ca: [u8; 3], cb: [u8; 3], alpha: f32| -> Vec4 {
                Vec4::new(
                    (f32::from(ca[0]) + (f32::from(cb[0]) - f32::from(ca[0])) * t) / 255.0,
                    (f32::from(ca[1]) + (f32::from(cb[1]) - f32::from(ca[1])) * t) / 255.0,
                    (f32::from(ca[2]) + (f32::from(cb[2]) - f32::from(ca[2])) * t) / 255.0,
                    alpha,
                )
            };
            ambient_color = rgb_lerp(a.ambient_rgb, b.ambient_rgb, 1.0);
            ambient_obj_color = rgb_lerp(a.ambient_obj_rgb, b.ambient_obj_rgb, 1.0);
            sky_top = rgb_lerp(a.sky_top_rgb, b.sky_top_rgb, 1.0);
            sky_bottom = rgb_lerp(a.sky_bottom_rgb, b.sky_bottom_rgb, 1.0);
            let sc = rgb_lerp(a.sun_core_rgb, b.sun_core_rgb, daylight * 0.75 + 0.35);
            sun_color = Vec4::new(sc.x.max(0.4), sc.y.max(0.35), sc.z.max(0.3), sc.w);
            let tc_fog = a.fog_start + (b.fog_start - a.fog_start) * t;
            let tc_far = (a.far_clip + (b.far_clip - a.far_clip) * t).max(300.0);
            fog_start = tc_fog.max(35.0).min(draw_distance * 0.65);
            fog_end = tc_far.max(draw_distance).max(fog_start + 80.0);
            water_color = Vec4::from_array(a.water_rgba).lerp(Vec4::from_array(b.water_rgba), t);
        }

        let mut fog_color = sky_bottom;
        let mut rain_intensity = 0.0;

        match self.weather {
            WeatherType::Rainy => {
                sky_top *= 0.65;
                rain_intensity = 1.0;
                fog_start *= 0.65;
            }
            WeatherType::Foggy => {
                fog_start = 45.0;
                fog_end = (draw_distance * 0.45).max(180.0);
                fog_color = sky_bottom.lerp(Vec4::new(0.65, 0.68, 0.70, 1.0), 0.5);
            }
            WeatherType::SandstormDesert => {
                fog_start = 60.0;
                fog_end = (draw_distance * 0.5).max(240.0);
                fog_color = Vec4::new(0.76, 0.58, 0.36, 1.0);
                sky_bottom = fog_color;
            }
            _ => {}
        }

        AtmosphereState {
            sun_dir,
            sun_color,
            ambient_color,
            ambient_obj_color,
            sky_top,
            sky_bottom,
            fog_color,
            water_color,
            fog_start,
            fog_end,
            rain_intensity,
            dn_balance,
        }
    }
}

impl WorldManager {
    /// Sample exact `water.dat` water surface height at `(pos.x, pos.y)` using 2D triangle barycentrics.
    pub fn water_height_at(&self, pos: Vec3) -> Option<f32> {
        let px = pos.x;
        let py = pos.y;
        let mut best_z: Option<f32> = None;

        let point_in_tri_z = |a: [f32; 3], b: [f32; 3], c: [f32; 3]| -> Option<f32> {
            let v0x = b[0] - a[0];
            let v0y = b[1] - a[1];
            let v1x = c[0] - a[0];
            let v1y = c[1] - a[1];
            let det = v0x * v1y - v1x * v0y;
            if det.abs() < 1e-5 {
                return None;
            }
            let dx = px - a[0];
            let dy = py - a[1];
            let inv_det = 1.0 / det;
            let u = (dx * v1y - v1x * dy) * inv_det;
            let v = (v0x * dy - dx * v0y) * inv_det;
            if u >= -0.001 && v >= -0.001 && (u + v) <= 1.001 {
                let w = 1.0 - u - v;
                Some(w * a[2] + u * b[2] + v * c[2])
            } else {
                None
            }
        };

        for q in &self.water_quads {
            let c = q.corners;
            let min_x = c[0][0].min(c[1][0]).min(c[2][0]).min(c[3][0]);
            let max_x = c[0][0].max(c[1][0]).max(c[2][0]).max(c[3][0]);
            let min_y = c[0][1].min(c[1][1]).min(c[2][1]).min(c[3][1]);
            let max_y = c[0][1].max(c[1][1]).max(c[2][1]).max(c[3][1]);
            if px < min_x || px > max_x || py < min_y || py > max_y {
                continue;
            }
            if let Some(wz) =
                point_in_tri_z(c[0], c[1], c[3]).or_else(|| point_in_tri_z(c[0], c[3], c[2]))
            {
                best_z = Some(best_z.map_or(wz, |prev| prev.max(wz)));
            }
        }
        if best_z.is_none()
            && self.loaded_from_original_data
            && (px.abs() > 3000.0 || py.abs() > 3000.0)
        {
            return Some(0.0);
        }
        best_z
    }
}

impl WorldPlacement {
    /// Returns true if this placement is active at integer hour `hour` (handles `tobj` night lamps like `20..6`).
    pub fn is_active_at_hour(&self, hour: u8) -> bool {
        let (Some(on), Some(off)) = (self.time_on, self.time_off) else {
            return true;
        };
        if on == off {
            return true;
        }
        if on < off {
            hour >= on && hour < off
        } else {
            hour >= on || hour < off
        }
    }
}

fn insert_col_model(map: &mut HashMap<String, ColModel>, model: ColModel) {
    let key = model.name.trim().to_ascii_lowercase();
    if key.is_empty() {
        return;
    }
    let incoming_geom = model.faces.len() + model.boxes.len() + model.spheres.len();
    let replace = match map.get(&key) {
        None => true,
        Some(existing) => {
            incoming_geom > existing.faces.len() + existing.boxes.len() + existing.spheres.len()
        }
    };
    if replace {
        map.insert(key, model);
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

    #[test]
    fn path_graph_picks_neighbors_without_immediate_uturn() {
        let mut graph = PathGraph::default();
        graph.nodes.insert(
            (15, 0),
            WorldPathNode {
                position: Vec3::new(2480.0, -1668.0, 13.0),
                neighbors: vec![(15, 1)],
                is_ped: false,
                is_water: false,
                is_highway: false,
            },
        );
        graph.nodes.insert(
            (15, 1),
            WorldPathNode {
                position: Vec3::new(2500.0, -1668.0, 13.0),
                neighbors: vec![(15, 0), (15, 2)],
                is_ped: false,
                is_water: false,
                is_highway: false,
            },
        );
        graph.nodes.insert(
            (15, 2),
            WorldPathNode {
                position: Vec3::new(2520.0, -1668.0, 13.0),
                neighbors: vec![(15, 1)],
                is_ped: false,
                is_water: false,
                is_highway: false,
            },
        );
        graph.areas.insert(15, vec![(15, 0), (15, 1), (15, 2)]);

        assert_eq!(
            graph.nearest_node(Vec3::new(2498.0, -1668.0, 13.0), false, 30.0),
            Some((15, 1))
        );
        assert_eq!(
            graph.pick_next_node((15, 1), Some((15, 0)), 42, false),
            Some((15, 2))
        );
    }

    #[test]
    fn water_height_at_interpolates_inside_quad_and_none_outside() {
        let mut world = WorldManager::default();
        world
            .water_quads
            .push(gtasarust_gta_formats::dat::WaterQuad {
                corners: [
                    [100.0, 100.0, 2.0],
                    [200.0, 100.0, 2.0],
                    [100.0, 200.0, 4.0],
                    [200.0, 200.0, 4.0],
                ],
                flags: 1,
            });
        let mid = world
            .water_height_at(Vec3::new(150.0, 150.0, 5.0))
            .expect("point inside quad should hit water");
        assert!((mid - 3.0).abs() < 0.01, "expected z=3.0, got {mid}");
        assert!(world.water_height_at(Vec3::new(50.0, 50.0, 0.0)).is_none());
    }
}
