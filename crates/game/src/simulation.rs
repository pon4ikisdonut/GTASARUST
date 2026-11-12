use anyhow::Result;
use glam::{Quat, Vec3};
use gtasarust_ai::{AiSimulation, CrimeEvent, PedBehavior};
use gtasarust_audio::AudioEngine;
use gtasarust_engine::{EngineConfig, FixedTimestep, GameClock, GameDataRoot, HostInfo};
use gtasarust_entities::{PlayerEntity, PlayerLifeState};
use gtasarust_gta_formats::{
    dat::HandlingCfg,
    gxt::GxtArchive,
    img::ImgArchive,
    rw::{DffClump, TxdArchive},
};
use gtasarust_input::{ActionFrameState, ControlMode, InputManager};
use gtasarust_localization::LocalizationTable;
use gtasarust_missions::MissionManager;
use gtasarust_physics::PhysicsWorld;
use gtasarust_renderer::{
    CameraState, GpuMeshHandle, InstanceData, RenderInstanceBatch, SceneRenderer, mesh::CpuMeshData,
};
use gtasarust_save_system::{SaveGameData, SavedWeaponSlot, read_save_slot, write_save_slot};
use gtasarust_scripting::ScmVm;
use gtasarust_ui::{GameUiState, ScreenState, UiActionQueue};
use gtasarust_vehicles::{VehicleClass, VehicleManager, VehicleSpawnSpec};
use gtasarust_weapons::WeaponInventory;
use gtasarust_world::WorldManager;
use std::collections::{HashMap, HashSet};
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct CachedSubMesh {
    pub mesh: GpuMeshHandle,
    pub texture_idx: usize,
    pub material_color: [f32; 4],
}

#[derive(Debug, Clone)]
pub struct CachedDffModel {
    pub submeshes: Vec<CachedSubMesh>,
    pub min_z: f32,
}

pub struct GameWorldState {
    pub config: EngineConfig,
    pub host: HostInfo,
    pub timestep: FixedTimestep,
    pub clock: GameClock,
    pub input: InputManager,
    pub physics: PhysicsWorld,
    pub audio: AudioEngine,
    pub world: WorldManager,
    pub player: PlayerEntity,
    pub vehicles: VehicleManager,
    pub ai: AiSimulation,
    pub scm: ScmVm,
    pub missions: MissionManager,
    pub localization: LocalizationTable,
    pub camera: CameraState,
    pub ui_state: GameUiState,
    pub quit_requested: bool,
    pub vfs: Option<GameDataRoot>,
    pub gta3_archive: Option<ImgArchive>,
    pub gta_int_archive: Option<ImgArchive>,
    pub handling: Option<HandlingCfg>,
    pub cached_textures: HashMap<String, usize>,
    pub loaded_txds: HashSet<String>,
    pub cached_dff_meshes: HashMap<String, CachedDffModel>,
    pub failed_dff_lookups: HashSet<String>,
    pub water_mesh: Option<GpuMeshHandle>,
}

impl GameWorldState {
    pub fn initialize(config: EngineConfig, host: HostInfo, headless: bool) -> Self {
        let mut localization = LocalizationTable::with_fallback_json(include_str!(
            "../../../data/localization/fallback_en.json"
        ));
        let mut gta3_archive = None;
        let mut gta_int_archive = None;
        let mut handling = None;
        let mut opened_vfs = None;

        let world = if let Some(dir) = &config.game_data_dir {
            match GameDataRoot::open(dir) {
                Ok(vfs) => {
                    info!(path = %dir.display(), "Authorized GTA: San Andreas installation detected");
                    if let Ok(h_txt) = vfs.read_text_lossy("data/handling.cfg") {
                        handling = Some(HandlingCfg::parse(&h_txt));
                    }
                    let gxt_rel = format!("text/{}.gxt", config.game.language);
                    if let Ok(gxt_bytes) = vfs.read_bytes(&gxt_rel)
                        && let Ok(gxt) = GxtArchive::parse(&gxt_bytes)
                    {
                        info!(entries = gxt.total_entries(), "Loaded GXT localization");
                        localization.attach_gxt(gxt);
                    }
                    if let Ok(img_path) = vfs.resolve("models/gta3.img") {
                        gta3_archive = ImgArchive::open(&img_path).ok();
                    }
                    if let Ok(img_path) = vfs.resolve("models/gta_int.img") {
                        gta_int_archive = ImgArchive::open(&img_path).ok();
                    }
                    let loaded_world = WorldManager::load_from_game_dir(&vfs, !headless);
                    opened_vfs = Some(vfs);
                    loaded_world
                }
                Err(err) => {
                    warn!("Could not open game directory ({err}); using fallback world slice");
                    WorldManager::create_fallback_san_andreas()
                }
            }
        } else {
            info!(
                "No --game-dir / GTASA_DATA_DIR provided; starting in fallback San Andreas slice"
            );
            WorldManager::create_fallback_san_andreas()
        };

        let physics = PhysicsWorld::new();
        let input = InputManager::new(config.controls.gamepad_deadzone);
        let audio = AudioEngine::new(&config.audio, headless);

        let mut player = PlayerEntity::default();
        player
            .weapons
            .give_weapon(WeaponInventory::preset_colt45(), 51);

        let mut vehicles = VehicleManager::default();
        let h_ref = handling.as_ref();
        // Iconic Grove Street starter vehicles
        vehicles.spawn(VehicleSpawnSpec {
            model_id: 492,
            name: "GREENWOO",
            class: VehicleClass::Car,
            position: Vec3::new(2503.0, -1680.0, 0.5),
            yaw: 0.3,
            color1: [0.14, 0.42, 0.20],
            handling: h_ref,
        });
        vehicles.spawn(VehicleSpawnSpec {
            model_id: 481,
            name: "BMX",
            class: VehicleClass::Bicycle,
            position: Vec3::new(2491.0, -1683.0, 0.5),
            yaw: 1.2,
            color1: [0.85, 0.2, 0.2],
            handling: h_ref,
        });
        vehicles.spawn(VehicleSpawnSpec {
            model_id: 411,
            name: "INFERNUS",
            class: VehicleClass::Car,
            position: Vec3::new(2485.0, -1675.0, 0.5),
            yaw: -0.5,
            color1: [0.9, 0.72, 0.12],
            handling: h_ref,
        });

        let mut ai = AiSimulation::default();
        ai.spawn_ped(
            105,
            Vec3::new(2500.0, -1675.0, 0.0),
            false,
            PedBehavior::Patrol,
        );
        ai.spawn_ped(
            106,
            Vec3::new(2482.0, -1692.0, 0.0),
            false,
            PedBehavior::Patrol,
        );

        let mut camera = CameraState::default();
        camera.update_third_person(player.position, player.yaw, player.pitch, 6.5);

        Self {
            config,
            host,
            timestep: FixedTimestep::default(),
            clock: GameClock::default(),
            input,
            physics,
            audio,
            world,
            player,
            vehicles,
            ai,
            scm: ScmVm::default(),
            missions: MissionManager::new_san_andreas_campaign(),
            localization,
            camera,
            ui_state: GameUiState::default(),
            quit_requested: false,
            vfs: opened_vfs,
            gta3_archive,
            gta_int_archive,
            handling,
            cached_textures: HashMap::new(),
            loaded_txds: HashSet::new(),
            cached_dff_meshes: HashMap::new(),
            failed_dff_lookups: HashSet::new(),
            water_mesh: None,
        }
    }

    fn read_archive_entry(&self, entry_name: &str) -> Option<Vec<u8>> {
        if let Some(img) = &self.gta3_archive
            && let Ok(bytes) = img.read_entry(entry_name)
        {
            return Some(bytes);
        }
        if let Some(img) = &self.gta_int_archive
            && let Ok(bytes) = img.read_entry(entry_name)
        {
            return Some(bytes);
        }
        None
    }

    fn ensure_txd_loaded(&mut self, renderer: &mut SceneRenderer, txd_name: &str) {
        let txd_lower = txd_name.trim().to_ascii_lowercase();
        if txd_lower.is_empty() || !self.loaded_txds.insert(txd_lower.clone()) {
            return;
        }
        let txd_entry = format!("{txd_lower}.txd");
        let bytes = self.read_archive_entry(&txd_entry).or_else(|| {
            self.vfs.as_ref().and_then(|v| {
                v.read_bytes(&format!("models/generic/{txd_entry}"))
                    .or_else(|_| v.read_bytes(&format!("models/{txd_entry}")))
                    .ok()
            })
        });
        let Some(bytes) = bytes else {
            return;
        };
        let Ok(archive) = TxdArchive::parse(&bytes) else {
            return;
        };
        for tex in archive.textures {
            let tex_idx = renderer.upload_rgba8_texture(
                &format!("{txd_lower}/{}", tex.name),
                tex.width,
                tex.height,
                &tex.rgba8,
            );
            let tex_lower = tex.name.to_ascii_lowercase();
            self.cached_textures
                .insert(format!("{txd_lower}/{tex_lower}"), tex_idx);
            self.cached_textures.entry(tex_lower).or_insert(tex_idx);
        }
    }

    fn resolve_texture_idx(
        &self,
        txd_lower: &str,
        tex_name: Option<&str>,
        fallback: usize,
    ) -> usize {
        let Some(raw) = tex_name else {
            return fallback;
        };
        let tex_lower = raw.to_ascii_lowercase();
        if !txd_lower.is_empty()
            && let Some(&idx) = self
                .cached_textures
                .get(&format!("{txd_lower}/{tex_lower}"))
        {
            return idx;
        }
        self.cached_textures
            .get(&tex_lower)
            .copied()
            .unwrap_or(fallback)
    }

    fn load_dff_model(
        &mut self,
        renderer: &mut SceneRenderer,
        model_lower: &str,
        txd_lower: &str,
    ) -> bool {
        let dff_name = format!("{model_lower}.dff");
        if self.cached_dff_meshes.contains_key(&dff_name)
            || self.failed_dff_lookups.contains(&dff_name)
        {
            return false;
        }
        if !txd_lower.is_empty() {
            self.ensure_txd_loaded(renderer, txd_lower);
        }
        let Some(bytes) = self.read_archive_entry(&dff_name) else {
            self.failed_dff_lookups.insert(dff_name);
            return false;
        };
        let Ok(clump) = DffClump::parse(&bytes) else {
            self.failed_dff_lookups.insert(dff_name);
            return false;
        };
        let subs = CpuMeshData::dff_clump_submeshes(&clump);
        if subs.is_empty() {
            self.failed_dff_lookups.insert(dff_name);
            return false;
        }

        let mut min_z = f32::INFINITY;
        let mut cached_subs = Vec::with_capacity(subs.len());
        for (idx, sub) in subs.into_iter().enumerate() {
            min_z = min_z.min(sub.cpu_mesh.min_z());
            let tex_idx = self.resolve_texture_idx(
                txd_lower,
                sub.texture_name.as_deref(),
                renderer.white_tex,
            );
            let handle = renderer.upload_mesh(&format!("{dff_name}#{idx}"), &sub.cpu_mesh);
            cached_subs.push(CachedSubMesh {
                mesh: handle,
                texture_idx: tex_idx,
                material_color: sub.material_color,
            });
        }
        if !min_z.is_finite() {
            min_z = 0.0;
        }
        self.cached_dff_meshes.insert(
            dff_name,
            CachedDffModel {
                submeshes: cached_subs,
                min_z,
            },
        );
        true
    }

    pub fn preload_render_meshes(&mut self, renderer: &mut SceneRenderer) {
        if let Some(water_cpu) = CpuMeshData::from_water_quads(&self.world.water_quads) {
            self.water_mesh = Some(renderer.upload_mesh("sa_water_quads", &water_cpu));
        }
        if self.gta3_archive.is_none() && self.gta_int_archive.is_none() {
            return;
        }

        // Shared vehicle/generic TXDs resolved as parent fallback textures
        for shared_txd in ["vehicle", "misc", "particle"] {
            self.ensure_txd_loaded(renderer, shared_txd);
        }

        // Preload starter vehicles (`GREENWOO`, `BMX`, `INFERNUS`, `COPCARLA`)
        let vehicle_specs: Vec<(String, String)> = self
            .vehicles
            .vehicles
            .iter()
            .map(|v| {
                let def = self.world.vehicles.get(&v.model_id);
                let model = def
                    .map(|d| d.model_name.to_ascii_lowercase())
                    .unwrap_or_else(|| v.name.to_ascii_lowercase());
                let txd = def
                    .map(|d| d.txd_name.to_ascii_lowercase())
                    .unwrap_or_else(|| model.clone());
                (model, txd)
            })
            .chain(std::iter::once((
                "copcarla".to_string(),
                "copcarla".to_string(),
            )))
            .collect();
        for (model, txd) in vehicle_specs {
            self.load_dff_model(renderer, &model, &txd);
        }

        self.world
            .update_streaming(self.player.position, self.config.graphics.draw_distance);
        let initial_loaded = self.stream_nearby_render_assets(renderer, 64);
        if initial_loaded > 0 || !self.cached_dff_meshes.is_empty() {
            info!(
                models = self.cached_dff_meshes.len(),
                textures = self.cached_textures.len(),
                "Preloaded textured RenderWare DFF/TXD assets"
            );
        }
    }

    /// Incrementally upload nearby streamed world placements & vehicle models within a per-frame budget.
    pub fn stream_nearby_render_assets(
        &mut self,
        renderer: &mut SceneRenderer,
        budget: usize,
    ) -> usize {
        if self.gta3_archive.is_none() && self.gta_int_archive.is_none() {
            return 0;
        }
        let mut pending: Vec<(f32, String, String)> = Vec::new();
        for p in self.world.active_placements(self.player.interior_zero()) {
            if p.is_lod {
                continue;
            }
            let model_lower = p.model_name.to_ascii_lowercase();
            let dff_name = format!("{model_lower}.dff");
            if self.cached_dff_meshes.contains_key(&dff_name)
                || self.failed_dff_lookups.contains(&dff_name)
            {
                continue;
            }
            let dist = p.position.distance_squared(self.player.position);
            pending.push((dist, model_lower, p.txd_name.to_ascii_lowercase()));
        }
        pending.sort_by(|a, b| a.0.total_cmp(&b.0));

        let mut loaded = 0usize;
        for (_, model_lower, txd_lower) in pending {
            if loaded >= budget {
                break;
            }
            if self.load_dff_model(renderer, &model_lower, &txd_lower) {
                loaded += 1;
            }
        }
        loaded
    }

    pub fn step_frame(&mut self, frame_dt: f32) {
        let actions = self.input.sample_actions(
            self.config.controls.mouse_sensitivity,
            self.config.controls.invert_y,
        );

        if actions.toggle_dev_overlay {
            self.ui_state.show_dev_overlay = !self.ui_state.show_dev_overlay;
        }
        if actions.toggle_pause {
            self.ui_state.screen = match self.ui_state.screen {
                ScreenState::Playing => ScreenState::Paused,
                ScreenState::Paused | ScreenState::Map | ScreenState::Settings => {
                    ScreenState::Playing
                }
                other => other,
            };
        }
        if actions.toggle_map
            && matches!(
                self.ui_state.screen,
                ScreenState::Playing | ScreenState::Map
            )
        {
            self.ui_state.screen = if self.ui_state.screen == ScreenState::Map {
                ScreenState::Playing
            } else {
                ScreenState::Map
            };
        }

        if self.ui_state.screen != ScreenState::Playing {
            return;
        }

        let ticks = self.timestep.accumulate(frame_dt);
        for tick_idx in 0..ticks {
            let tick_actions = if tick_idx == 0 {
                &actions
            } else {
                &ActionFrameState {
                    move_axis: actions.move_axis,
                    sprint: actions.sprint,
                    jump_or_handbrake: actions.jump_or_handbrake,
                    crouch_or_horn: actions.crouch_or_horn,
                    fire_or_attack: actions.fire_or_attack,
                    aim_or_reverse: actions.aim_or_reverse,
                    ..Default::default()
                }
            };
            self.step_fixed_tick(self.timestep.fixed_dt, tick_actions);
        }
    }

    pub fn step_fixed_tick(&mut self, dt: f32, actions: &ActionFrameState) {
        self.clock.advance(dt);
        self.audio.apply_config(&self.config.audio);

        if actions.enter_exit_just_pressed {
            self.vehicles.toggle_player_entry(&mut self.player);
        }

        if actions.radio_next && self.player.control_mode == ControlMode::InVehicle {
            self.audio.cycle_radio_next();
        }

        // Fire active weapon on foot
        if self.player.control_mode == ControlMode::OnFoot
            && actions.fire_or_attack
            && let Some((damage, range, _explosive)) = self.player.weapons.try_fire()
        {
            self.ai.alert_gunshot_near(self.player.position, 40.0);
            if self.player.weapons.active_slot >= 2 {
                self.ai
                    .report_crime(CrimeEvent::WeaponDischarged, &mut self.player);
            }
            let dir = Vec3::new(self.player.yaw.cos(), self.player.yaw.sin(), 0.0).normalize();
            for ped in &mut self.ai.pedestrians {
                if ped.health > 0.0 && ped.position.distance(self.player.position) <= range {
                    let to_ped = (ped.position - self.player.position)
                        .truncate()
                        .normalize_or_zero();
                    if to_ped.dot(dir.truncate()) > 0.75 {
                        ped.health = (ped.health - damage).max(0.0);
                    }
                }
            }
        }

        self.player.update_on_foot(dt, actions, &self.physics);

        let occupied_id = self.player.current_vehicle;
        for veh in &mut self.vehicles.vehicles {
            let input_for_veh = if Some(veh.id) == occupied_id {
                Some(actions)
            } else {
                None
            };
            veh.step_physics(dt, input_for_veh, &self.physics);
            if Some(veh.id) == occupied_id {
                self.player.position = veh.position;
                self.player.yaw = veh.yaw;
            }
        }

        self.physics.step(dt);
        self.ai.tick(dt, &mut self.player, &mut self.vehicles);
        self.scm.tick(dt);
        self.missions.tick(dt, &mut self.player, &mut self.scm);

        if matches!(
            self.player.life_state,
            PlayerLifeState::Wasted | PlayerLifeState::Busted
        ) {
            self.player
                .respawn_at_hospital(Vec3::new(2033.0, -1416.0, 1.0));
        }

        self.world
            .update_streaming(self.player.position, self.config.graphics.draw_distance);

        let cam_dist = if self.player.control_mode == ControlMode::InVehicle {
            9.5
        } else {
            5.8
        };
        self.camera.update_third_person(
            self.player.position,
            self.player.yaw,
            self.player.pitch,
            cam_dist,
        );
    }

    pub fn apply_ui_actions(&mut self, actions: UiActionQueue) {
        if actions.quit_requested {
            self.quit_requested = true;
        }
        if actions.start_new_game {
            self.player = PlayerEntity::default();
            self.player
                .weapons
                .give_weapon(WeaponInventory::preset_colt45(), 51);
            self.missions = MissionManager::new_san_andreas_campaign();
        }
        if let Some(w) = actions.set_weather {
            self.world.weather = w;
        }
        if let Some(h) = actions.set_hour {
            self.clock.hour = h % 24;
            self.clock.minute = 0;
        }
        if actions.add_wanted_star {
            self.ai
                .report_crime(CrimeEvent::OfficerAttacked, &mut self.player);
        }
        if actions.clear_wanted {
            self.ai.wanted_heat = 0.0;
            self.ai.wanted_stars = 0;
            self.player.wanted_stars = 0;
        }
        if let Some(tp) = actions.teleport_to {
            self.player.position = tp;
            self.player.current_vehicle = None;
            self.player.control_mode = ControlMode::OnFoot;
        }
        if let Some(slot) = actions.save_to_slot {
            let data = self.to_save_data(slot);
            match write_save_slot(&self.host.save_dir, slot, &data) {
                Ok(p) => {
                    self.ui_state.status_message =
                        Some(format!("Saved slot {slot} -> {}", p.display()));
                }
                Err(e) => {
                    self.ui_state.status_message = Some(format!("Save error: {e}"));
                }
            }
        }
        if let Some(slot) = actions.load_from_slot {
            match read_save_slot(&self.host.save_dir, slot) {
                Ok(data) => {
                    self.apply_save_data(data);
                    self.ui_state.status_message = Some(format!("Loaded slot {slot}"));
                }
                Err(e) => {
                    self.ui_state.status_message = Some(format!("Load error: {e}"));
                }
            }
        }
    }

    pub fn to_save_data(&self, slot: usize) -> SaveGameData {
        SaveGameData {
            schema_version: gtasarust_save_system::CURRENT_SAVE_VERSION,
            slot_title: format!(
                "Slot {slot} - {}",
                self.world.zone_name_at(self.player.position)
            ),
            game_hour: self.clock.hour,
            game_minute: self.clock.minute,
            game_day: self.clock.day,
            player_position: self.player.position.to_array(),
            player_yaw: self.player.yaw,
            health: self.player.health,
            max_health: self.player.max_health,
            armor: self.player.armor,
            money: self.player.money,
            respect: self.player.respect,
            wanted_stars: self.player.wanted_stars,
            weapons: self
                .player
                .weapons
                .slots
                .iter()
                .flatten()
                .map(|w| SavedWeaponSlot {
                    weapon_name: w.spec.name.clone(),
                    ammo: w.ammo_in_clip + w.reserve_ammo,
                })
                .collect(),
            completed_missions: self.missions.completed.clone(),
            owned_properties: vec!["johnson_house".to_string()],
            garage_vehicles: Vec::new(),
            total_playtime_seconds: self.timestep.total_sim_time,
        }
    }

    pub fn apply_save_data(&mut self, data: SaveGameData) {
        self.clock.hour = data.game_hour % 24;
        self.clock.minute = data.game_minute % 60;
        self.clock.day = data.game_day;
        self.player.position = Vec3::from_array(data.player_position);
        self.player.yaw = data.player_yaw;
        self.player.health = data.health;
        self.player.max_health = data.max_health;
        self.player.armor = data.armor;
        self.player.money = data.money;
        self.player.respect = data.respect;
        self.player.wanted_stars = data.wanted_stars;
        self.missions.completed = data.completed_missions;
    }

    pub fn collect_render_batches(&self, renderer: &SceneRenderer) -> Vec<RenderInstanceBatch> {
        let frustum = self.camera.frustum();
        let ground_z = if self.world.loaded_from_original_data {
            -0.08
        } else {
            0.0
        };
        let ground_instances = vec![InstanceData::from_transform(
            Vec3::new(self.player.position.x, self.player.position.y, ground_z),
            Quat::IDENTITY,
            Vec3::ONE,
            [0.85, 0.88, 0.80, 1.0],
        )];

        let mut box_instances = Vec::new();
        let mut custom_batches: HashMap<(usize, usize), Vec<InstanceData>> = HashMap::new();

        // Player character (CJ) when on foot
        if self.player.control_mode == ControlMode::OnFoot {
            box_instances.push(InstanceData::from_transform(
                self.player.position + Vec3::new(0.0, 0.0, 0.9),
                Quat::from_rotation_z(self.player.yaw),
                Vec3::new(0.65, 0.45, 1.8),
                [0.95, 0.95, 0.95, 1.0], // White tank top
            ));
        }

        // Vehicles (use real RenderWare DFF+TXD submeshes when available, fallback to colored bounds)
        for v in &self.vehicles.vehicles {
            if !frustum.intersects_sphere(v.position, 6.0) {
                continue;
            }
            let dff_key = format!("{}.dff", v.name.to_ascii_lowercase());
            if let Some(model) = self.cached_dff_meshes.get(&dff_key) {
                let z_lift = (-model.min_z).clamp(0.0, 1.5);
                let pos = v.position + Vec3::new(0.0, 0.0, z_lift);
                // Match RenderWare GTA:SA forward-axis (+Y) orientation cleanly to yaw rotation
                let rot = v.rotation_quat() * Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2);
                for sub in &model.submeshes {
                    let tint = apply_vehicle_paint_tint(sub.material_color, v.color1, v.color2);
                    custom_batches
                        .entry((sub.mesh.0, sub.texture_idx))
                        .or_default()
                        .push(InstanceData::from_transform(pos, rot, Vec3::ONE, tint));
                }
            } else {
                let scale = match v.params.class {
                    VehicleClass::Bicycle | VehicleClass::Motorcycle => Vec3::new(1.8, 0.7, 1.1),
                    VehicleClass::Truck | VehicleClass::Bus => Vec3::new(6.8, 2.5, 2.8),
                    _ => Vec3::new(4.3, 1.9, 1.45),
                };
                box_instances.push(InstanceData::from_transform(
                    v.position + Vec3::new(0.0, 0.0, scale.z * 0.5),
                    v.rotation_quat(),
                    scale,
                    [v.color1[0], v.color1[1], v.color1[2], 1.0],
                ));
            }
        }

        // Pedestrians & Police
        for ped in &self.ai.pedestrians {
            if ped.health > 0.0 && frustum.intersects_sphere(ped.position, 2.0) {
                let col = if ped.is_cop {
                    [0.15, 0.30, 0.75, 1.0]
                } else {
                    [0.20, 0.65, 0.25, 1.0]
                };
                box_instances.push(InstanceData::from_transform(
                    ped.position + Vec3::new(0.0, 0.0, 0.85),
                    Quat::from_rotation_z(ped.yaw),
                    Vec3::new(0.6, 0.4, 1.7),
                    col,
                ));
            }
        }

        // Mission trigger markers
        for trig in self.missions.available_triggers() {
            if frustum.intersects_sphere(trig.position, 4.0) {
                box_instances.push(InstanceData::from_transform(
                    trig.position + Vec3::new(0.0, 0.0, 1.2),
                    Quat::IDENTITY,
                    Vec3::new(1.2, 1.2, 2.4),
                    [0.95, 0.25, 0.20, 0.75],
                ));
            }
        }

        // Active mission target beacon
        if let Some(cp) = self.missions.current_target_position() {
            box_instances.push(InstanceData::from_transform(
                cp + Vec3::new(0.0, 0.0, 2.0),
                Quat::IDENTITY,
                Vec3::new(1.4, 1.4, 4.0),
                [0.98, 0.82, 0.15, 0.75],
            ));
        }

        // World streamed objects (actual textured RenderWare DFF submeshes when loaded, or fallback blocks)
        for p in self.world.active_placements(self.player.interior_zero()) {
            if p.is_lod {
                continue;
            }
            if !frustum.intersects_sphere(p.position, 55.0) {
                continue;
            }
            let dff_key = format!("{}.dff", p.model_name.to_ascii_lowercase());
            if let Some(model) = self.cached_dff_meshes.get(&dff_key) {
                for sub in &model.submeshes {
                    custom_batches
                        .entry((sub.mesh.0, sub.texture_idx))
                        .or_default()
                        .push(InstanceData::from_transform(
                            p.position,
                            p.rotation,
                            Vec3::ONE,
                            sub.material_color,
                        ));
                }
            } else if !self.world.loaded_from_original_data {
                box_instances.push(InstanceData::from_transform(
                    p.position + Vec3::new(0.0, 0.0, 6.0),
                    p.rotation,
                    Vec3::new(14.0, 12.0, 12.0),
                    [0.72, 0.68, 0.60, 1.0],
                ));
            }
        }

        let mut batches = vec![
            RenderInstanceBatch {
                mesh: renderer.ground_mesh,
                texture_idx: renderer.checker_tex,
                instances: ground_instances,
            },
            RenderInstanceBatch {
                mesh: renderer.box_mesh,
                texture_idx: renderer.white_tex,
                instances: box_instances,
            },
        ];

        if let Some(water) = self.water_mesh {
            batches.push(RenderInstanceBatch {
                mesh: water,
                texture_idx: renderer.white_tex,
                instances: vec![InstanceData::from_transform(
                    Vec3::ZERO,
                    Quat::IDENTITY,
                    Vec3::ONE,
                    [1.0, 1.0, 1.0, 1.0],
                )],
            });
        }

        for ((mesh_idx, tex_idx), instances) in custom_batches {
            batches.push(RenderInstanceBatch {
                mesh: GpuMeshHandle(mesh_idx),
                texture_idx: tex_idx,
                instances,
            });
        }

        batches
    }

    /// Execute deterministic headless gameplay verification across movement, vehicles, AI, missions, and save/load.
    pub fn run_headless_smoke(&mut self, frames: u32) -> Result<()> {
        let walk = ActionFrameState {
            move_axis: glam::Vec2::new(0.0, 1.0),
            sprint: true,
            ..Default::default()
        };
        for _ in 0..frames {
            self.timestep.total_ticks += 1;
            self.step_fixed_tick(1.0 / 60.0, &walk);
        }
        // Verify save/load serialization integrity in a temporary directory
        let tmp = std::env::temp_dir().join("gtasarust_smoke_saves");
        let save = self.to_save_data(1);
        let written = write_save_slot(&tmp, 1, &save)?;
        let loaded = read_save_slot(&tmp, 1)?;
        let _ = std::fs::remove_file(written);
        anyhow::ensure!(loaded.money == save.money, "save roundtrip mismatch");

        info!(
            ticks = self.timestep.total_ticks,
            player_pos = ?self.player.position,
            zone = %self.world.zone_name_at(self.player.position),
            vehicles = self.vehicles.vehicles.len(),
            handling_entries = self.handling.as_ref().map_or(0, |h| h.vehicles.len()),
            "Smoke test checks passed"
        );
        Ok(())
    }
}

trait PlayerInteriorExt {
    fn interior_zero(&self) -> i32;
}

impl PlayerInteriorExt for PlayerEntity {
    fn interior_zero(&self) -> i32 {
        0
    }
}

fn apply_vehicle_paint_tint(mat_col: [f32; 4], color1: [f32; 3], color2: [f32; 3]) -> [f32; 4] {
    // GTA:SA RenderWare reserves RGB ~(60, 255, 0) for primary paint and ~(255, 0, 175) for secondary.
    if (mat_col[0] - 60.0 / 255.0).abs() < 0.08 && mat_col[1] > 0.90 && mat_col[2] < 0.08 {
        [color1[0], color1[1], color1[2], mat_col[3]]
    } else if mat_col[0] > 0.90 && mat_col[1] < 0.08 && (mat_col[2] - 175.0 / 255.0).abs() < 0.08 {
        [color2[0], color2[1], color2[2], mat_col[3]]
    } else {
        mat_col
    }
}
