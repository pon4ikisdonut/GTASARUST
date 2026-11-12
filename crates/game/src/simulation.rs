use anyhow::Result;
use glam::{Quat, Vec3};
use gtasarust_ai::{AiSimulation, CrimeEvent, PedBehavior};
use gtasarust_animation::{AnimState, AnimationController, IfpPoseLibrary, compute_skin_matrices};
use gtasarust_audio::{AudioEngine, SfxCue};
use gtasarust_engine::{EngineConfig, FixedTimestep, GameClock, GameDataRoot, HostInfo};
use gtasarust_entities::{PlayerEntity, PlayerLifeState};
use gtasarust_gta_formats::{
    dat::HandlingCfg,
    gxt::GxtArchive,
    ide::{
        IDE_FLAG_ADDITIVE, IDE_FLAG_DISABLE_BACKFACE_CULLING, IDE_FLAG_DRAW_LAST,
        IDE_FLAG_IS_GLASS_TYPE_1, IDE_FLAG_IS_GLASS_TYPE_2, IDE_FLAG_IS_PALM, IDE_FLAG_IS_TREE,
        IDE_FLAG_NO_ZWRITE,
    },
    ifp::IfpArchive,
    img::ImgArchive,
    rw::{DffClump, DffFrame, DffSkin, TxdArchive, merge_rgba_alpha_mask},
    scm::ScmImage,
};
use gtasarust_input::{ActionFrameState, ControlMode, InputManager};
use gtasarust_localization::LocalizationTable;
use gtasarust_missions::MissionManager;
use gtasarust_physics::{ColliderHandle, PhysicsWorld};
use gtasarust_renderer::{
    CameraState, DffSubMesh, GpuMeshHandle, InstanceData, RenderInstanceBatch, SceneRenderer,
    VehicleWheelMount, Vertex3D, mesh::CpuMeshData,
};
use gtasarust_save_system::{SaveGameData, SavedWeaponSlot, read_save_slot, write_save_slot};
use gtasarust_scripting::{ScmVm, ScmWorldCommand};
use gtasarust_ui::{GameUiState, ScreenState, UiActionQueue};
use gtasarust_vehicles::{VehicleClass, VehicleManager, VehicleSpawnSpec};
use gtasarust_weapons::{WeaponInventory, WeaponKind};
use gtasarust_world::{SectorCoord, WeatherType, WorldManager};
use std::collections::{HashMap, HashSet};
use tracing::{debug, info, warn};

#[derive(Debug, Clone)]
pub struct CachedSubMesh {
    pub mesh: GpuMeshHandle,
    pub texture_idx: usize,
    pub material_color: [f32; 4],
    pub has_alpha_tex: bool,
}

#[derive(Debug, Clone)]
pub struct CachedDffModel {
    pub submeshes: Vec<CachedSubMesh>,
    pub bounding_radius: f32,
}

#[derive(Debug, Clone)]
pub struct CachedVehicleModel {
    pub body: Vec<CachedSubMesh>,
    pub wheel: Vec<CachedSubMesh>,
    pub wheel_mounts: Vec<VehicleWheelMount>,
    pub min_z: f32,
}

pub struct SkinnedSubMeshTemplate {
    pub sub: DffSubMesh,
    pub texture_idx: usize,
    pub material_color: [f32; 4],
}

pub struct CachedSkinnedPedRig {
    pub frames: Vec<DffFrame>,
    pub skin: Option<DffSkin>,
    pub submeshes: Vec<SkinnedSubMeshTemplate>,
}

pub struct CachedCjPart {
    pub rig: CachedSkinnedPedRig,
    pub mesh_handles: Vec<(GpuMeshHandle, usize, [f32; 4])>,
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
    pub player_archive: Option<ImgArchive>,
    pub handling: Option<HandlingCfg>,
    pub ped_ifp_lib: IfpPoseLibrary,
    pub cached_textures: HashMap<String, usize>,
    pub txd_first_tex: HashMap<String, usize>,
    pub alpha_textures: HashSet<usize>,
    pub loaded_txds: HashSet<String>,
    pub cached_dff_meshes: HashMap<String, CachedDffModel>,
    pub cached_vehicle_models: HashMap<String, CachedVehicleModel>,
    pub ped_rigs: HashMap<String, CachedSkinnedPedRig>,
    pub cj_parts: Vec<CachedCjPart>,
    pub cj_skinned_meshes: Vec<(GpuMeshHandle, usize, [f32; 4])>,
    pub ped_skinned_meshes: HashMap<u64, (String, Vec<GpuMeshHandle>)>,
    skin_vertex_scratch: Vec<Vertex3D>,
    pub failed_dff_lookups: HashSet<String>,
    pub sector_colliders: HashMap<SectorCoord, Vec<ColliderHandle>>,
    pub loaded_collider_interior: i32,
    pub water_mesh: Option<GpuMeshHandle>,
    pub marker_mesh: Option<GpuMeshHandle>,
    pub scm_pickups: Vec<(Vec3, i32)>,
    pub scm_blips: Vec<(Vec3, u8)>,
    pub enex_cooldown: f32,
    pub enex_lock_pos: Option<Vec3>,
    pub interior_floor_z: Option<f32>,
    pub active_enex_name: Option<String>,
    pub interior_service_pos: Option<Vec3>,
    pub service_cooldown: f32,
    pub status_message_timer: f32,
}

impl GameWorldState {
    pub fn initialize(config: EngineConfig, host: HostInfo, headless: bool) -> Self {
        let mut localization = LocalizationTable::with_fallback_json(include_str!(
            "../../../data/localization/fallback_en.json"
        ));
        let mut gta3_archive = None;
        let mut gta_int_archive = None;
        let mut player_archive = None;
        let mut handling = None;
        let mut opened_vfs = None;
        let mut ped_ifp_lib = IfpPoseLibrary::default();
        let mut scm = ScmVm::default();
        let mut ui_state = GameUiState {
            screen: if headless {
                ScreenState::Playing
            } else {
                ScreenState::MainMenu
            },
            ..Default::default()
        };

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
                    if let Ok(ifp_bytes) = vfs.read_bytes("anim/ped.ifp")
                        && let Ok(archive) = IfpArchive::parse_full(&ifp_bytes)
                    {
                        let clips = ped_ifp_lib.insert_archive(archive);
                        info!(clips, "Loaded ped.ifp skeletal animation clips");
                    }
                    if let Ok(scm_bytes) = vfs.read_bytes("data/script/main.scm")
                        && let Ok(scm_img) = ScmImage::parse(scm_bytes)
                    {
                        info!(
                            main_offset = scm_img.main_offset,
                            globals_bytes = scm_img.globals_end,
                            missions = scm_img.mission_offsets.len(),
                            "Loaded retail GTA:SA main.scm bytecode image"
                        );
                        scm.load_scm_image(scm_img);
                    }
                    if let Ok(img_path) = vfs.resolve("models/gta3.img") {
                        gta3_archive = ImgArchive::open(&img_path).ok();
                    }
                    if let Ok(img_path) = vfs.resolve("models/gta_int.img") {
                        gta_int_archive = ImgArchive::open(&img_path).ok();
                    }
                    if let Ok(img_path) = vfs.resolve("models/player.img") {
                        player_archive = ImgArchive::open(&img_path).ok();
                    }
                    if !headless {
                        load_ui_media(&vfs, gta3_archive.as_ref(), &mut ui_state);
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
        let mut audio = AudioEngine::new(&config.audio, headless);
        if let Some(vfs) = &opened_vfs {
            audio.attach_game_dir(vfs.root_path());
        }

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

        let mut state = Self {
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
            scm,
            missions: MissionManager::new_san_andreas_campaign(),
            localization,
            camera,
            ui_state,
            quit_requested: false,
            vfs: opened_vfs,
            gta3_archive,
            gta_int_archive,
            player_archive,
            handling,
            ped_ifp_lib,
            cached_textures: HashMap::new(),
            txd_first_tex: HashMap::new(),
            alpha_textures: HashSet::new(),
            loaded_txds: HashSet::new(),
            cached_dff_meshes: HashMap::new(),
            cached_vehicle_models: HashMap::new(),
            ped_rigs: HashMap::new(),
            cj_parts: Vec::new(),
            cj_skinned_meshes: Vec::new(),
            ped_skinned_meshes: HashMap::new(),
            skin_vertex_scratch: Vec::new(),
            failed_dff_lookups: HashSet::new(),
            sector_colliders: HashMap::new(),
            loaded_collider_interior: 0,
            water_mesh: None,
            marker_mesh: None,
            scm_pickups: Vec::new(),
            scm_blips: Vec::new(),
            enex_cooldown: 0.0,
            enex_lock_pos: None,
            interior_floor_z: None,
            active_enex_name: None,
            interior_service_pos: None,
            service_cooldown: 0.0,
            status_message_timer: 0.0,
        };
        state.drain_scm_world_commands();
        if state.world.loaded_from_original_data {
            state.spawn_nearby_ipl_vehicles(260.0, 20);
        }
        state
            .world
            .update_streaming(state.player.position, state.config.graphics.draw_distance);
        state.sync_collision_sectors();
        state.snap_initial_spawns_to_ground();
        state.update_player_camera();
        state
    }

    fn update_player_camera(&mut self) {
        let dist = if self.player.control_mode == ControlMode::InVehicle {
            (self.player.camera_distance + 2.5).clamp(4.0, 14.0)
        } else {
            self.player.camera_distance.clamp(2.2, 12.0)
        };
        let physics = &self.physics;
        let ray_probe = |origin: Vec3, dir: Vec3, max_d: f32| {
            physics.raycast(origin, dir, max_d).map(|(toi, _)| toi)
        };
        self.camera.update_third_person_collided(
            self.player.position,
            self.player.camera_yaw,
            self.player.camera_pitch,
            dist,
            Some(&ray_probe),
        );
    }

    fn drain_scm_world_commands(&mut self) {
        let cmds = std::mem::take(&mut self.scm.world_commands);
        for cmd in cmds {
            match cmd {
                ScmWorldCommand::SetTimeOfDay { hour, minute } => {
                    self.clock.hour = hour % 24;
                    self.clock.minute = minute % 60;
                }
                ScmWorldCommand::ForceWeather(id) => {
                    self.world.weather = weather_from_sa_id(id);
                }
                ScmWorldCommand::CreateCar { model_id, pos, .. } => {
                    let p = Vec3::from_array(pos);
                    if self.vehicles.vehicles.len() < 48
                        && p.length_squared() > 1.0
                        && !self
                            .vehicles
                            .vehicles
                            .iter()
                            .any(|v| v.position.truncate().distance_squared(p.truncate()) < 16.0)
                        && let Some(def) = self.world.vehicles.get(&(model_id as u32))
                    {
                        self.vehicles.spawn(VehicleSpawnSpec {
                            model_id: model_id as u32,
                            name: &def.handling_id,
                            class: vehicle_class_from_ide(&def.vehicle_type),
                            position: p,
                            yaw: 0.0,
                            color1: sample_sa_car_color(model_id as usize),
                            handling: self.handling.as_ref(),
                        });
                    }
                }
                ScmWorldCommand::CreatePickup { model_id, pos, .. } => {
                    let p = Vec3::from_array(pos);
                    if self.scm_pickups.len() < 128
                        && !self
                            .scm_pickups
                            .iter()
                            .any(|(ep, _)| ep.distance_squared(p) < 1.0)
                    {
                        self.scm_pickups.push((p, model_id));
                    }
                }
                ScmWorldCommand::AddBlip { pos, .. } => {
                    let p = Vec3::from_array(pos);
                    if self.scm_blips.len() < 64
                        && !self
                            .scm_blips
                            .iter()
                            .any(|(bp, _)| bp.distance_squared(p) < 4.0)
                    {
                        self.scm_blips.push((p, 0));
                    }
                }
                ScmWorldCommand::PrintText {
                    key, duration_ms, ..
                } => {
                    let txt = self.localization.get("MAIN", &key).to_string();
                    if !txt.is_empty() && txt != key {
                        self.set_status(txt, (duration_ms as f32 / 1000.0).clamp(2.0, 10.0));
                    }
                }
                ScmWorldCommand::CreatePlayer { .. }
                | ScmWorldCommand::CreateChar { .. }
                | ScmWorldCommand::CreateCarGenerator { .. }
                | ScmWorldCommand::DoFade { .. }
                | ScmWorldCommand::LoadScene(_)
                | ScmWorldCommand::DisplayHud(_)
                | ScmWorldCommand::DisplayRadar(_)
                | ScmWorldCommand::RequestModel(_) => {
                    debug!(?cmd, "Processed SCM boot/script opcode");
                }
            }
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

    fn load_single_txd(&mut self, renderer: &mut SceneRenderer, txd_lower: &str) {
        if txd_lower.is_empty() || !self.loaded_txds.insert(txd_lower.to_string()) {
            return;
        }
        let txd_entry = format!("{txd_lower}.txd");
        let bytes = self.read_archive_entry(&txd_entry).or_else(|| {
            self.vfs.as_ref().and_then(|v| {
                v.read_bytes(&format!("models/generic/{txd_entry}"))
                    .or_else(|_| v.read_bytes(&format!("models/{txd_entry}")))
                    .or_else(|_| v.read_bytes(&format!("models/txd/{txd_entry}")))
                    .ok()
            })
        });
        let Some(bytes) = bytes else {
            return;
        };
        for (name_lower, w, h, rgba, has_alpha) in decode_txd_with_masks(&bytes) {
            let tex_idx =
                renderer.upload_rgba8_texture(&format!("{txd_lower}/{name_lower}"), w, h, &rgba);
            if has_alpha {
                self.alpha_textures.insert(tex_idx);
            }
            self.txd_first_tex
                .entry(txd_lower.to_string())
                .or_insert(tex_idx);
            self.cached_textures
                .insert(format!("{txd_lower}/{name_lower}"), tex_idx);
            self.cached_textures.entry(name_lower).or_insert(tex_idx);
        }
    }

    fn ensure_txd_loaded(&mut self, renderer: &mut SceneRenderer, txd_name: &str) -> Vec<String> {
        let chain = self.world.resolve_txd_chain(txd_name);
        for txd in chain.iter().rev() {
            self.load_single_txd(renderer, txd);
        }
        chain
    }

    fn resolve_texture_idx(
        &self,
        txd_chain: &[String],
        tex_name: Option<&str>,
        fallback: usize,
    ) -> usize {
        let Some(raw) = tex_name else {
            return fallback;
        };
        let tex_lower = raw.to_ascii_lowercase();
        for txd_lower in txd_chain {
            if let Some(&idx) = self
                .cached_textures
                .get(&format!("{txd_lower}/{tex_lower}"))
            {
                return idx;
            }
        }
        for shared in ["generic", "particle", "vehicle", "effectspc", "misc"] {
            if let Some(&idx) = self.cached_textures.get(&format!("{shared}/{tex_lower}")) {
                return idx;
            }
        }
        self.cached_textures
            .get(&tex_lower)
            .copied()
            .unwrap_or(fallback)
    }

    fn upload_submeshes(
        &self,
        renderer: &mut SceneRenderer,
        label_prefix: &str,
        subs: Vec<DffSubMesh>,
        txd_chain: &[String],
    ) -> (Vec<CachedSubMesh>, f32, f32) {
        let mut min_z = f32::INFINITY;
        let mut max_r = 2.0_f32;
        let mut cached_subs = Vec::with_capacity(subs.len());
        for (idx, sub) in subs.into_iter().enumerate() {
            min_z = min_z.min(sub.cpu_mesh.min_z());
            let tex_idx = self.resolve_texture_idx(
                txd_chain,
                sub.texture_name.as_deref(),
                renderer.white_tex,
            );
            let handle = renderer.upload_mesh(&format!("{label_prefix}#{idx}"), &sub.cpu_mesh);
            max_r = max_r.max(renderer.mesh_radius(handle));
            let has_alpha_tex = self.alpha_textures.contains(&tex_idx);
            cached_subs.push(CachedSubMesh {
                mesh: handle,
                texture_idx: tex_idx,
                material_color: sub.material_color,
                has_alpha_tex,
            });
        }
        if !min_z.is_finite() {
            min_z = 0.0;
        }
        (cached_subs, min_z, max_r)
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
        let txd_chain = self.ensure_txd_loaded(renderer, txd_lower);
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
        let (cached_subs, _, bounding_radius) =
            self.upload_submeshes(renderer, &dff_name, subs, &txd_chain);
        self.cached_dff_meshes.insert(
            dff_name,
            CachedDffModel {
                submeshes: cached_subs,
                bounding_radius,
            },
        );
        true
    }

    fn load_vehicle_model(
        &mut self,
        renderer: &mut SceneRenderer,
        model_lower: &str,
        txd_lower: &str,
    ) -> bool {
        let dff_name = format!("{model_lower}.dff");
        if self.cached_vehicle_models.contains_key(&dff_name)
            || self.failed_dff_lookups.contains(&dff_name)
        {
            return false;
        }
        let txd_chain = self.ensure_txd_loaded(renderer, txd_lower);
        let Some(bytes) = self.read_archive_entry(&dff_name) else {
            self.failed_dff_lookups.insert(dff_name);
            return false;
        };
        let Ok(clump) = DffClump::parse(&bytes) else {
            self.failed_dff_lookups.insert(dff_name);
            return false;
        };
        let parts = CpuMeshData::dff_vehicle_parts(&clump);
        if parts.body.is_empty() && parts.wheel.is_empty() {
            self.failed_dff_lookups.insert(dff_name);
            return false;
        }
        let wheel_local_min_z = parts
            .wheel
            .iter()
            .map(|s| s.cpu_mesh.min_z())
            .fold(f32::INFINITY, f32::min);
        let (body, mut min_z, _) = self.upload_submeshes(
            renderer,
            &format!("{dff_name}/body"),
            parts.body,
            &txd_chain,
        );
        let (wheel, _, _) = self.upload_submeshes(
            renderer,
            &format!("{dff_name}/wheel"),
            parts.wheel,
            &txd_chain,
        );
        if wheel_local_min_z.is_finite() {
            for mount in &parts.wheel_mounts {
                min_z = min_z.min(mount.offset.z + wheel_local_min_z);
            }
        }
        self.cached_vehicle_models.insert(
            dff_name,
            CachedVehicleModel {
                body,
                wheel,
                wheel_mounts: parts.wheel_mounts,
                min_z,
            },
        );
        true
    }

    fn load_ped_rig(&mut self, renderer: &mut SceneRenderer, model_lower: &str) -> bool {
        if self.ped_rigs.contains_key(model_lower) {
            return true;
        }
        let dff_name = format!("{model_lower}.dff");
        if self.failed_dff_lookups.contains(&dff_name) {
            return false;
        }
        let txd_chain = self.ensure_txd_loaded(renderer, model_lower);
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
        let skin = clump.geometries.iter().find_map(|g| g.skin.clone());
        let submeshes = subs
            .into_iter()
            .map(|sub| {
                let texture_idx = self.resolve_texture_idx(
                    &txd_chain,
                    sub.texture_name.as_deref(),
                    renderer.white_tex,
                );
                let material_color = sub.material_color;
                SkinnedSubMeshTemplate {
                    sub,
                    texture_idx,
                    material_color,
                }
            })
            .collect();
        self.ped_rigs.insert(
            model_lower.to_string(),
            CachedSkinnedPedRig {
                frames: clump.frames,
                skin,
                submeshes,
            },
        );
        true
    }

    fn load_cj_from_player_img(&mut self, renderer: &mut SceneRenderer) {
        let Some(player_img) = &self.player_archive else {
            return;
        };
        // Default CJ Grove Street outfit: head (`player_face`), white tank top (`vest`), blue jeans (`jeansdenim`), sneakers (`sneakerbincblk`), bare hands (`hands`).
        const CJ_PARTS: [(&str, &str); 5] = [
            ("head", "player_face"),
            ("vest", "vest"),
            ("jeans", "jeansdenim"),
            ("sneaker", "sneakerbincblk"),
            ("hands", "hands"),
        ];
        for (dff_stem, txd_stem) in CJ_PARTS {
            let mut part_textures = HashMap::new();
            let mut primary_tex = None;
            let txd_entry = format!("{txd_stem}.txd");
            if let Ok(txd_bytes) = player_img.read_entry(&txd_entry) {
                for (tex_lower, w, h, rgba, has_alpha) in decode_txd_with_masks(&txd_bytes) {
                    let tid = renderer.upload_rgba8_texture(
                        &format!("cj/{txd_stem}/{tex_lower}"),
                        w,
                        h,
                        &rgba,
                    );
                    if has_alpha {
                        self.alpha_textures.insert(tid);
                    }
                    if primary_tex.is_none() {
                        primary_tex = Some(tid);
                    }
                    part_textures.insert(tex_lower, tid);
                }
            }
            let Ok(dff_bytes) = player_img.read_entry(&format!("{dff_stem}.dff")) else {
                continue;
            };
            let Ok(clump) = DffClump::parse(&dff_bytes) else {
                continue;
            };
            let subs = CpuMeshData::dff_clump_submeshes(&clump);
            if subs.is_empty() {
                continue;
            }
            let skin = clump.geometries.iter().find_map(|g| g.skin.clone());
            let mut submeshes = Vec::with_capacity(subs.len());
            let mut mesh_handles = Vec::with_capacity(subs.len());
            for (idx, sub) in subs.into_iter().enumerate() {
                let fallback_tex = primary_tex.unwrap_or(renderer.white_tex);
                let texture_idx = sub
                    .texture_name
                    .as_deref()
                    .and_then(|n| part_textures.get(&n.to_ascii_lowercase()).copied())
                    .unwrap_or(fallback_tex);
                let material_color = sub.material_color;
                let handle = renderer
                    .upload_dynamic_mesh(&format!("cj_part_{dff_stem}#{idx}"), &sub.cpu_mesh);
                mesh_handles.push((handle, texture_idx, material_color));
                submeshes.push(SkinnedSubMeshTemplate {
                    sub,
                    texture_idx,
                    material_color,
                });
            }
            self.cj_parts.push(CachedCjPart {
                rig: CachedSkinnedPedRig {
                    frames: clump.frames,
                    skin,
                    submeshes,
                },
                mesh_handles,
            });
        }
        if !self.cj_parts.is_empty() {
            info!(
                parts = self.cj_parts.len(),
                "Loaded modular skinned CJ parts from player.img"
            );
        }
    }

    pub fn preload_render_meshes(&mut self, renderer: &mut SceneRenderer) {
        if let Some(water_cpu) = CpuMeshData::from_water_quads(&self.world.water_quads) {
            self.water_mesh = Some(renderer.upload_mesh("sa_water_quads", &water_cpu));
        }
        self.marker_mesh =
            Some(renderer.upload_mesh("sa_enex_marker", &CpuMeshData::marker_cone(0.62, 1.15)));
        if self.player_archive.is_some() {
            self.load_cj_from_player_img(renderer);
        }
        if self.gta3_archive.is_none() && self.gta_int_archive.is_none() {
            return;
        }

        // Shared vehicle/generic TXDs resolved as fallback & txdp roots
        for shared_txd in ["generic", "particle", "vehicle", "effectspc", "misc"] {
            self.load_single_txd(renderer, shared_txd);
        }

        // Skinned pedestrian models (`ped.ifp` bone rigs)
        for ped_model in ["fam1", "fam2", "fam3", "bmydj", "lapd1", "hmost"] {
            self.load_ped_rig(renderer, ped_model);
        }

        // Preload starter vehicles (`GREENWOO`, `BMX`, `INFERNUS`, `COPCARLA`) with 4-wheel mounts
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
            self.load_vehicle_model(renderer, &model, &txd);
        }

        self.world
            .update_streaming(self.player.position, self.config.graphics.draw_distance);
        let initial_loaded = self.stream_nearby_render_assets(renderer, 350);
        if initial_loaded > 0 || !self.cached_dff_meshes.is_empty() {
            info!(
                models = self.cached_dff_meshes.len(),
                vehicles = self.cached_vehicle_models.len(),
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
        let mut pending_vehicles: Vec<(String, String)> = Vec::new();
        for v in &self.vehicles.vehicles {
            let def = self.world.vehicles.get(&v.model_id);
            let model_lower = def
                .map(|d| d.model_name.to_ascii_lowercase())
                .unwrap_or_else(|| v.name.to_ascii_lowercase());
            let dff_name = format!("{model_lower}.dff");
            if !self.cached_vehicle_models.contains_key(&dff_name)
                && !self.failed_dff_lookups.contains(&dff_name)
            {
                let txd_lower = def
                    .map(|d| d.txd_name.to_ascii_lowercase())
                    .unwrap_or_else(|| model_lower.clone());
                pending_vehicles.push((model_lower, txd_lower));
            }
        }
        let mut loaded = 0usize;
        for (model_lower, txd_lower) in pending_vehicles {
            if loaded >= budget {
                return loaded;
            }
            if self.load_vehicle_model(renderer, &model_lower, &txd_lower) {
                loaded += 1;
            }
        }

        let detail_cap = (self.config.graphics.draw_distance * 0.65).clamp(180.0, 320.0);
        let max_draw_sq = self.config.graphics.draw_distance.powi(2);
        let mut pending: Vec<(f32, String, String)> = Vec::new();
        for p in self.world.active_placements(self.player.interior) {
            if !p.is_active_at_hour(self.clock.hour) {
                continue;
            }
            let dist_sq = p.position.distance_squared(self.player.position);
            if dist_sq > max_draw_sq {
                continue;
            }
            if p.is_lod && dist_sq < (detail_cap - 20.0).powi(2) {
                continue;
            }
            let model_lower = p.model_name.to_ascii_lowercase();
            let dff_name = format!("{model_lower}.dff");
            if self.cached_dff_meshes.contains_key(&dff_name)
                || self.failed_dff_lookups.contains(&dff_name)
            {
                continue;
            }
            // Stream nearby non-LOD street geometry before distant skyline LODs
            let priority = if p.is_lod {
                dist_sq + 200_000.0
            } else {
                dist_sq
            };
            pending.push((priority, model_lower, p.txd_name.to_ascii_lowercase()));
        }
        pending.sort_by(|a, b| a.0.total_cmp(&b.0));

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

    /// Deform CJ & active NPC pedestrian vertex buffers using sampled `ped.ifp` bone poses.
    pub fn update_skinned_meshes(&mut self, renderer: &mut SceneRenderer) {
        if self.cj_parts.is_empty() && self.ped_rigs.is_empty() {
            return;
        }

        if self.player.control_mode == ControlMode::OnFoot {
            let poses = if !self.ped_ifp_lib.is_empty() {
                Some(self.player.anim.sample_poses(&self.ped_ifp_lib))
            } else {
                None
            };
            if !self.cj_parts.is_empty() {
                if let Some(poses) = &poses {
                    for part in &mut self.cj_parts {
                        if let Some(skin) = &part.rig.skin {
                            let mats = compute_skin_matrices(&part.rig.frames, skin, poses);
                            if !mats.is_empty() {
                                for (idx, tpl) in part.rig.submeshes.iter().enumerate() {
                                    if tpl.sub.skin_weights.is_some()
                                        && let Some(&(handle, _, _)) = part.mesh_handles.get(idx)
                                    {
                                        tpl.sub.compute_skinned_vertices(
                                            &mats,
                                            &mut self.skin_vertex_scratch,
                                        );
                                        renderer.update_mesh_vertices(
                                            handle,
                                            &self.skin_vertex_scratch,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            } else if let Some(rig) = self.ped_rigs.get("fam1") {
                if self.cj_skinned_meshes.is_empty() {
                    for (idx, tpl) in rig.submeshes.iter().enumerate() {
                        let handle = renderer
                            .upload_dynamic_mesh(&format!("cj_skin#{idx}"), &tpl.sub.cpu_mesh);
                        self.cj_skinned_meshes
                            .push((handle, tpl.texture_idx, tpl.material_color));
                    }
                }
                if let (Some(poses), Some(skin)) = (&poses, &rig.skin) {
                    let mats = compute_skin_matrices(&rig.frames, skin, poses);
                    if !mats.is_empty() {
                        for (idx, tpl) in rig.submeshes.iter().enumerate() {
                            if tpl.sub.skin_weights.is_some()
                                && let Some(&(handle, _, _)) = self.cj_skinned_meshes.get(idx)
                            {
                                tpl.sub
                                    .compute_skinned_vertices(&mats, &mut self.skin_vertex_scratch);
                                renderer.update_mesh_vertices(handle, &self.skin_vertex_scratch);
                            }
                        }
                    }
                }
            }
        }

        // Prune despawned or dead pedestrian dynamic mesh handles
        self.ped_skinned_meshes.retain(|pid, _| {
            self.ai
                .pedestrians
                .iter()
                .any(|p| p.id == *pid && p.health > 0.0)
        });

        let Self {
            ped_rigs,
            ped_ifp_lib,
            ped_skinned_meshes,
            skin_vertex_scratch,
            ai,
            player,
            timestep,
            ..
        } = self;

        for ped in &ai.pedestrians {
            if ped.health <= 0.0 || ped.position.distance_squared(player.position) > 75.0 * 75.0 {
                continue;
            }
            let preferred = ped_model_name_for(ped.model_id);
            let model_key = if ped_rigs.contains_key(preferred) {
                preferred
            } else if ped_rigs.contains_key("fam2") {
                "fam2"
            } else if ped_rigs.contains_key("fam1") {
                "fam1"
            } else {
                continue;
            };
            let Some(rig) = ped_rigs.get(model_key) else {
                continue;
            };

            let (_, handles) = ped_skinned_meshes.entry(ped.id).or_insert_with(|| {
                let hs = rig
                    .submeshes
                    .iter()
                    .enumerate()
                    .map(|(idx, tpl)| {
                        renderer.upload_dynamic_mesh(
                            &format!("ped_{}_{model_key}#{idx}", ped.id),
                            &tpl.sub.cpu_mesh,
                        )
                    })
                    .collect();
                (model_key.to_string(), hs)
            });

            if !ped_ifp_lib.is_empty()
                && let Some(skin) = &rig.skin
            {
                let npc_state = match ped.behavior {
                    PedBehavior::Idle => AnimState::Idle,
                    PedBehavior::Patrol => AnimState::Walk,
                    PedBehavior::FleePanic => AnimState::Sprint,
                    PedBehavior::PolicePursuit | PedBehavior::GangHostile => AnimState::Run,
                };
                let phase = (timestep.total_sim_time as f32) + (ped.id as f32) * 0.67;
                let poses = ped_ifp_lib.sample_clip(
                    AnimationController::clip_name_for(npc_state),
                    phase,
                    true,
                );
                let mats = compute_skin_matrices(&rig.frames, skin, &poses);
                if !mats.is_empty() {
                    for (idx, tpl) in rig.submeshes.iter().enumerate() {
                        if tpl.sub.skin_weights.is_some()
                            && let Some(&handle) = handles.get(idx)
                        {
                            tpl.sub.compute_skinned_vertices(&mats, skin_vertex_scratch);
                            renderer.update_mesh_vertices(handle, skin_vertex_scratch);
                        }
                    }
                }
            }
        }
    }

    /// Keep `PhysicsWorld` static `.col` colliders synchronized with active world sectors.
    pub fn sync_collision_sectors(&mut self) {
        let mut changed = false;
        let interior = self.player.interior;
        if self.loaded_collider_interior != interior {
            for (_, handles) in self.sector_colliders.drain() {
                for h in handles {
                    self.physics.remove_collider(h);
                }
                changed = true;
            }
            self.loaded_collider_interior = interior;
        }
        if self.world.col_models.is_empty() {
            if changed {
                self.physics.sync_queries();
            }
            return;
        }
        let stale: Vec<SectorCoord> = self
            .sector_colliders
            .keys()
            .copied()
            .filter(|sc| !self.world.active_sectors.contains(sc))
            .collect();
        for sc in stale {
            if let Some(handles) = self.sector_colliders.remove(&sc) {
                for h in handles {
                    self.physics.remove_collider(h);
                }
                changed = true;
            }
        }

        let new_sectors: Vec<SectorCoord> = self
            .world
            .active_sectors
            .iter()
            .copied()
            .filter(|sc| !self.sector_colliders.contains_key(sc))
            .collect();
        for sc in new_sectors {
            let mut handles = Vec::new();
            if let Some(placements) = self.world.sectors.get(&sc) {
                for p in placements {
                    if p.interior != interior || p.is_lod {
                        continue;
                    }
                    let key = p.model_name.to_ascii_lowercase();
                    let Some(col) = self.world.col_models.get(&key) else {
                        continue;
                    };
                    for b in &col.boxes {
                        let bmin = Vec3::from_array(b.min);
                        let bmax = Vec3::from_array(b.max);
                        handles.push(self.physics.add_static_oriented_box(
                            (bmin + bmax) * 0.5,
                            (bmax - bmin) * 0.5,
                            p.position,
                            p.rotation,
                        ));
                    }
                    for s in &col.spheres {
                        handles.push(self.physics.add_static_sphere(
                            Vec3::from_array(s.center),
                            s.radius,
                            p.position,
                            p.rotation,
                        ));
                    }
                    if !col.vertices.is_empty() && !col.faces.is_empty() {
                        let tris: Vec<[u32; 3]> = col
                            .faces
                            .iter()
                            .map(|f| {
                                [
                                    u32::from(f.indices[0]),
                                    u32::from(f.indices[1]),
                                    u32::from(f.indices[2]),
                                ]
                            })
                            .collect();
                        if let Some(h) = self.physics.add_static_transformed_trimesh(
                            &col.vertices,
                            &tris,
                            p.position,
                            p.rotation,
                        ) {
                            handles.push(h);
                        }
                    }
                }
            }
            if !handles.is_empty() {
                changed = true;
            }
            self.sector_colliders.insert(sc, handles);
        }

        if changed {
            self.physics.sync_queries();
        }
    }

    /// Spawn nearby ambient parked vehicles parsed from binary/text IPL `CARS` sections.
    fn spawn_nearby_ipl_vehicles(&mut self, max_dist: f32, max_count: usize) {
        if self.world.car_spawns.is_empty() {
            return;
        }
        const FALLBACK_EAST_LS_CARS: [u32; 8] = [492, 467, 542, 405, 426, 445, 517, 534];
        let mut candidates: Vec<(f32, usize)> = self
            .world
            .car_spawns
            .iter()
            .enumerate()
            .filter_map(|(idx, cs)| {
                let pos = Vec3::from_array(cs.position);
                let d2 = pos
                    .truncate()
                    .distance_squared(self.player.position.truncate());
                (d2 <= max_dist * max_dist).then_some((d2, idx))
            })
            .collect();
        candidates.sort_by(|a, b| a.0.total_cmp(&b.0));

        let mut spawned = 0usize;
        for (_, idx) in candidates {
            if spawned >= max_count {
                break;
            }
            let cs = &self.world.car_spawns[idx];
            let pos = Vec3::from_array(cs.position);
            if self
                .vehicles
                .vehicles
                .iter()
                .any(|v| v.position.truncate().distance_squared(pos.truncate()) < 36.0)
            {
                continue;
            }
            let model_id = if cs.car_id > 0 {
                cs.car_id as u32
            } else {
                FALLBACK_EAST_LS_CARS[idx % FALLBACK_EAST_LS_CARS.len()]
            };
            let Some(def) = self.world.vehicles.get(&model_id) else {
                continue;
            };
            let class = vehicle_class_from_ide(&def.vehicle_type);
            if class == VehicleClass::Boat || def.vehicle_type.eq_ignore_ascii_case("trailer") {
                continue;
            }
            let palette_idx = if cs.color1 >= 0 {
                cs.color1 as usize
            } else {
                idx
            };
            let color1 = sample_sa_car_color(palette_idx);
            self.vehicles.spawn(VehicleSpawnSpec {
                model_id,
                name: &def.handling_id,
                class,
                position: pos + Vec3::new(0.0, 0.0, 0.15),
                yaw: cs.angle,
                color1,
                handling: self.handling.as_ref(),
            });
            spawned += 1;
        }
    }

    /// Project initial Grove Street spawns onto real `.col` road/terrain elevation when available.
    fn snap_initial_spawns_to_ground(&mut self) {
        if self.world.col_models.is_empty() {
            return;
        }
        if let Some(gz) = self.physics.ground_height_below(
            self.player.position.x,
            self.player.position.y,
            24.0,
            26.0,
        ) && gz > 0.5
        {
            self.player.position.z = gz + 0.05;
            self.player.velocity.z = 0.0;
            self.player.grounded = true;
        }
        for veh in &mut self.vehicles.vehicles {
            let ceiling = if veh.position.z > 2.0 {
                veh.position.z + 2.5
            } else {
                24.0
            };
            if let Some(gz) = self.physics.ground_height_below(
                veh.position.x,
                veh.position.y,
                ceiling,
                ceiling + 2.0,
            ) && gz > 0.5
            {
                veh.position.z = gz + 0.08;
                veh.velocity.z = 0.0;
            }
        }
        for ped in &mut self.ai.pedestrians {
            if let Some(gz) =
                self.physics
                    .ground_height_below(ped.position.x, ped.position.y, 24.0, 26.0)
                && gz > 0.5
            {
                ped.position.z = gz + 0.05;
            }
        }
        info!(
            player_pos = ?self.player.position,
            colliders = self.physics.collider_set.len(),
            vehicles = self.vehicles.vehicles.len(),
            "World physics and initial spawns grounded on COL surfaces"
        );
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

    fn set_status(&mut self, msg: impl Into<String>, duration: f32) {
        self.ui_state.status_message = Some(msg.into());
        self.status_message_timer = duration;
    }

    fn try_handle_enex_transition(&mut self) {
        if let Some(lock_pos) = self.enex_lock_pos
            && self.player.position.distance_squared(lock_pos) > 2.6 * 2.6
        {
            self.enex_lock_pos = None;
        }
        if self.player.control_mode != ControlMode::OnFoot
            || self.enex_cooldown > 0.0
            || self.enex_lock_pos.is_some()
        {
            return;
        }
        let candidates = self
            .world
            .nearby_enex(self.player.position, self.player.interior, 1.55);
        let Some(source) = candidates.into_iter().min_by(|a, b| {
            let da = Vec3::from_array(a.entrance).distance_squared(self.player.position);
            let db = Vec3::from_array(b.entrance).distance_squared(self.player.position);
            da.total_cmp(&db)
        }) else {
            return;
        };
        let Some(dest) = self
            .world
            .find_paired_enex(source, self.player.interior)
            .cloned()
        else {
            return;
        };

        let dest_yaw = (dest.exit_rot + 90.0).to_radians();
        self.player.position = Vec3::from_array(dest.exit);
        self.player.yaw = dest_yaw;
        self.player.velocity = Vec3::ZERO;
        self.player.interior = dest.target_interior;
        self.enex_cooldown = 1.5;
        self.enex_lock_pos = Some(Vec3::from_array(dest.entrance));

        self.world
            .update_streaming(self.player.position, self.config.graphics.draw_distance);
        self.sync_collision_sectors();

        if dest.target_interior != 0 {
            if !self.world.col_models.is_empty()
                && let Some(gz) = self.physics.ground_height_below(
                    dest.exit[0],
                    dest.exit[1],
                    dest.exit[2] + 2.0,
                    5.0,
                )
            {
                self.player.position.z = gz + 0.05;
                self.interior_floor_z = Some(gz - 0.5);
            } else {
                self.interior_floor_z = Some(dest.exit[2] - 0.35);
            }
            let upper = dest.name.to_ascii_uppercase();
            let fwd = Vec3::new(dest_yaw.cos(), dest_yaw.sin(), 0.0);
            self.interior_service_pos =
                if upper == "CARLS" || upper.starts_with("FDCHICK") || upper.starts_with("AMMUN") {
                    Some(self.player.position + fwd * 4.5)
                } else {
                    None
                };
            self.active_enex_name = Some(upper.clone());
            self.set_status(
                format!("Entered {upper} (Interior {})", dest.target_interior),
                4.0,
            );
        } else {
            self.interior_floor_z = None;
            self.active_enex_name = None;
            self.interior_service_pos = None;
            if !self.world.col_models.is_empty()
                && let Some(gz) = self.physics.ground_height_below(
                    dest.exit[0],
                    dest.exit[1],
                    dest.exit[2] + 3.0,
                    8.0,
                )
            {
                self.player.position.z = gz + 0.05;
            }
            self.set_status("Exited to San Andreas", 3.0);
        }
    }

    fn try_handle_interior_service(&mut self) {
        if self.player.interior == 0 || self.service_cooldown > 0.0 {
            return;
        }
        let Some(svc_pos) = self.interior_service_pos else {
            return;
        };
        if self.player.position.truncate().distance(svc_pos.truncate()) > 1.65 {
            return;
        }
        match self.active_enex_name.as_deref() {
            Some("CARLS") => {
                self.player.health = self.player.max_health;
                let data = self.to_save_data(1);
                match write_save_slot(&self.host.save_dir, 1, &data) {
                    Ok(_) => self.set_status("Safehouse: Health restored & saved to Slot 1", 4.5),
                    Err(e) => self.set_status(format!("Safehouse heal ok (save error: {e})"), 4.5),
                }
                self.service_cooldown = 4.0;
            }
            Some(name) if name.starts_with("FDCHICK") => {
                if self.player.money >= 10 {
                    self.player.money -= 10;
                    self.player.health = (self.player.health + 50.0).min(self.player.max_health);
                    self.set_status("Cluckin' Bell: Bought meal (-$10, +50 HP)", 4.0);
                } else {
                    self.set_status("Cluckin' Bell: Need $10 for meal", 3.5);
                }
                self.service_cooldown = 3.5;
            }
            Some(name) if name.starts_with("AMMUN") => {
                if self.player.money >= 200 {
                    self.player.money -= 200;
                    if let Some(slot2) = &mut self.player.weapons.slots[2] {
                        slot2.reserve_ammo += 50;
                    } else {
                        self.player
                            .weapons
                            .give_weapon(WeaponInventory::preset_colt45(), 50);
                    }
                    self.set_status("Ammu-Nation: Purchased 9mm +50 rounds (-$200)", 4.0);
                } else {
                    self.set_status("Ammu-Nation: Need $200 for 9mm ammo", 3.5);
                }
                self.service_cooldown = 3.5;
            }
            _ => {}
        }
    }

    pub fn step_fixed_tick(&mut self, dt: f32, actions: &ActionFrameState) {
        self.clock.advance(dt);
        self.audio.apply_config(&self.config.audio);
        self.audio.update(dt);
        self.enex_cooldown = (self.enex_cooldown - dt).max(0.0);
        self.service_cooldown = (self.service_cooldown - dt).max(0.0);
        if self.status_message_timer > 0.0 {
            self.status_message_timer -= dt;
            if self.status_message_timer <= 0.0 {
                self.ui_state.status_message = None;
            }
        }

        if actions.enter_exit_just_pressed {
            let was_on_foot = self.player.control_mode == ControlMode::OnFoot;
            if self.vehicles.toggle_player_entry(&mut self.player) {
                if was_on_foot && let Some(vid) = self.player.current_vehicle {
                    self.ai.on_vehicle_entered(vid, &mut self.player);
                    self.audio.set_in_vehicle(true);
                    if self.config.audio.radio_auto_tune && self.audio.current_station_idx == 0 {
                        self.audio.cycle_radio_next();
                    }
                } else {
                    self.audio.set_in_vehicle(false);
                }
            }
        }

        if actions.radio_next && self.player.control_mode == ControlMode::InVehicle {
            self.audio.cycle_radio_next();
        }

        // Fire active weapon on foot
        if self.player.control_mode == ControlMode::OnFoot
            && actions.fire_or_attack
            && let Some((damage, range, _explosive)) = self.player.weapons.try_fire()
        {
            let shot_cue = match self.player.weapons.active().map(|w| w.spec.kind) {
                Some(WeaponKind::Unarmed | WeaponKind::Melee) => SfxCue::Punch,
                Some(
                    WeaponKind::Shotgun
                    | WeaponKind::AssaultRifle
                    | WeaponKind::SniperRifle
                    | WeaponKind::Heavy,
                ) => SfxCue::HeavyShot,
                _ => SfxCue::PistolShot,
            };
            self.audio.play_sfx(shot_cue);
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
        if self.player.interior != 0
            && let Some(floor_z) = self.interior_floor_z
            && self.player.position.z < floor_z
        {
            self.player.position.z = floor_z;
            self.player.velocity.z = 0.0;
            self.player.grounded = true;
        }
        self.try_handle_enex_transition();
        self.try_handle_interior_service();
        if self.player.control_mode == ControlMode::OnFoot
            && self.player.grounded
            && self.player.velocity.truncate().length() > 1.2
        {
            self.audio.play_sfx(SfxCue::Footstep);
        }

        let occupied_id = self.player.current_vehicle;
        if occupied_id.is_some() && actions.crouch_or_horn {
            self.audio.play_sfx(SfxCue::CarHorn);
        }
        let mut player_crash = false;
        for veh in &mut self.vehicles.vehicles {
            let ai_action = if Some(veh.id) == occupied_id {
                None
            } else {
                self.ai.traffic_action_for(
                    veh,
                    self.player.position,
                    &self.physics,
                    &self.world.path_graph,
                )
            };
            let input_for_veh = if Some(veh.id) == occupied_id {
                Some(actions)
            } else {
                ai_action.as_ref()
            };
            let prev_hp = veh.health;
            veh.step_physics(dt, input_for_veh, &self.physics);
            if Some(veh.id) == occupied_id {
                if prev_hp - veh.health > 8.0 {
                    player_crash = true;
                }
                self.player.position = veh.position;
                self.player.yaw = veh.yaw;
            }
        }
        if player_crash {
            self.audio.play_sfx(SfxCue::Crash);
        }

        self.physics.step(dt);
        self.ai.tick_world(
            dt,
            &mut self.player,
            &mut self.vehicles,
            self.handling.as_ref(),
            &self.physics,
            &self.world.path_graph,
        );
        self.scm.tick(dt);
        self.drain_scm_world_commands();
        self.missions.tick(dt, &mut self.player, &mut self.scm);

        if matches!(
            self.player.life_state,
            PlayerLifeState::Wasted | PlayerLifeState::Busted
        ) {
            let hospital_z = self
                .physics
                .ground_height_below(2033.0, -1416.0, 35.0, 40.0)
                .map(|z| z + 0.1)
                .unwrap_or(17.0);
            self.player
                .respawn_at_hospital(Vec3::new(2033.0, -1416.0, hospital_z));
        }

        self.world
            .update_streaming(self.player.position, self.config.graphics.draw_distance);
        self.sync_collision_sectors();
        self.update_player_camera();
    }

    pub fn apply_ui_actions(&mut self, actions: UiActionQueue) {
        if actions.quit_requested {
            self.quit_requested = true;
        }
        if actions.start_new_game {
            self.player = PlayerEntity::default();
            self.player.position = Vec3::new(2488.56, -1666.86, 13.38);
            self.player
                .weapons
                .give_weapon(WeaponInventory::preset_colt45(), 51);
            self.missions = MissionManager::new_san_andreas_campaign();
            self.interior_floor_z = None;
            self.active_enex_name = None;
            self.interior_service_pos = None;
            self.enex_lock_pos = None;
            self.world
                .update_streaming(self.player.position, self.config.graphics.draw_distance);
            self.sync_collision_sectors();
            self.snap_initial_spawns_to_ground();
            self.update_player_camera();
        }
        if actions.resume_game {
            self.update_player_camera();
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
            self.player.interior = 0;
            self.interior_floor_z = None;
            self.active_enex_name = None;
            self.interior_service_pos = None;
            self.enex_lock_pos = None;
            self.world
                .update_streaming(self.player.position, self.config.graphics.draw_distance);
            self.sync_collision_sectors();
            if tp.z < 1.0
                && !self.world.col_models.is_empty()
                && let Some(gz) = self.physics.ground_height_below(tp.x, tp.y, 120.0, 140.0)
                && gz > 0.5
            {
                self.player.position.z = gz + 0.1;
            }
            self.update_player_camera();
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
            interior: self.player.interior,
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
        self.player.interior = data.interior;
        self.player.health = data.health;
        self.player.max_health = data.max_health;
        self.player.armor = data.armor;
        self.player.money = data.money;
        self.player.respect = data.respect;
        self.player.wanted_stars = data.wanted_stars;
        self.missions.completed = data.completed_missions;
        self.interior_floor_z = (self.player.interior != 0).then_some(self.player.position.z - 0.5);
        self.world
            .update_streaming(self.player.position, self.config.graphics.draw_distance);
        self.sync_collision_sectors();
    }

    pub fn collect_render_batches(&self, renderer: &SceneRenderer) -> Vec<RenderInstanceBatch> {
        let frustum = self.camera.frustum();
        let in_exterior = self.player.interior == 0;
        let ground_instances = if in_exterior && !self.world.loaded_from_original_data {
            vec![InstanceData::from_transform_lit(
                Vec3::new(self.player.position.x, self.player.position.y, 0.0),
                Quat::IDENTITY,
                Vec3::ONE,
                [0.85, 0.88, 0.80, 1.0],
                1.0,
                0.0,
            )]
        } else {
            Vec::new()
        };

        let mut box_instances = Vec::new();
        let mut custom_batches: HashMap<(usize, usize, bool, bool), Vec<InstanceData>> =
            HashMap::new();

        // Player character (CJ): modular player.img rig -> fallback fam1 rig -> fallback box
        if self.player.control_mode == ControlMode::OnFoot {
            let rot = Quat::from_rotation_z(self.player.yaw - std::f32::consts::FRAC_PI_2);
            let pos = self.player.position + Vec3::Z;
            if !self.cj_parts.is_empty() {
                for part in &self.cj_parts {
                    for &(mesh, tex_idx, mat_col) in &part.mesh_handles {
                        let transparent =
                            self.alpha_textures.contains(&tex_idx) || mat_col[3] < 0.98;
                        custom_batches
                            .entry((mesh.0, tex_idx, transparent, false))
                            .or_default()
                            .push(InstanceData::from_transform_lit(
                                pos,
                                rot,
                                Vec3::ONE,
                                mat_col,
                                1.0,
                                0.05,
                            ));
                    }
                }
            } else if !self.cj_skinned_meshes.is_empty() {
                for &(mesh, tex_idx, mat_col) in &self.cj_skinned_meshes {
                    let transparent = self.alpha_textures.contains(&tex_idx) || mat_col[3] < 0.98;
                    custom_batches
                        .entry((mesh.0, tex_idx, transparent, false))
                        .or_default()
                        .push(InstanceData::from_transform_lit(
                            pos,
                            rot,
                            Vec3::ONE,
                            mat_col,
                            1.0,
                            0.05,
                        ));
                }
            } else {
                box_instances.push(InstanceData::from_transform_lit(
                    self.player.position + Vec3::new(0.0, 0.0, 0.9),
                    Quat::from_rotation_z(self.player.yaw),
                    Vec3::new(0.65, 0.45, 1.8),
                    [0.95, 0.95, 0.95, 1.0],
                    1.0,
                    0.0,
                ));
            }
        }

        if in_exterior {
            // Vehicles (RenderWare body + 4 articulated wheel mounts, fallback bounds)
            for v in &self.vehicles.vehicles {
                if !frustum.intersects_sphere(v.position, 6.0) {
                    continue;
                }
                let model_lower = self
                    .world
                    .vehicles
                    .get(&v.model_id)
                    .map(|d| d.model_name.to_ascii_lowercase())
                    .unwrap_or_else(|| v.name.to_ascii_lowercase());
                let dff_key = format!("{model_lower}.dff");
                if let Some(v_model) = self.cached_vehicle_models.get(&dff_key) {
                    let z_lift = if v_model.min_z.is_finite() {
                        (-v_model.min_z).clamp(0.0, 1.5)
                    } else {
                        0.0
                    };
                    let body_pos = v.position + Vec3::new(0.0, 0.0, z_lift);
                    // Match RenderWare GTA:SA forward-axis (+Y) orientation cleanly to yaw rotation
                    let body_rot =
                        v.rotation_quat() * Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2);
                    for sub in &v_model.body {
                        let tint = apply_vehicle_paint_tint(sub.material_color, v.color1, v.color2);
                        let transparent = tint[3] < 0.98;
                        let alpha_cutoff = if sub.has_alpha_tex && !transparent {
                            0.35
                        } else if transparent {
                            0.02
                        } else {
                            0.0
                        };
                        custom_batches
                            .entry((sub.mesh.0, sub.texture_idx, transparent, false))
                            .or_default()
                            .push(InstanceData::from_transform_lit(
                                body_pos,
                                body_rot,
                                Vec3::ONE,
                                tint,
                                1.0,
                                alpha_cutoff,
                            ));
                    }
                    if !v_model.wheel.is_empty() {
                        for mount in &v_model.wheel_mounts {
                            let flip = if mount.is_left {
                                Quat::from_rotation_z(std::f32::consts::PI)
                            } else {
                                Quat::IDENTITY
                            };
                            let spin = Quat::from_rotation_x(-v.wheel_spin);
                            let steer = if mount.is_front {
                                Quat::from_rotation_z(v.steer_angle)
                            } else {
                                Quat::IDENTITY
                            };
                            let wheel_rot = body_rot * (steer * spin * flip);
                            let wheel_pos = body_pos + body_rot * mount.offset;
                            for sub in &v_model.wheel {
                                let transparent = sub.material_color[3] < 0.98;
                                custom_batches
                                    .entry((sub.mesh.0, sub.texture_idx, transparent, false))
                                    .or_default()
                                    .push(InstanceData::from_transform_lit(
                                        wheel_pos,
                                        wheel_rot,
                                        Vec3::ONE,
                                        sub.material_color,
                                        1.0,
                                        if sub.has_alpha_tex { 0.35 } else { 0.0 },
                                    ));
                            }
                        }
                    }
                } else {
                    let scale = match v.params.class {
                        VehicleClass::Bicycle | VehicleClass::Motorcycle => {
                            Vec3::new(1.8, 0.7, 1.1)
                        }
                        VehicleClass::Truck | VehicleClass::Bus => Vec3::new(6.8, 2.5, 2.8),
                        _ => Vec3::new(4.3, 1.9, 1.45),
                    };
                    box_instances.push(InstanceData::from_transform_lit(
                        v.position + Vec3::new(0.0, 0.0, scale.z * 0.5),
                        v.rotation_quat(),
                        scale,
                        [v.color1[0], v.color1[1], v.color1[2], 1.0],
                        1.0,
                        0.0,
                    ));
                }
            }

            // Pedestrians & Police (skinned ped.ifp models when uploaded, fallback colored box)
            for ped in &self.ai.pedestrians {
                if ped.health > 0.0 && frustum.intersects_sphere(ped.position, 2.0) {
                    if let Some((model_key, handles)) = self.ped_skinned_meshes.get(&ped.id)
                        && let Some(rig) = self.ped_rigs.get(model_key)
                    {
                        let rot = Quat::from_rotation_z(ped.yaw - std::f32::consts::FRAC_PI_2);
                        let pos = ped.position + Vec3::Z;
                        for (idx, handle) in handles.iter().enumerate() {
                            if let Some(tpl) = rig.submeshes.get(idx) {
                                let transparent = self.alpha_textures.contains(&tpl.texture_idx)
                                    || tpl.material_color[3] < 0.98;
                                custom_batches
                                    .entry((handle.0, tpl.texture_idx, transparent, false))
                                    .or_default()
                                    .push(InstanceData::from_transform_lit(
                                        pos,
                                        rot,
                                        Vec3::ONE,
                                        tpl.material_color,
                                        1.0,
                                        0.05,
                                    ));
                            }
                        }
                    } else {
                        let col = if ped.is_cop {
                            [0.15, 0.30, 0.75, 1.0]
                        } else {
                            [0.20, 0.65, 0.25, 1.0]
                        };
                        box_instances.push(InstanceData::from_transform_lit(
                            ped.position + Vec3::new(0.0, 0.0, 0.85),
                            Quat::from_rotation_z(ped.yaw),
                            Vec3::new(0.6, 0.4, 1.7),
                            col,
                            1.0,
                            0.0,
                        ));
                    }
                }
            }
        }

        // Mission trigger markers
        for trig in self.missions.available_triggers() {
            if frustum.intersects_sphere(trig.position, 4.0) {
                box_instances.push(InstanceData::from_transform_lit(
                    trig.position + Vec3::new(0.0, 0.0, 1.2),
                    Quat::IDENTITY,
                    Vec3::new(1.2, 1.2, 2.4),
                    [0.95, 0.25, 0.20, 0.75],
                    2.0,
                    0.0,
                ));
            }
        }

        // Active mission target beacon
        if let Some(cp) = self.missions.current_target_position() {
            box_instances.push(InstanceData::from_transform_lit(
                cp + Vec3::new(0.0, 0.0, 2.0),
                Quat::IDENTITY,
                Vec3::new(1.4, 1.4, 4.0),
                [0.98, 0.82, 0.15, 0.75],
                2.0,
                0.0,
            ));
        }

        // Floating ENEX doorway, interior counter, and SCM pickup markers
        if let Some(marker) = self.marker_mesh {
            let t = self.timestep.total_sim_time as f32;
            let bob = (t * 3.0).sin() * 0.14;
            let spin = Quat::from_rotation_z(t * 1.6);
            let cur_int = self.player.interior;
            let mut marker_instances = Vec::new();
            for e in &self.world.entry_exits {
                if e.target_interior != cur_int {
                    continue;
                }
                let pos = Vec3::from_array(e.entrance);
                if pos.distance_squared(self.player.position) <= 120.0 * 120.0
                    && frustum.intersects_sphere(pos + Vec3::Z, 2.0)
                {
                    marker_instances.push(InstanceData::from_transform_lit(
                        pos + Vec3::new(0.0, 0.0, 1.1 + bob),
                        spin,
                        Vec3::ONE,
                        [1.0, 0.85, 0.12, 0.92],
                        2.0,
                        0.0,
                    ));
                }
            }
            if cur_int == 0 {
                for &(pickup_pos, _) in &self.scm_pickups {
                    if pickup_pos.distance_squared(self.player.position) <= 120.0 * 120.0
                        && frustum.intersects_sphere(pickup_pos + Vec3::Z, 2.0)
                    {
                        marker_instances.push(InstanceData::from_transform_lit(
                            pickup_pos + Vec3::new(0.0, 0.0, 1.0 + bob),
                            spin,
                            Vec3::splat(0.75),
                            [0.20, 0.95, 0.65, 0.90],
                            2.0,
                            0.0,
                        ));
                    }
                }
            }
            if cur_int != 0
                && let Some(svc_pos) = self.interior_service_pos
            {
                let svc_color = if self
                    .active_enex_name
                    .as_deref()
                    .is_some_and(|n| n.starts_with("AMMUN"))
                {
                    [0.20, 0.72, 1.0, 0.95]
                } else {
                    [0.18, 0.92, 0.36, 0.95]
                };
                marker_instances.push(InstanceData::from_transform_lit(
                    svc_pos + Vec3::new(0.0, 0.0, 1.05 + bob),
                    spin,
                    Vec3::ONE,
                    svc_color,
                    2.0,
                    0.0,
                ));
            }
            if !marker_instances.is_empty() {
                custom_batches
                    .entry((marker.0, renderer.white_tex, true, false))
                    .or_default()
                    .extend(marker_instances);
            }
        }

        // World streamed objects (time-of-day active check, detail vs LOD skyline, real IDE flags & alpha masks)
        let detail_cap = (self.config.graphics.draw_distance * 0.65).clamp(180.0, 320.0);
        let max_draw_sq = self.config.graphics.draw_distance.powi(2);
        for p in self.world.active_placements(self.player.interior) {
            if !p.is_active_at_hour(self.clock.hour) {
                continue;
            }
            let dist_sq = p.position.distance_squared(self.player.position);
            if dist_sq > max_draw_sq {
                continue;
            }
            if p.is_lod {
                if !self.world.loaded_from_original_data || dist_sq < (detail_cap - 15.0).powi(2) {
                    continue;
                }
            } else {
                let obj_detail = if p.draw_distance > 1.0 {
                    p.draw_distance.max(120.0).min(detail_cap)
                } else {
                    detail_cap
                };
                if dist_sq > obj_detail.powi(2) {
                    continue;
                }
            }
            let dff_key = format!("{}.dff", p.model_name.to_ascii_lowercase());
            if let Some(model) = self.cached_dff_meshes.get(&dff_key) {
                let radius = model
                    .submeshes
                    .iter()
                    .map(|s| renderer.mesh_radius(s.mesh))
                    .fold(model.bounding_radius, f32::max);
                if !frustum.intersects_sphere(p.position, radius) {
                    continue;
                }
                let is_veg = (p.flags & (IDE_FLAG_IS_TREE | IDE_FLAG_IS_PALM)) != 0;
                let double_sided = is_veg || (p.flags & IDE_FLAG_DISABLE_BACKFACE_CULLING) != 0;
                let flag_transparent = (p.flags
                    & (IDE_FLAG_IS_GLASS_TYPE_1
                        | IDE_FLAG_IS_GLASS_TYPE_2
                        | IDE_FLAG_ADDITIVE
                        | IDE_FLAG_DRAW_LAST
                        | IDE_FLAG_NO_ZWRITE))
                    != 0;
                for sub in &model.submeshes {
                    let cut_only = (is_veg || sub.has_alpha_tex) && !flag_transparent;
                    let transparent =
                        (!cut_only && flag_transparent) || sub.material_color[3] < 0.98;
                    let alpha_cutoff = if sub.has_alpha_tex || double_sided {
                        0.35
                    } else if transparent {
                        0.02
                    } else {
                        0.0
                    };
                    custom_batches
                        .entry((sub.mesh.0, sub.texture_idx, transparent, double_sided))
                        .or_default()
                        .push(InstanceData::from_transform_lit(
                            p.position,
                            p.rotation,
                            Vec3::ONE,
                            sub.material_color,
                            0.0,
                            alpha_cutoff,
                        ));
                }
            } else if !p.is_lod
                && !self.world.loaded_from_original_data
                && frustum.intersects_sphere(p.position, 25.0)
            {
                box_instances.push(InstanceData::from_transform_lit(
                    p.position + Vec3::new(0.0, 0.0, 6.0),
                    p.rotation,
                    Vec3::new(14.0, 12.0, 12.0),
                    [0.72, 0.68, 0.60, 1.0],
                    0.0,
                    0.0,
                ));
            }
        }

        let mut batches = Vec::new();
        if !ground_instances.is_empty() {
            batches.push(RenderInstanceBatch {
                mesh: renderer.ground_mesh,
                texture_idx: renderer.checker_tex,
                instances: ground_instances,
                transparent: false,
                double_sided: false,
            });
        }
        if !box_instances.is_empty() {
            batches.push(RenderInstanceBatch {
                mesh: renderer.box_mesh,
                texture_idx: renderer.white_tex,
                instances: box_instances,
                transparent: false,
                double_sided: false,
            });
        }

        if in_exterior && let Some(water) = self.water_mesh {
            batches.push(RenderInstanceBatch {
                mesh: water,
                texture_idx: renderer.white_tex,
                instances: vec![InstanceData::from_transform_lit(
                    Vec3::ZERO,
                    Quat::IDENTITY,
                    Vec3::ONE,
                    [1.0, 1.0, 1.0, 1.0],
                    2.0,
                    0.0,
                )],
                transparent: true,
                double_sided: true,
            });
        }

        for ((mesh_idx, tex_idx, transparent, double_sided), instances) in custom_batches {
            batches.push(RenderInstanceBatch {
                mesh: GpuMeshHandle(mesh_idx),
                texture_idx: tex_idx,
                instances,
                transparent,
                double_sided,
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

        let decoded_radio_sample = self.audio.verify_first_radio_track_decodes();

        let mut skinned_moved_verts = 0usize;
        if let Some(img) = &self.gta3_archive
            && let Ok(bytes) = img.read_entry("fam1.dff")
            && let Ok(clump) = DffClump::parse(&bytes)
            && !self.ped_ifp_lib.is_empty()
            && let Some(skin) = clump.geometries.iter().find_map(|g| g.skin.as_ref())
        {
            let subs = CpuMeshData::dff_clump_submeshes(&clump);
            let poses = self.ped_ifp_lib.sample_clip("walk_civi", 0.25, true);
            let mats = compute_skin_matrices(&clump.frames, skin, &poses);
            let mut out_verts = Vec::new();
            for sub in &subs {
                sub.compute_skinned_vertices(&mats, &mut out_verts);
                for (a, b) in out_verts.iter().zip(&sub.cpu_mesh.vertices) {
                    if Vec3::from_array(a.position).distance_squared(Vec3::from_array(b.position))
                        > 1e-4
                    {
                        skinned_moved_verts += 1;
                    }
                }
            }
            anyhow::ensure!(
                skinned_moved_verts > 0,
                "fam1.dff ped.ifp skinning produced zero deformed vertices"
            );
        }

        let carls_paired_interior = self
            .world
            .nearby_enex(Vec3::new(2495.33, -1690.75, 13.78), 0, 2.0)
            .into_iter()
            .next()
            .and_then(|e| self.world.find_paired_enex(e, 0))
            .map(|d| d.target_interior);
        if !self.world.entry_exits.is_empty() {
            anyhow::ensure!(
                carls_paired_interior == Some(3),
                "Expected CARLS paired ENEX interior 3, got {:?}",
                carls_paired_interior
            );
        }

        info!(
            ticks = self.timestep.total_ticks,
            player_pos = ?self.player.position,
            zone = %self.world.zone_name_at(self.player.position),
            vehicles = self.vehicles.vehicles.len(),
            handling_entries = self.handling.as_ref().map_or(0, |h| h.vehicles.len()),
            audio_stations = self.audio.available_radio_count(),
            audio_sfx_clips = self.audio.loaded_sfx_count(),
            ifp_clips = self.ped_ifp_lib.clips.len(),
            enex_portals = self.world.entry_exits.len(),
            skinned_fam1_moved_verts = skinned_moved_verts,
            carls_paired_interior = ?carls_paired_interior,
            decoded_radio = ?decoded_radio_sample,
            "Smoke test checks passed"
        );
        Ok(())
    }
}

fn ped_model_name_for(model_id: u32) -> &'static str {
    match model_id {
        105 => "fam1",
        106 => "fam2",
        107 => "fam3",
        21 => "bmydj",
        280 => "lapd1",
        9 | 14 => "hmost",
        _ => "fam2",
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

fn vehicle_class_from_ide(vt: &str) -> VehicleClass {
    match vt.trim().to_ascii_lowercase().as_str() {
        "bike" => VehicleClass::Motorcycle,
        "bmx" => VehicleClass::Bicycle,
        "boat" => VehicleClass::Boat,
        "plane" => VehicleClass::Plane,
        "heli" => VehicleClass::Helicopter,
        _ => VehicleClass::Car,
    }
}

fn sample_sa_car_color(idx: usize) -> [f32; 3] {
    const PALETTE: [[f32; 3]; 8] = [
        [0.14, 0.36, 0.18], // Grove Forest Green
        [0.65, 0.14, 0.14], // LS Crimson
        [0.18, 0.32, 0.56], // Pacific Blue
        [0.72, 0.68, 0.58], // Desert Silver-Beige
        [0.24, 0.25, 0.27], // Gunmetal Gray
        [0.82, 0.62, 0.14], // Yellow Taxi/Taxi Gold
        [0.85, 0.85, 0.85], // Pearl White
        [0.36, 0.16, 0.42], // Ballas Purple
    ];
    PALETTE[idx % PALETTE.len()]
}

fn weather_from_sa_id(id: i32) -> WeatherType {
    match id {
        0 | 5 | 6 => WeatherType::ExtraSunnyLA,
        1..=3 => WeatherType::SunnyLA,
        10..=15 => WeatherType::ExtraSunnySmog,
        4 | 7 => WeatherType::Cloudy,
        8 | 16 => WeatherType::Rainy,
        9 => WeatherType::Foggy,
        19 => WeatherType::SandstormDesert,
        _ => WeatherType::SunnyLA,
    }
}

fn decode_txd_with_masks(bytes: &[u8]) -> Vec<(String, u32, u32, Vec<u8>, bool)> {
    let Ok(archive) = TxdArchive::parse(bytes) else {
        return Vec::new();
    };
    let by_name: HashMap<String, usize> = archive
        .textures
        .iter()
        .enumerate()
        .map(|(i, t)| (t.name.to_ascii_lowercase(), i))
        .collect();
    archive
        .textures
        .iter()
        .map(|tex| {
            let lower = tex.name.to_ascii_lowercase();
            let alpha_lower = tex.alpha_name.trim().to_ascii_lowercase();
            let rgba = if !alpha_lower.is_empty()
                && let Some(&mask_idx) = by_name.get(&alpha_lower)
            {
                let mask = &archive.textures[mask_idx];
                merge_rgba_alpha_mask(
                    &tex.rgba8,
                    tex.width,
                    tex.height,
                    &mask.rgba8,
                    mask.width,
                    mask.height,
                )
            } else {
                tex.rgba8.clone()
            };
            let has_alpha = rgba.as_chunks::<4>().0.iter().any(|px| px[3] < 250);
            (lower, tex.width, tex.height, rgba, has_alpha)
        })
        .collect()
}

fn load_ui_media(vfs: &GameDataRoot, gta3: Option<&ImgArchive>, ui_state: &mut GameUiState) {
    if let Ok(bytes) = vfs.read_bytes("models/txd/LOADSCS.TXD") {
        let entries = decode_txd_with_masks(&bytes);
        let chosen = entries
            .iter()
            .find(|(n, ..)| n == "loadsc0")
            .or_else(|| entries.iter().find(|(n, ..)| n == "title_pc_us"))
            .or_else(|| entries.iter().find(|(n, ..)| n == "title_pc_eu"))
            .or_else(|| entries.first());
        if let Some((_, w, h, rgba, _)) = chosen {
            ui_state.media.raw_backdrop = Some((*w, *h, rgba.clone()));
        }
    }

    if let Ok(bytes) = vfs.read_bytes("models/hud.txd") {
        for (name, w, h, rgba, _) in decode_txd_with_masks(&bytes) {
            ui_state.media.raw_hud_sprites.insert(name, (w, h, rgba));
        }
    }

    if let Some(img) = gta3 {
        let mut tiles_loaded = 0usize;
        for tile in 0..144usize {
            let entry = format!("radar{tile:02}.txd");
            if let Ok(bytes) = img.read_entry(&entry)
                && let Some((_, w, h, rgba, _)) = decode_txd_with_masks(&bytes).into_iter().next()
            {
                ui_state.media.raw_radar_tiles[tile] = Some((w, h, rgba));
                tiles_loaded += 1;
            }
        }
        info!(
            backdrop = ui_state.media.raw_backdrop.is_some(),
            hud_sprites = ui_state.media.raw_hud_sprites.len(),
            radar_tiles = tiles_loaded,
            "Loaded retail GTA:SA HUD, menu splash, and radar tiles"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gtasarust_gta_formats::ipl::IplEntryExit;

    #[test]
    fn enex_teleports_to_interior_and_triggers_services() {
        let cfg = EngineConfig {
            game_data_dir: None,
            ..Default::default()
        };
        let mut host = HostInfo::detect();
        host.save_dir = std::env::temp_dir().join("gtasarust_enex_test_saves");
        let mut sim = GameWorldState::initialize(cfg, host, true);
        sim.world.entry_exits = vec![
            IplEntryExit {
                entrance: [2495.33, -1690.75, 13.78],
                entrance_rot: 0.0,
                size: [1.4, 1.4, 8.0],
                exit: [2495.33, -1690.25, 13.78],
                exit_rot: 0.0,
                target_interior: 0,
                flags: 4,
                name: "CARLS".into(),
            },
            IplEntryExit {
                entrance: [2496.05, -1692.73, 1013.75],
                entrance_rot: std::f32::consts::PI,
                size: [2.0, 2.0, 8.0],
                exit: [2496.05, -1692.93, 1013.75],
                exit_rot: -180.0,
                target_interior: 3,
                flags: 0,
                name: "CARLS".into(),
            },
            IplEntryExit {
                entrance: [2419.95, -1509.8, 23.16],
                entrance_rot: 0.0,
                size: [1.5, 1.5, 8.0],
                exit: [2419.95, -1511.5, 23.16],
                exit_rot: 0.0,
                target_interior: 0,
                flags: 4,
                name: "FDCHICK".into(),
            },
            IplEntryExit {
                entrance: [365.67, -11.61, 1000.87],
                entrance_rot: 0.0,
                size: [2.0, 2.0, 8.0],
                exit: [365.67, -9.8, 1000.87],
                exit_rot: 0.0,
                target_interior: 9,
                flags: 0,
                name: "FDCHICK".into(),
            },
            IplEntryExit {
                entrance: [1368.35, -1279.06, 12.55],
                entrance_rot: 0.0,
                size: [1.5, 1.5, 8.0],
                exit: [1368.35, -1281.0, 12.55],
                exit_rot: 0.0,
                target_interior: 0,
                flags: 4,
                name: "AMMUN1".into(),
            },
            IplEntryExit {
                entrance: [286.15, -41.54, 1000.57],
                entrance_rot: 0.0,
                size: [2.0, 2.0, 8.0],
                exit: [286.15, -39.5, 1000.57],
                exit_rot: 0.0,
                target_interior: 1,
                flags: 0,
                name: "AMMUN1".into(),
            },
        ];

        // Enter Johnson House through CARLS ENEX
        sim.player.position = Vec3::new(2495.33, -1690.75, 13.78);
        sim.player.health = 45.0;
        sim.step_fixed_tick(1.0 / 60.0, &ActionFrameState::default());
        assert_eq!(sim.player.interior, 3);
        assert!((sim.player.position.z - 1013.75).abs() < 1.0);

        // Save roundtrip preserves interior = 3
        let saved = sim.to_save_data(2);
        assert_eq!(saved.interior, 3);

        // Safehouse service counter restores health
        let svc = sim.interior_service_pos.expect("CARLS service spot");
        sim.player.position = svc;
        sim.step_fixed_tick(1.0 / 60.0, &ActionFrameState::default());
        assert!((sim.player.health - sim.player.max_health).abs() < 1e-3);

        // Walk back to interior CARLS exit to return to grove street exterior
        sim.enex_cooldown = 0.0;
        sim.enex_lock_pos = None;
        sim.player.position = Vec3::new(2496.05, -1692.73, 1013.75);
        sim.step_fixed_tick(1.0 / 60.0, &ActionFrameState::default());
        assert_eq!(sim.player.interior, 0);

        // Enter Cluckin' Bell and buy food (-$10, +50 HP)
        sim.enex_cooldown = 0.0;
        sim.enex_lock_pos = None;
        sim.player.position = Vec3::new(2419.95, -1509.8, 23.16);
        sim.step_fixed_tick(1.0 / 60.0, &ActionFrameState::default());
        assert_eq!(sim.player.interior, 9);
        let fd_svc = sim.interior_service_pos.expect("FDCHICK service spot");
        sim.player.health = 30.0;
        sim.player.money = 100;
        sim.service_cooldown = 0.0;
        sim.player.position = fd_svc;
        sim.step_fixed_tick(1.0 / 60.0, &ActionFrameState::default());
        assert_eq!(sim.player.money, 90);
        assert!((sim.player.health - 80.0).abs() < 1e-3);

        // Exit Cluckin' Bell and enter Ammu-Nation to buy 9mm ammo (-$200, +50 reserve)
        sim.enex_cooldown = 0.0;
        sim.enex_lock_pos = None;
        sim.player.interior = 0;
        sim.player.position = Vec3::new(1368.35, -1279.06, 12.55);
        sim.step_fixed_tick(1.0 / 60.0, &ActionFrameState::default());
        assert_eq!(sim.player.interior, 1);
        let am_svc = sim.interior_service_pos.expect("AMMUN1 service spot");
        let before_ammo = sim.player.weapons.slots[2]
            .as_ref()
            .map_or(0, |w| w.reserve_ammo);
        sim.player.money = 350;
        sim.service_cooldown = 0.0;
        sim.player.position = am_svc;
        sim.step_fixed_tick(1.0 / 60.0, &ActionFrameState::default());
        assert_eq!(sim.player.money, 150);
        assert_eq!(
            sim.player.weapons.slots[2].as_ref().unwrap().reserve_ammo,
            before_ammo + 50
        );
    }

    #[test]
    fn scm_world_commands_and_decoupled_vehicle_camera_work() {
        let mut sim = GameWorldState::initialize(EngineConfig::default(), HostInfo::detect(), true);
        sim.scm.world_commands.push(ScmWorldCommand::SetTimeOfDay {
            hour: 22,
            minute: 15,
        });
        sim.scm
            .world_commands
            .push(ScmWorldCommand::ForceWeather(8));
        sim.scm.world_commands.push(ScmWorldCommand::CreatePickup {
            model_id: 1240,
            pickup_type: 2,
            pos: [2490.0, -1670.0, 13.5],
            handle: 1,
        });
        sim.drain_scm_world_commands();
        assert_eq!(sim.clock.hour, 22);
        assert_eq!(sim.clock.minute, 15);
        assert_eq!(sim.world.weather, WeatherType::Rainy);
        assert_eq!(sim.scm_pickups.len(), 1);

        let vid = sim.vehicles.vehicles[0].id;
        sim.player.control_mode = ControlMode::InVehicle;
        sim.player.current_vehicle = Some(vid);
        let init_cam_yaw = sim.player.camera_yaw;
        let act = ActionFrameState {
            look_delta: glam::Vec2::new(80.0, -20.0),
            ..Default::default()
        };
        sim.step_fixed_tick(1.0 / 60.0, &act);
        assert!((sim.player.camera_yaw - init_cam_yaw).abs() > 0.1);
    }
}
