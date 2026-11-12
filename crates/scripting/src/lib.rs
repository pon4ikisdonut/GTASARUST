//! Clean-room SCM bytecode virtual machine supporting both high-level script fibers
//! and compiled GTA: San Andreas `main.scm` chunk/opcode execution.

use gtasarust_gta_formats::scm::{
    ScmArg, ScmDecodedInstruction, ScmImage, decode_instruction_slice,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::{debug, trace};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScriptVal {
    Int(i32),
    Float(f32),
    Text(String),
}

impl Default for ScriptVal {
    fn default() -> Self {
        Self::Int(0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScriptInstruction {
    WaitMs(u32),
    Jump(usize),
    JumpIfFalse(usize),
    SetGlobalInt(usize, i32),
    AddPlayerMoney(i32),
    SetWantedLevel(u8),
    PrintMissionBanner(String),
    CompleteObjective(String),
    TerminateThread,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptThread {
    pub name: String,
    pub ip: usize,
    pub wait_seconds: f32,
    pub condition_result: bool,
    pub locals: [ScriptVal; 32],
    pub terminated: bool,
    pub program: Vec<ScriptInstruction>,
}

#[derive(Debug, Default)]
pub struct ScriptEffectQueue {
    pub cash_deltas: Vec<i32>,
    pub wanted_override: Option<u8>,
    pub banners: Vec<String>,
    pub completed_objectives: Vec<String>,
}

/// Code buffer a running SCM instruction pointer addresses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScmCodeSource {
    Main,
    Streamed(usize),
}

/// Active state for multi-instruction `SWITCH_START` / `SWITCH_CONTINUED` tables.
#[derive(Debug, Clone)]
pub struct PendingSwitch {
    pub value: i32,
    pub remaining: usize,
    pub default_label: Option<i32>,
}

/// Active GTA:SA `CRunningScript` execution fiber inside a loaded `ScmImage`.
#[derive(Debug, Clone)]
pub struct ScmBytecodeThread {
    pub name: String,
    pub ip: usize,
    pub base_ip: usize,
    pub source: ScmCodeSource,
    pub home_source: ScmCodeSource,
    pub stack: Vec<(ScmCodeSource, usize)>,
    pub locals: Vec<i32>,
    pub wait_seconds: f32,
    pub cond_result: bool,
    pub and_or_state: u8,
    pub is_mission: bool,
    pub terminated: bool,
    pub instructions_executed: u64,
    pub scene_skip_ip: Option<(ScmCodeSource, usize)>,
    pub pending_switch: Option<PendingSwitch>,
}

impl ScmBytecodeThread {
    fn new(name: impl Into<String>, entry_ip: usize, is_mission: bool) -> Self {
        Self::with_source(name, entry_ip, entry_ip, ScmCodeSource::Main, is_mission)
    }

    fn with_source(
        name: impl Into<String>,
        entry_ip: usize,
        base_ip: usize,
        source: ScmCodeSource,
        is_mission: bool,
    ) -> Self {
        Self {
            name: name.into(),
            ip: entry_ip,
            base_ip,
            source,
            home_source: source,
            stack: Vec::with_capacity(16),
            locals: vec![0i32; if is_mission { 1024 } else { 256 }],
            wait_seconds: 0.0,
            cond_result: false,
            and_or_state: 0,
            is_mission,
            terminated: false,
            instructions_executed: 0,
            scene_skip_ip: None,
            pending_switch: None,
        }
    }
}

/// Read-only snapshot of live world/cutscene/input state evaluated by SCM conditional opcodes.
#[derive(Debug, Clone)]
pub struct ScmWorldSnapshot {
    pub player_pos: [f32; 3],
    pub player_heading_deg: f32,
    pub player_health: i32,
    pub player_in_car: bool,
    pub player_car_handle: Option<i32>,
    pub player_car_model: Option<i32>,
    pub player_stopped: bool,
    pub player_dead: bool,
    pub player_control: bool,
    pub fade_active: bool,
    pub cutscene_loaded: bool,
    pub cutscene_time_ms: i32,
    pub cutscene_finished: bool,
    pub cutscene_skipped: bool,
    pub buttons_pressed: [bool; 24],
    pub active_car_pos: HashMap<i32, [f32; 3]>,
    pub active_char_pos: HashMap<i32, [f32; 3]>,
    pub dead_chars: HashSet<i32>,
    pub dead_cars: HashSet<i32>,
    pub garage_open: HashMap<String, bool>,
}

impl Default for ScmWorldSnapshot {
    fn default() -> Self {
        Self {
            player_pos: [0.0; 3],
            player_heading_deg: 0.0,
            player_health: 100,
            player_in_car: false,
            player_car_handle: None,
            player_car_model: None,
            player_stopped: true,
            player_dead: false,
            player_control: true,
            fade_active: false,
            cutscene_loaded: false,
            cutscene_time_ms: 0,
            cutscene_finished: false,
            cutscene_skipped: false,
            buttons_pressed: [false; 24],
            active_car_pos: HashMap::new(),
            active_char_pos: HashMap::new(),
            dead_chars: HashSet::new(),
            dead_cars: HashSet::new(),
            garage_open: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScmBlipKind {
    Coord,
    Sprite(i32),
}

/// Side-effect emitted by compiled `main.scm` opcodes for the game simulation to apply.
#[derive(Debug, Clone)]
pub enum ScmWorldCommand {
    SetTimeOfDay {
        hour: u8,
        minute: u8,
    },
    ForceWeather(i32),
    DoFade {
        duration_ms: i32,
        out: bool,
    },
    CreatePlayer {
        player_idx: i32,
        pos: [f32; 3],
        handle: i32,
    },
    CreateCar {
        model_id: i32,
        pos: [f32; 3],
        handle: i32,
    },
    CreateChar {
        ped_type: i32,
        model_id: i32,
        pos: [f32; 3],
        handle: i32,
    },
    CreatePickup {
        model_id: i32,
        pickup_type: i32,
        pos: [f32; 3],
        handle: i32,
    },
    CreateCarGenerator {
        pos: [f32; 3],
        angle_deg: f32,
        model_id: i32,
    },
    AddBlip {
        kind: ScmBlipKind,
        pos: [f32; 3],
        handle: i32,
    },
    RemoveBlip(i32),
    PrintText {
        key: String,
        duration_ms: i32,
        style: i32,
    },
    ClearPrints,
    PrintHelp {
        key: String,
    },
    ClearHelp,
    LoadScene([f32; 3]),
    DisplayHud(bool),
    DisplayRadar(bool),
    RequestModel(i32),
    SetCharCoordinates {
        handle: i32,
        pos: [f32; 3],
    },
    SetCharHeading {
        handle: i32,
        heading_deg: f32,
    },
    SetCarCoordinates {
        handle: i32,
        pos: [f32; 3],
    },
    SetPlayerControl {
        player_idx: i32,
        enabled: bool,
    },
    ToggleWidescreen(bool),
    AlterWantedLevel {
        player_idx: i32,
        stars: u8,
    },
    ClearWantedLevel {
        player_idx: i32,
    },
    SetMaxWantedLevel(u8),
    GiveWeaponToChar {
        handle: i32,
        weapon_type: i32,
        ammo: i32,
    },
    AddArmourToChar {
        handle: i32,
        amount: i32,
    },
    SetCharHealth {
        handle: i32,
        health: i32,
    },
    SetFixedCameraPosition {
        pos: [f32; 3],
        rot_offset: [f32; 3],
    },
    PointCameraAtPoint {
        pos: [f32; 3],
        switch_style: i32,
    },
    PointCameraAtChar {
        handle: i32,
        mode: i32,
        switch_style: i32,
    },
    PointCameraAtCar {
        handle: i32,
        mode: i32,
        switch_style: i32,
    },
    RestoreCamera {
        jumpcut: bool,
    },
    SetCameraBehindPlayer,
    LoadCutscene(String),
    SetCutsceneOffset([f32; 3]),
    StartCutscene,
    ClearCutscene,
    ActivateGarage {
        name: String,
        active: bool,
    },
    ReportMissionAudioEvent {
        pos: Option<[f32; 3]>,
        sound_id: i32,
    },
    CreateCheckpoint {
        kind: i32,
        pos: [f32; 3],
        point_at: [f32; 3],
        radius: f32,
        handle: i32,
    },
    DeleteCheckpoint(i32),
}

#[derive(Debug)]
pub struct ScmVm {
    pub globals: HashMap<usize, ScriptVal>,
    pub threads: Vec<ScriptThread>,
    pub pending_effects: ScriptEffectQueue,
    pub scm_image: Option<ScmImage>,
    pub scm_globals: Vec<u8>,
    pub streamed_scripts: HashMap<usize, Vec<u8>>,
    pub bytecode_threads: Vec<ScmBytecodeThread>,
    pub world_commands: Vec<ScmWorldCommand>,
    pub world_snapshot: ScmWorldSnapshot,
    pub elapsed_ms: i32,
    pub last_dt: f32,
    pub player_actor_handle: i32,
    beat_track_status: i32,
    spawned_pickups: HashMap<i32, [f32; 3]>,
    rng_state: u64,
    next_handle: i32,
}

impl Default for ScmVm {
    fn default() -> Self {
        Self {
            globals: HashMap::new(),
            threads: Vec::new(),
            pending_effects: ScriptEffectQueue::default(),
            scm_image: None,
            scm_globals: Vec::new(),
            streamed_scripts: HashMap::new(),
            bytecode_threads: Vec::new(),
            world_commands: Vec::new(),
            world_snapshot: ScmWorldSnapshot::default(),
            elapsed_ms: 0,
            last_dt: 1.0 / 60.0,
            player_actor_handle: 0,
            beat_track_status: 0,
            spawned_pickups: HashMap::new(),
            rng_state: 0x9E37_79B9_7F4A_7C15,
            next_handle: 1,
        }
    }
}

impl ScmVm {
    pub fn spawn_thread(&mut self, name: impl Into<String>, program: Vec<ScriptInstruction>) {
        self.threads.push(ScriptThread {
            name: name.into(),
            ip: 0,
            wait_seconds: 0.0,
            condition_result: true,
            locals: std::array::from_fn(|_| ScriptVal::Int(0)),
            terminated: false,
            program,
        });
    }

    /// Attach a compiled GTA:SA `ScmImage`, initialize global variable memory, and boot `MAIN`.
    pub fn load_scm_image(&mut self, image: ScmImage) {
        let mut globals_mem = vec![0u8; image.globals_end.max(65536)];
        let copy_len = image
            .globals_end
            .min(image.data.len())
            .min(globals_mem.len());
        globals_mem[..copy_len].copy_from_slice(&image.data[..copy_len]);
        let entry_ip = image.main_offset;
        self.scm_image = Some(image);
        self.scm_globals = globals_mem;
        self.streamed_scripts.clear();
        self.spawned_pickups.clear();
        self.player_actor_handle = 0;
        self.bytecode_threads.clear();
        self.bytecode_threads
            .push(ScmBytecodeThread::new("main", entry_ip, false));
        // Execute initial boot commands immediately so world time / player spawn are queued.
        self.step_bytecode_threads(0.0);
    }

    /// Register raw compiled bytecode for a `script.img` entry matching `scm_image.streamed_scripts`.
    /// Returns the assigned streamed script ID if matched.
    pub fn load_streamed_script_bytes(
        &mut self,
        name_or_file: &str,
        bytes: Vec<u8>,
    ) -> Option<usize> {
        let stem = name_or_file
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or(name_or_file)
            .strip_suffix(".scm")
            .or_else(|| {
                name_or_file
                    .rsplit(['/', '\\'])
                    .next()
                    .unwrap_or(name_or_file)
                    .strip_suffix(".SCM")
            })
            .unwrap_or(name_or_file);
        let idx = self
            .scm_image
            .as_ref()?
            .streamed_scripts
            .iter()
            .position(|s| s.name.eq_ignore_ascii_case(stem))?;
        self.streamed_scripts.insert(idx, bytes);
        Some(idx)
    }

    pub fn tick(&mut self, dt: f32) {
        for thread in &mut self.threads {
            if thread.terminated {
                continue;
            }
            if thread.wait_seconds > 0.0 {
                thread.wait_seconds -= dt;
                if thread.wait_seconds > 0.0 {
                    continue;
                }
            }

            for _ in 0..256 {
                let Some(instr) = thread.program.get(thread.ip).cloned() else {
                    thread.terminated = true;
                    break;
                };
                match instr {
                    ScriptInstruction::WaitMs(ms) => {
                        thread.ip += 1;
                        thread.wait_seconds = (ms as f32) / 1000.0;
                        break;
                    }
                    ScriptInstruction::Jump(target) => {
                        thread.ip = target;
                    }
                    ScriptInstruction::JumpIfFalse(target) => {
                        if !thread.condition_result {
                            thread.ip = target;
                        } else {
                            thread.ip += 1;
                        }
                    }
                    ScriptInstruction::SetGlobalInt(idx, v) => {
                        self.globals.insert(idx, ScriptVal::Int(v));
                        thread.ip += 1;
                    }
                    ScriptInstruction::AddPlayerMoney(delta) => {
                        self.pending_effects.cash_deltas.push(delta);
                        thread.ip += 1;
                    }
                    ScriptInstruction::SetWantedLevel(stars) => {
                        self.pending_effects.wanted_override = Some(stars);
                        thread.ip += 1;
                    }
                    ScriptInstruction::PrintMissionBanner(text) => {
                        self.pending_effects.banners.push(text);
                        thread.ip += 1;
                    }
                    ScriptInstruction::CompleteObjective(id) => {
                        self.pending_effects.completed_objectives.push(id);
                        thread.ip += 1;
                    }
                    ScriptInstruction::TerminateThread => {
                        thread.terminated = true;
                        break;
                    }
                }
            }
        }
        self.threads.retain(|t| !t.terminated);
        self.step_bytecode_threads(dt);
    }

    fn step_bytecode_threads(&mut self, dt: f32) {
        if self.scm_image.is_none() || self.bytecode_threads.is_empty() {
            return;
        }

        let dt_pos = dt.max(0.0);
        let dt_ms = (dt_pos * 1000.0).round() as i32;
        if dt_pos > 0.0 {
            self.last_dt = dt_pos;
            self.elapsed_ms = self.elapsed_ms.wrapping_add(dt_ms.max(1));
        }

        let mut newly_spawned: Vec<ScmBytecodeThread> = Vec::new();
        let thread_count = self.bytecode_threads.len();

        for idx in 0..thread_count {
            let mut thread = self.bytecode_threads[idx].clone();
            if thread.terminated {
                continue;
            }
            if dt_pos > 0.0 {
                // SA `CRunningScript` increments local script timers TIMERA (32@) and TIMERB (33@).
                let add_ms = dt_ms.max(1);
                if thread.locals.len() > 33 {
                    thread.locals[32] = thread.locals[32].saturating_add(add_ms);
                    thread.locals[33] = thread.locals[33].saturating_add(add_ms);
                }
            }
            if let Some((skip_src, skip_ip)) = thread.scene_skip_ip
                && self.world_snapshot.cutscene_skipped
            {
                thread.scene_skip_ip = None;
                thread.source = skip_src;
                thread.ip = skip_ip;
                thread.wait_seconds = 0.0;
            }
            if thread.wait_seconds > 0.0 {
                thread.wait_seconds -= dt_pos;
                if thread.wait_seconds > 0.0 {
                    self.bytecode_threads[idx] = thread;
                    continue;
                }
                thread.wait_seconds = 0.0;
            }

            for _ in 0..8192 {
                let decoded = {
                    let Some(image) = &self.scm_image else {
                        break;
                    };
                    match thread.source {
                        ScmCodeSource::Main => {
                            decode_instruction_slice(&image.data, thread.ip).ok()
                        }
                        ScmCodeSource::Streamed(sid) => self
                            .streamed_scripts
                            .get(&sid)
                            .and_then(|buf| decode_instruction_slice(buf, thread.ip).ok()),
                    }
                };
                let Some(instr) = decoded else {
                    debug!(
                        thread = %thread.name,
                        ip = thread.ip,
                        source = ?thread.source,
                        "Suspending SCM bytecode thread on decode boundary or unmapped opcode"
                    );
                    thread.terminated = true;
                    break;
                };
                thread.instructions_executed += 1;
                if !self.execute_bytecode_instruction(&mut thread, &instr, &mut newly_spawned) {
                    break;
                }
                if thread.terminated {
                    break;
                }
            }

            self.bytecode_threads[idx] = thread;
        }

        self.bytecode_threads.extend(newly_spawned);
        self.bytecode_threads.retain(|t| !t.terminated);
    }

    fn alloc_handle(&mut self) -> i32 {
        let h = self.next_handle;
        self.next_handle = self.next_handle.wrapping_add(1).max(1);
        h
    }

    fn next_random_u64(&mut self) -> u64 {
        let mut x = self.rng_state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng_state = x;
        x
    }

    fn resolve_thread_jump(
        &self,
        thread: &ScmBytecodeThread,
        target: i32,
    ) -> (ScmCodeSource, usize) {
        if target >= 0 {
            (ScmCodeSource::Main, target as usize)
        } else {
            (
                thread.home_source,
                thread
                    .base_ip
                    .saturating_add(target.unsigned_abs() as usize),
            )
        }
    }

    fn is_player_actor(&self, handle: i32) -> bool {
        self.player_actor_handle > 0 && handle == self.player_actor_handle
    }

    fn entity_pos(&self, handle: i32, is_car: bool) -> Option<[f32; 3]> {
        if !is_car && self.is_player_actor(handle) {
            return Some(self.world_snapshot.player_pos);
        }
        if is_car {
            self.world_snapshot.active_car_pos.get(&handle).copied()
        } else {
            self.world_snapshot.active_char_pos.get(&handle).copied()
        }
    }

    /// Execute one decoded SCM instruction; returns `true` to continue executing this frame.
    fn execute_bytecode_instruction(
        &mut self,
        thread: &mut ScmBytecodeThread,
        instr: &ScmDecodedInstruction,
        spawned: &mut Vec<ScmBytecodeThread>,
    ) -> bool {
        let args = &instr.args;
        let next = instr.next_offset;

        match instr.opcode {
            0x0000 => {
                // NOP
                thread.ip = next;
            }
            0x0001 => {
                // WAIT
                let ms = args
                    .first()
                    .map(|a| self.eval_i32(thread, a))
                    .unwrap_or(0)
                    .max(0);
                thread.ip = next;
                thread.wait_seconds = (ms as f32) / 1000.0;
                return false;
            }
            0x0002 => {
                // GOTO
                let target = args.first().map(|a| self.eval_i32(thread, a)).unwrap_or(0);
                let (src, ip) = self.resolve_thread_jump(thread, target);
                thread.source = src;
                thread.ip = ip;
            }
            0x0004 | 0x0006 | 0x0084 | 0x0085 | 0x008A | 0x008B | 0x04AE | 0x04AF => {
                // SET int
                if args.len() >= 2 {
                    let v = self.eval_i32(thread, &args[1]);
                    self.write_i32(thread, &args[0], v);
                }
                thread.ip = next;
            }
            0x0005 | 0x0007 | 0x0086 | 0x0087 | 0x0088 | 0x0089 => {
                // SET float
                if args.len() >= 2 {
                    let v = self.eval_f32(thread, &args[1]);
                    self.write_f32(thread, &args[0], v);
                }
                thread.ip = next;
            }
            0x008C | 0x008E | 0x0090 | 0x0092 => {
                // CSET int from float
                if args.len() >= 2 {
                    let v = self.eval_f32(thread, &args[1]) as i32;
                    self.write_i32(thread, &args[0], v);
                }
                thread.ip = next;
            }
            0x008D | 0x008F | 0x0091 | 0x0093 => {
                // CSET float from int
                if args.len() >= 2 {
                    let v = self.eval_i32(thread, &args[1]) as f32;
                    self.write_f32(thread, &args[0], v);
                }
                thread.ip = next;
            }
            0x0078..=0x007D => {
                // ADD_TIMED float
                if args.len() >= 2 {
                    let step = self.last_dt * 50.0;
                    let v =
                        self.eval_f32(thread, &args[0]) + self.eval_f32(thread, &args[1]) * step;
                    self.write_f32(thread, &args[0], v);
                }
                thread.ip = next;
            }
            0x007E..=0x0083 => {
                // SUB_TIMED float
                if args.len() >= 2 {
                    let step = self.last_dt * 50.0;
                    let v =
                        self.eval_f32(thread, &args[0]) - self.eval_f32(thread, &args[1]) * step;
                    self.write_f32(thread, &args[0], v);
                }
                thread.ip = next;
            }
            0x0094 | 0x0095 => {
                // ABS int
                if let Some(a) = args.first() {
                    let v = self.eval_i32(thread, a).saturating_abs();
                    self.write_i32(thread, a, v);
                }
                thread.ip = next;
            }
            0x0096 | 0x0097 => {
                // ABS float
                if let Some(a) = args.first() {
                    let v = self.eval_f32(thread, a).abs();
                    self.write_f32(thread, a, v);
                }
                thread.ip = next;
            }
            0x0098 => {
                // GENERATE_RANDOM_FLOAT(out)
                if let Some(out) = args.first() {
                    let r = ((self.next_random_u64() >> 40) as f32) / ((1u64 << 24) as f32);
                    self.write_f32(thread, out, r);
                }
                thread.ip = next;
            }
            0x0099 => {
                // GENERATE_RANDOM_INT(out)
                if let Some(out) = args.first() {
                    let r = (self.next_random_u64() & 0x7FFF_FFFF) as i32;
                    self.write_i32(thread, out, r);
                }
                thread.ip = next;
            }
            0x0008 | 0x000A | 0x0058 | 0x005A | 0x005C | 0x005E => {
                // ADD int
                if args.len() >= 2 {
                    let v = self
                        .eval_i32(thread, &args[0])
                        .saturating_add(self.eval_i32(thread, &args[1]));
                    self.write_i32(thread, &args[0], v);
                }
                thread.ip = next;
            }
            0x0009 | 0x000B | 0x0059 | 0x005B | 0x005D | 0x005F => {
                // ADD float
                if args.len() >= 2 {
                    let v = self.eval_f32(thread, &args[0]) + self.eval_f32(thread, &args[1]);
                    self.write_f32(thread, &args[0], v);
                }
                thread.ip = next;
            }
            0x000C | 0x000E | 0x0060 | 0x0062 | 0x0064 | 0x0066 => {
                // SUB int
                if args.len() >= 2 {
                    let v = self
                        .eval_i32(thread, &args[0])
                        .saturating_sub(self.eval_i32(thread, &args[1]));
                    self.write_i32(thread, &args[0], v);
                }
                thread.ip = next;
            }
            0x000D | 0x000F | 0x0061 | 0x0063 | 0x0065 | 0x0067 => {
                // SUB float
                if args.len() >= 2 {
                    let v = self.eval_f32(thread, &args[0]) - self.eval_f32(thread, &args[1]);
                    self.write_f32(thread, &args[0], v);
                }
                thread.ip = next;
            }
            0x0010 | 0x0012 | 0x0068 | 0x006A | 0x006C | 0x006E => {
                // MULT int
                if args.len() >= 2 {
                    let v = self
                        .eval_i32(thread, &args[0])
                        .saturating_mul(self.eval_i32(thread, &args[1]));
                    self.write_i32(thread, &args[0], v);
                }
                thread.ip = next;
            }
            0x0011 | 0x0013 | 0x0069 | 0x006B | 0x006D | 0x006F => {
                // MULT float
                if args.len() >= 2 {
                    let v = self.eval_f32(thread, &args[0]) * self.eval_f32(thread, &args[1]);
                    self.write_f32(thread, &args[0], v);
                }
                thread.ip = next;
            }
            0x0014 | 0x0016 | 0x0070 | 0x0072 | 0x0074 | 0x0076 => {
                // DIV int
                if args.len() >= 2 {
                    let rhs = self.eval_i32(thread, &args[1]);
                    if rhs != 0 {
                        let v = self.eval_i32(thread, &args[0]) / rhs;
                        self.write_i32(thread, &args[0], v);
                    }
                }
                thread.ip = next;
            }
            0x0015 | 0x0017 | 0x0071 | 0x0073 | 0x0075 | 0x0077 => {
                // DIV float
                if args.len() >= 2 {
                    let rhs = self.eval_f32(thread, &args[1]);
                    if rhs.abs() > 1e-8 {
                        let v = self.eval_f32(thread, &args[0]) / rhs;
                        self.write_f32(thread, &args[0], v);
                    }
                }
                thread.ip = next;
            }
            0x0018..=0x001F => {
                // int >
                let cond = args.len() >= 2
                    && self.eval_i32(thread, &args[0]) > self.eval_i32(thread, &args[1]);
                update_compare_flag(thread, cond, instr.not_flag);
                thread.ip = next;
            }
            0x0020..=0x0027 => {
                // float >
                let cond = args.len() >= 2
                    && self.eval_f32(thread, &args[0]) > self.eval_f32(thread, &args[1]);
                update_compare_flag(thread, cond, instr.not_flag);
                thread.ip = next;
            }
            0x0028..=0x002F => {
                // int >=
                let cond = args.len() >= 2
                    && self.eval_i32(thread, &args[0]) >= self.eval_i32(thread, &args[1]);
                update_compare_flag(thread, cond, instr.not_flag);
                thread.ip = next;
            }
            0x0030..=0x0037 => {
                // float >=
                let cond = args.len() >= 2
                    && self.eval_f32(thread, &args[0]) >= self.eval_f32(thread, &args[1]);
                update_compare_flag(thread, cond, instr.not_flag);
                thread.ip = next;
            }
            0x0038..=0x003C | 0x04A3 | 0x04A4 => {
                // int ==
                let cond = args.len() >= 2
                    && self.eval_i32(thread, &args[0]) == self.eval_i32(thread, &args[1]);
                update_compare_flag(thread, cond, instr.not_flag);
                thread.ip = next;
            }
            0x0042..=0x0046 => {
                // float ==
                let cond = args.len() >= 2
                    && (self.eval_f32(thread, &args[0]) - self.eval_f32(thread, &args[1])).abs()
                        < 1e-5;
                update_compare_flag(thread, cond, instr.not_flag);
                thread.ip = next;
            }
            0x004C => {
                // GOTO_IF_TRUE
                let target = args.first().map(|a| self.eval_i32(thread, a)).unwrap_or(0);
                if thread.cond_result {
                    let (src, ip) = self.resolve_thread_jump(thread, target);
                    thread.source = src;
                    thread.ip = ip;
                } else {
                    thread.ip = next;
                }
            }
            0x004D => {
                // GOTO_IF_FALSE
                let target = args.first().map(|a| self.eval_i32(thread, a)).unwrap_or(0);
                if !thread.cond_result {
                    let (src, ip) = self.resolve_thread_jump(thread, target);
                    thread.source = src;
                    thread.ip = ip;
                } else {
                    thread.ip = next;
                }
            }
            0x004E => {
                // TERMINATE_THIS_SCRIPT
                thread.terminated = true;
                return false;
            }
            0x004F => {
                // START_NEW_SCRIPT(target, ...args)
                if let Some(first) = args.first() {
                    let target = self.eval_i32(thread, first);
                    let (src, entry_ip) = self.resolve_thread_jump(thread, target);
                    let base_ip = if target < 0 { thread.base_ip } else { entry_ip };
                    let mut child = ScmBytecodeThread::with_source(
                        format!("scr_{entry_ip:05x}"),
                        entry_ip,
                        base_ip,
                        src,
                        thread.is_mission,
                    );
                    for (slot, arg) in args.iter().skip(1).enumerate() {
                        if slot < child.locals.len() {
                            child.locals[slot] = self.eval_raw_word(thread, arg);
                        }
                    }
                    spawned.push(child);
                }
                thread.ip = next;
            }
            0x0050 => {
                // GOSUB
                if let Some(first) = args.first() {
                    let target = self.eval_i32(thread, first);
                    thread.stack.push((thread.source, next));
                    let (src, ip) = self.resolve_thread_jump(thread, target);
                    thread.source = src;
                    thread.ip = ip;
                } else {
                    thread.ip = next;
                }
            }
            0x0051 => {
                // RETURN
                if let Some((ret_src, ret_ip)) = thread.stack.pop() {
                    thread.source = ret_src;
                    thread.ip = ret_ip;
                } else {
                    thread.terminated = true;
                    return false;
                }
            }
            0x0053 => {
                // CREATE_PLAYER(idx, x, y, z, out_handle)
                if args.len() >= 5 {
                    let player_idx = self.eval_i32(thread, &args[0]);
                    let pos = [
                        self.eval_f32(thread, &args[1]),
                        self.eval_f32(thread, &args[2]),
                        self.eval_f32(thread, &args[3]),
                    ];
                    let handle = self.alloc_handle();
                    if player_idx == 0 && self.player_actor_handle == 0 {
                        self.player_actor_handle = handle;
                    }
                    self.write_i32(thread, &args[4], handle);
                    self.world_commands.push(ScmWorldCommand::CreatePlayer {
                        player_idx,
                        pos,
                        handle,
                    });
                }
                thread.ip = next;
            }
            0x0054 => {
                // GET_PLAYER_COORDINATES(player_idx, out_x, out_y, out_z)
                if args.len() >= 4 {
                    let pos = self.world_snapshot.player_pos;
                    self.write_f32(thread, &args[1], pos[0]);
                    self.write_f32(thread, &args[2], pos[1]);
                    self.write_f32(thread, &args[3], pos[2]);
                }
                thread.ip = next;
            }
            0x0055 => {
                // SET_PLAYER_COORDINATES(player_idx, x, y, z)
                if args.len() >= 4 {
                    let pos = [
                        self.eval_f32(thread, &args[1]),
                        self.eval_f32(thread, &args[2]),
                        self.eval_f32(thread, &args[3]),
                    ];
                    self.world_commands
                        .push(ScmWorldCommand::SetCharCoordinates {
                            handle: self.player_actor_handle.max(1),
                            pos,
                        });
                }
                thread.ip = next;
            }
            0x009A => {
                // CREATE_CHAR(ped_type, model_id, x, y, z, out_handle)
                if args.len() >= 6 {
                    let ped_type = self.eval_i32(thread, &args[0]);
                    let model_id = self.eval_i32(thread, &args[1]);
                    let pos = [
                        self.eval_f32(thread, &args[2]),
                        self.eval_f32(thread, &args[3]),
                        self.eval_f32(thread, &args[4]),
                    ];
                    let handle = self.alloc_handle();
                    self.write_i32(thread, &args[5], handle);
                    self.world_commands.push(ScmWorldCommand::CreateChar {
                        ped_type,
                        model_id,
                        pos,
                        handle,
                    });
                }
                thread.ip = next;
            }
            0x00A0 => {
                // GET_CHAR_COORDINATES(char, out_x, out_y, out_z)
                if args.len() >= 4 {
                    let handle = self.eval_i32(thread, &args[0]);
                    let pos = self
                        .entity_pos(handle, false)
                        .unwrap_or(self.world_snapshot.player_pos);
                    self.write_f32(thread, &args[1], pos[0]);
                    self.write_f32(thread, &args[2], pos[1]);
                    self.write_f32(thread, &args[3], pos[2]);
                }
                thread.ip = next;
            }
            0x00A1 => {
                // SET_CHAR_COORDINATES(char, x, y, z)
                if args.len() >= 4 {
                    let handle = self.eval_i32(thread, &args[0]);
                    let pos = [
                        self.eval_f32(thread, &args[1]),
                        self.eval_f32(thread, &args[2]),
                        self.eval_f32(thread, &args[3]),
                    ];
                    self.world_commands
                        .push(ScmWorldCommand::SetCharCoordinates { handle, pos });
                }
                thread.ip = next;
            }
            0x00A3 | 0x01A3 => {
                // IS_CHAR_IN_AREA_2D / IS_CHAR_STOPPED_IN_AREA_2D
                let cond = if args.len() >= 5 {
                    let handle = self.eval_i32(thread, &args[0]);
                    let stopped_ok = instr.opcode == 0x00A3
                        || !self.is_player_actor(handle)
                        || self.world_snapshot.player_stopped;
                    stopped_ok
                        && self.entity_pos(handle, false).is_some_and(|p| {
                            in_area_2d(
                                p,
                                self.eval_f32(thread, &args[1]),
                                self.eval_f32(thread, &args[2]),
                                self.eval_f32(thread, &args[3]),
                                self.eval_f32(thread, &args[4]),
                            )
                        })
                } else {
                    false
                };
                update_compare_flag(thread, cond, instr.not_flag);
                thread.ip = next;
            }
            0x00A4 | 0x01A8 => {
                // IS_CHAR_IN_AREA_3D / IS_CHAR_STOPPED_IN_AREA_3D
                let cond = if args.len() >= 7 {
                    let handle = self.eval_i32(thread, &args[0]);
                    let stopped_ok = instr.opcode == 0x00A4
                        || !self.is_player_actor(handle)
                        || self.world_snapshot.player_stopped;
                    stopped_ok
                        && self.entity_pos(handle, false).is_some_and(|p| {
                            in_area_3d(
                                p,
                                self.eval_f32(thread, &args[1]),
                                self.eval_f32(thread, &args[2]),
                                self.eval_f32(thread, &args[3]),
                                self.eval_f32(thread, &args[4]),
                                self.eval_f32(thread, &args[5]),
                                self.eval_f32(thread, &args[6]),
                            )
                        })
                } else {
                    false
                };
                update_compare_flag(thread, cond, instr.not_flag);
                thread.ip = next;
            }
            0x00A5 => {
                // CREATE_CAR(model_id, x, y, z, out_handle)
                if args.len() >= 5 {
                    let model_id = self.eval_i32(thread, &args[0]);
                    let pos = [
                        self.eval_f32(thread, &args[1]),
                        self.eval_f32(thread, &args[2]),
                        self.eval_f32(thread, &args[3]),
                    ];
                    let handle = self.alloc_handle();
                    self.write_i32(thread, &args[4], handle);
                    self.world_commands.push(ScmWorldCommand::CreateCar {
                        model_id,
                        pos,
                        handle,
                    });
                }
                thread.ip = next;
            }
            0x00AA => {
                // GET_CAR_COORDINATES(car, out_x, out_y, out_z)
                if args.len() >= 4 {
                    let handle = self.eval_i32(thread, &args[0]);
                    if let Some(pos) = self.entity_pos(handle, true) {
                        self.write_f32(thread, &args[1], pos[0]);
                        self.write_f32(thread, &args[2], pos[1]);
                        self.write_f32(thread, &args[3], pos[2]);
                    }
                }
                thread.ip = next;
            }
            0x00AB => {
                // SET_CAR_COORDINATES(car, x, y, z)
                if args.len() >= 4 {
                    let handle = self.eval_i32(thread, &args[0]);
                    let pos = [
                        self.eval_f32(thread, &args[1]),
                        self.eval_f32(thread, &args[2]),
                        self.eval_f32(thread, &args[3]),
                    ];
                    self.world_commands
                        .push(ScmWorldCommand::SetCarCoordinates { handle, pos });
                }
                thread.ip = next;
            }
            0x00B0 => {
                // IS_CAR_IN_AREA_2D
                let cond = if args.len() >= 5 {
                    let handle = self.eval_i32(thread, &args[0]);
                    self.entity_pos(handle, true).is_some_and(|p| {
                        in_area_2d(
                            p,
                            self.eval_f32(thread, &args[1]),
                            self.eval_f32(thread, &args[2]),
                            self.eval_f32(thread, &args[3]),
                            self.eval_f32(thread, &args[4]),
                        )
                    })
                } else {
                    false
                };
                update_compare_flag(thread, cond, instr.not_flag);
                thread.ip = next;
            }
            0x00B1 => {
                // IS_CAR_IN_AREA_3D
                let cond = if args.len() >= 7 {
                    let handle = self.eval_i32(thread, &args[0]);
                    self.entity_pos(handle, true).is_some_and(|p| {
                        in_area_3d(
                            p,
                            self.eval_f32(thread, &args[1]),
                            self.eval_f32(thread, &args[2]),
                            self.eval_f32(thread, &args[3]),
                            self.eval_f32(thread, &args[4]),
                            self.eval_f32(thread, &args[5]),
                            self.eval_f32(thread, &args[6]),
                        )
                    })
                } else {
                    false
                };
                update_compare_flag(thread, cond, instr.not_flag);
                thread.ip = next;
            }
            0x00BA..=0x00BC => {
                // PRINT_BIG / PRINT / PRINT_NOW
                if args.len() >= 3 {
                    let key = args[0].as_str_literal().unwrap_or_default().to_string();
                    let duration_ms = self.eval_i32(thread, &args[1]);
                    let style = self.eval_i32(thread, &args[2]);
                    if !key.is_empty() {
                        self.world_commands.push(ScmWorldCommand::PrintText {
                            key,
                            duration_ms,
                            style,
                        });
                    }
                }
                thread.ip = next;
            }
            0x00BE => {
                // CLEAR_PRINTS
                self.world_commands.push(ScmWorldCommand::ClearPrints);
                thread.ip = next;
            }
            0x00C0 => {
                // SET_TIME_OF_DAY(hour, minute)
                if args.len() >= 2 {
                    let hour = self.eval_i32(thread, &args[0]).clamp(0, 23) as u8;
                    let minute = self.eval_i32(thread, &args[1]).clamp(0, 59) as u8;
                    self.world_commands
                        .push(ScmWorldCommand::SetTimeOfDay { hour, minute });
                }
                thread.ip = next;
            }
            0x00C5 => {
                // RETURN_TRUE
                update_compare_flag(thread, true, false);
                thread.ip = next;
            }
            0x00C6 => {
                // RETURN_FALSE
                update_compare_flag(thread, false, false);
                thread.ip = next;
            }
            0x00D6 => {
                // ANDOR(logical_op) per RunningScript::AndOr
                let op = args.first().map(|a| self.eval_i32(thread, a)).unwrap_or(0) as u8;
                if op == 0 {
                    thread.and_or_state = 0;
                    thread.cond_result = false;
                } else if (1..=8).contains(&op) {
                    thread.and_or_state = op + 1;
                    thread.cond_result = true;
                } else if (21..=28).contains(&op) {
                    thread.and_or_state = op + 1;
                    thread.cond_result = false;
                }
                thread.ip = next;
            }
            0x00D8 => {
                // MISSION_HAS_FINISHED
                if thread.is_mission {
                    thread.terminated = true;
                    return false;
                }
                thread.ip = next;
            }
            0x00D9 | 0x03C0 => {
                // STORE_CAR_CHAR_IS_IN / NO_SAVE(char, out_car)
                if args.len() >= 2 {
                    let handle = self.eval_i32(thread, &args[0]);
                    let car = if self.is_player_actor(handle) {
                        self.world_snapshot.player_car_handle.unwrap_or(0)
                    } else {
                        0
                    };
                    self.write_i32(thread, &args[1], car);
                }
                thread.ip = next;
            }
            0x00DB => {
                // IS_CHAR_IN_CAR(char, car)
                let cond = if args.len() >= 2 {
                    let ch = self.eval_i32(thread, &args[0]);
                    let car = self.eval_i32(thread, &args[1]);
                    self.is_player_actor(ch)
                        && self.world_snapshot.player_in_car
                        && self.world_snapshot.player_car_handle == Some(car)
                } else {
                    false
                };
                update_compare_flag(thread, cond, instr.not_flag);
                thread.ip = next;
            }
            0x00DD => {
                // IS_CHAR_IN_MODEL(char, model_id)
                let cond = if args.len() >= 2 {
                    let ch = self.eval_i32(thread, &args[0]);
                    let model = self.eval_i32(thread, &args[1]);
                    self.is_player_actor(ch)
                        && self.world_snapshot.player_in_car
                        && self.world_snapshot.player_car_model == Some(model)
                } else {
                    false
                };
                update_compare_flag(thread, cond, instr.not_flag);
                thread.ip = next;
            }
            0x00DF => {
                // IS_CHAR_IN_ANY_CAR(char)
                let cond = args.first().is_some_and(|a| {
                    let ch = self.eval_i32(thread, a);
                    self.is_player_actor(ch) && self.world_snapshot.player_in_car
                });
                update_compare_flag(thread, cond, instr.not_flag);
                thread.ip = next;
            }
            0x044B => {
                // IS_CHAR_ON_FOOT(char)
                let cond = args.first().is_some_and(|a| {
                    let ch = self.eval_i32(thread, a);
                    self.is_player_actor(ch) && !self.world_snapshot.player_in_car
                });
                update_compare_flag(thread, cond, instr.not_flag);
                thread.ip = next;
            }
            0x00E1 => {
                // IS_BUTTON_PRESSED(pad_id, button_id)
                let cond = if args.len() >= 2 {
                    let btn = self.eval_i32(thread, &args[1]);
                    usize::try_from(btn)
                        .ok()
                        .and_then(|i| self.world_snapshot.buttons_pressed.get(i))
                        .copied()
                        .unwrap_or(false)
                } else {
                    false
                };
                update_compare_flag(thread, cond, instr.not_flag);
                thread.ip = next;
            }
            0x00EC..=0x00EE => {
                // LOCATE_CHAR_ANY_MEANS_2D / ON_FOOT_2D / IN_CAR_2D
                let cond = if args.len() >= 5 {
                    let ch = self.eval_i32(thread, &args[0]);
                    let mode_ok = match instr.opcode {
                        0x00ED => !self.world_snapshot.player_in_car,
                        0x00EE => self.world_snapshot.player_in_car,
                        _ => true,
                    };
                    mode_ok
                        && self.entity_pos(ch, false).is_some_and(|p| {
                            locate_2d(
                                p,
                                self.eval_f32(thread, &args[1]),
                                self.eval_f32(thread, &args[2]),
                                self.eval_f32(thread, &args[3]),
                                self.eval_f32(thread, &args[4]),
                            )
                        })
                } else {
                    false
                };
                update_compare_flag(thread, cond, instr.not_flag);
                thread.ip = next;
            }
            0x00FE..=0x0103 => {
                // LOCATE_[STOPPED_]CHAR_ANY_MEANS/ON_FOOT/IN_CAR_3D
                let cond = if args.len() >= 7 {
                    let ch = self.eval_i32(thread, &args[0]);
                    let mode_ok = match instr.opcode {
                        0x00FF | 0x0102 => !self.world_snapshot.player_in_car,
                        0x0100 | 0x0103 => self.world_snapshot.player_in_car,
                        _ => true,
                    };
                    let stopped_ok = instr.opcode <= 0x0100
                        || !self.is_player_actor(ch)
                        || self.world_snapshot.player_stopped;
                    mode_ok
                        && stopped_ok
                        && self.entity_pos(ch, false).is_some_and(|p| {
                            locate_3d(
                                p,
                                self.eval_f32(thread, &args[1]),
                                self.eval_f32(thread, &args[2]),
                                self.eval_f32(thread, &args[3]),
                                self.eval_f32(thread, &args[4]),
                                self.eval_f32(thread, &args[5]),
                                self.eval_f32(thread, &args[6]),
                            )
                        })
                } else {
                    false
                };
                update_compare_flag(thread, cond, instr.not_flag);
                thread.ip = next;
            }
            0x0104 => {
                // LOCATE_CHAR_ANY_MEANS_CHAR_3D(char, target, rx, ry, rz, draw_sphere)
                let cond = if args.len() >= 5 {
                    let ch = self.eval_i32(thread, &args[0]);
                    let tgt = self.eval_i32(thread, &args[1]);
                    match (self.entity_pos(ch, false), self.entity_pos(tgt, false)) {
                        (Some(a), Some(b)) => locate_3d(
                            a,
                            b[0],
                            b[1],
                            b[2],
                            self.eval_f32(thread, &args[2]),
                            self.eval_f32(thread, &args[3]),
                            self.eval_f32(thread, &args[4]),
                        ),
                        _ => false,
                    }
                } else {
                    false
                };
                update_compare_flag(thread, cond, instr.not_flag);
                thread.ip = next;
            }
            0x0109 => {
                // ADD_SCORE(player, amount)
                if args.len() >= 2 {
                    let delta = self.eval_i32(thread, &args[1]);
                    self.pending_effects.cash_deltas.push(delta);
                }
                thread.ip = next;
            }
            0x010D => {
                // ALTER_WANTED_LEVEL(player_idx, stars)
                if args.len() >= 2 {
                    let player_idx = self.eval_i32(thread, &args[0]);
                    let stars = self.eval_i32(thread, &args[1]).clamp(0, 6) as u8;
                    self.world_commands
                        .push(ScmWorldCommand::AlterWantedLevel { player_idx, stars });
                }
                thread.ip = next;
            }
            0x0110 => {
                // CLEAR_WANTED_LEVEL(player_idx)
                if let Some(a) = args.first() {
                    let player_idx = self.eval_i32(thread, a);
                    self.world_commands
                        .push(ScmWorldCommand::ClearWantedLevel { player_idx });
                }
                thread.ip = next;
            }
            0x0117 => {
                // IS_PLAYER_DEAD(player_idx)
                update_compare_flag(thread, self.world_snapshot.player_dead, instr.not_flag);
                thread.ip = next;
            }
            0x0118 => {
                // IS_CHAR_DEAD(char)
                let h = args.first().map(|a| self.eval_i32(thread, a)).unwrap_or(0);
                let dead = h <= 0
                    || (self.is_player_actor(h) && self.world_snapshot.player_dead)
                    || self.world_snapshot.dead_chars.contains(&h);
                update_compare_flag(thread, dead, instr.not_flag);
                thread.ip = next;
            }
            0x0119 => {
                // IS_CAR_DEAD(car)
                let h = args.first().map(|a| self.eval_i32(thread, a)).unwrap_or(0);
                let dead = h <= 0 || self.world_snapshot.dead_cars.contains(&h);
                update_compare_flag(thread, dead, instr.not_flag);
                thread.ip = next;
            }
            0x056D => {
                // DOES_CHAR_EXIST(char)
                let h = args.first().map(|a| self.eval_i32(thread, a)).unwrap_or(0);
                let exists = h > 0 && !self.world_snapshot.dead_chars.contains(&h);
                update_compare_flag(thread, exists, instr.not_flag);
                thread.ip = next;
            }
            0x056E => {
                // DOES_VEHICLE_EXIST(car)
                let h = args.first().map(|a| self.eval_i32(thread, a)).unwrap_or(0);
                let exists = h > 0 && !self.world_snapshot.dead_cars.contains(&h);
                update_compare_flag(thread, exists, instr.not_flag);
                thread.ip = next;
            }
            0x014B => {
                // CREATE_CAR_GENERATOR(x, y, z, angle, model, c1, c2, force, alarm, door_lock, min_delay, max_delay, out_handle)
                if args.len() >= 13 {
                    let pos = [
                        self.eval_f32(thread, &args[0]),
                        self.eval_f32(thread, &args[1]),
                        self.eval_f32(thread, &args[2]),
                    ];
                    let angle_deg = self.eval_f32(thread, &args[3]);
                    let model_id = self.eval_i32(thread, &args[4]);
                    let handle = self.alloc_handle();
                    self.write_i32(thread, &args[12], handle);
                    self.world_commands
                        .push(ScmWorldCommand::CreateCarGenerator {
                            pos,
                            angle_deg,
                            model_id,
                        });
                }
                thread.ip = next;
            }
            0x0158 => {
                // POINT_CAMERA_AT_CAR(car, mode, switch_style)
                if args.len() >= 3 {
                    let handle = self.eval_i32(thread, &args[0]);
                    let mode = self.eval_i32(thread, &args[1]);
                    let switch_style = self.eval_i32(thread, &args[2]);
                    self.world_commands.push(ScmWorldCommand::PointCameraAtCar {
                        handle,
                        mode,
                        switch_style,
                    });
                }
                thread.ip = next;
            }
            0x0159 => {
                // POINT_CAMERA_AT_CHAR(char, mode, switch_style)
                if args.len() >= 3 {
                    let handle = self.eval_i32(thread, &args[0]);
                    let mode = self.eval_i32(thread, &args[1]);
                    let switch_style = self.eval_i32(thread, &args[2]);
                    self.world_commands
                        .push(ScmWorldCommand::PointCameraAtChar {
                            handle,
                            mode,
                            switch_style,
                        });
                }
                thread.ip = next;
            }
            0x015A | 0x02EB => {
                // RESTORE_CAMERA / RESTORE_CAMERA_JUMPCUT
                self.world_commands.push(ScmWorldCommand::RestoreCamera {
                    jumpcut: instr.opcode == 0x02EB,
                });
                thread.ip = next;
            }
            0x015F => {
                // SET_FIXED_CAMERA_POSITION(x, y, z, rx, ry, rz)
                if args.len() >= 6 {
                    self.world_commands
                        .push(ScmWorldCommand::SetFixedCameraPosition {
                            pos: [
                                self.eval_f32(thread, &args[0]),
                                self.eval_f32(thread, &args[1]),
                                self.eval_f32(thread, &args[2]),
                            ],
                            rot_offset: [
                                self.eval_f32(thread, &args[3]),
                                self.eval_f32(thread, &args[4]),
                                self.eval_f32(thread, &args[5]),
                            ],
                        });
                }
                thread.ip = next;
            }
            0x0160 => {
                // POINT_CAMERA_AT_POINT(x, y, z, switch_style)
                if args.len() >= 4 {
                    self.world_commands
                        .push(ScmWorldCommand::PointCameraAtPoint {
                            pos: [
                                self.eval_f32(thread, &args[0]),
                                self.eval_f32(thread, &args[1]),
                                self.eval_f32(thread, &args[2]),
                            ],
                            switch_style: self.eval_i32(thread, &args[3]),
                        });
                }
                thread.ip = next;
            }
            0x0373 => {
                // SET_CAMERA_BEHIND_PLAYER
                self.world_commands
                    .push(ScmWorldCommand::SetCameraBehindPlayer);
                thread.ip = next;
            }
            0x0164 => {
                // REMOVE_BLIP(handle)
                if let Some(a) = args.first() {
                    let handle = self.eval_i32(thread, a);
                    self.world_commands
                        .push(ScmWorldCommand::RemoveBlip(handle));
                }
                thread.ip = next;
            }
            0x016A => {
                // DO_FADE(ms, direction)
                if args.len() >= 2 {
                    let duration_ms = self.eval_i32(thread, &args[0]);
                    let out = self.eval_i32(thread, &args[1]) == 0;
                    self.world_commands
                        .push(ScmWorldCommand::DoFade { duration_ms, out });
                }
                thread.ip = next;
            }
            0x016B => {
                // GET_FADING_STATUS
                update_compare_flag(thread, self.world_snapshot.fade_active, instr.not_flag);
                thread.ip = next;
            }
            0x0171 => {
                // SET_PLAYER_HEADING(player, heading_deg)
                if args.len() >= 2 {
                    let heading_deg = self.eval_f32(thread, &args[1]);
                    self.world_commands.push(ScmWorldCommand::SetCharHeading {
                        handle: self.player_actor_handle.max(1),
                        heading_deg,
                    });
                }
                thread.ip = next;
            }
            0x0172 => {
                // GET_CHAR_HEADING(char, out_heading)
                if args.len() >= 2 {
                    self.write_f32(thread, &args[1], self.world_snapshot.player_heading_deg);
                }
                thread.ip = next;
            }
            0x0173 => {
                // SET_CHAR_HEADING(char, heading_deg)
                if args.len() >= 2 {
                    let handle = self.eval_i32(thread, &args[0]);
                    let heading_deg = self.eval_f32(thread, &args[1]);
                    self.world_commands.push(ScmWorldCommand::SetCharHeading {
                        handle,
                        heading_deg,
                    });
                }
                thread.ip = next;
            }
            0x018A => {
                // ADD_BLIP_FOR_COORD(x, y, z, out_handle)
                if args.len() >= 4 {
                    let pos = [
                        self.eval_f32(thread, &args[0]),
                        self.eval_f32(thread, &args[1]),
                        self.eval_f32(thread, &args[2]),
                    ];
                    let handle = self.alloc_handle();
                    self.write_i32(thread, &args[3], handle);
                    self.world_commands.push(ScmWorldCommand::AddBlip {
                        kind: ScmBlipKind::Coord,
                        pos,
                        handle,
                    });
                }
                thread.ip = next;
            }
            0x01B2 => {
                // GIVE_WEAPON_TO_CHAR(char, weapon_type, ammo)
                if args.len() >= 3 {
                    let handle = self.eval_i32(thread, &args[0]);
                    let weapon_type = self.eval_i32(thread, &args[1]);
                    let ammo = self.eval_i32(thread, &args[2]);
                    self.world_commands.push(ScmWorldCommand::GiveWeaponToChar {
                        handle,
                        weapon_type,
                        ammo,
                    });
                }
                thread.ip = next;
            }
            0x01B4 => {
                // SET_PLAYER_CONTROL(player_idx, state)
                if args.len() >= 2 {
                    let player_idx = self.eval_i32(thread, &args[0]);
                    let enabled = self.eval_i32(thread, &args[1]) != 0;
                    self.world_commands.push(ScmWorldCommand::SetPlayerControl {
                        player_idx,
                        enabled,
                    });
                }
                thread.ip = next;
            }
            0x01B5 | 0x01B6 => {
                // FORCE_WEATHER / FORCE_WEATHER_NOW
                if let Some(a) = args.first() {
                    let w = self.eval_i32(thread, a);
                    self.world_commands.push(ScmWorldCommand::ForceWeather(w));
                }
                thread.ip = next;
            }
            0x01BD => {
                // GET_GAME_TIMER(out_ms)
                if let Some(out) = args.first() {
                    self.write_i32(thread, out, self.elapsed_ms);
                }
                thread.ip = next;
            }
            0x01F0 => {
                // SET_MAX_WANTED_LEVEL(stars)
                if let Some(a) = args.first() {
                    let stars = self.eval_i32(thread, a).clamp(0, 6) as u8;
                    self.world_commands
                        .push(ScmWorldCommand::SetMaxWantedLevel(stars));
                }
                thread.ip = next;
            }
            0x01F5 => {
                // GET_PLAYER_CHAR(player_idx, out_handle)
                if args.len() >= 2 {
                    let h = if self.player_actor_handle > 0 {
                        self.player_actor_handle
                    } else {
                        let allocated = self.alloc_handle();
                        self.player_actor_handle = allocated;
                        allocated
                    };
                    self.write_i32(thread, &args[1], h);
                }
                thread.ip = next;
            }
            0x0208 => {
                // GENERATE_RANDOM_FLOAT_IN_RANGE(min, max, out)
                if args.len() >= 3 {
                    let lo = self.eval_f32(thread, &args[0]);
                    let hi = self.eval_f32(thread, &args[1]);
                    let t = ((self.next_random_u64() >> 40) as f32) / ((1u64 << 24) as f32);
                    self.write_f32(thread, &args[2], lo + (hi - lo) * t);
                }
                thread.ip = next;
            }
            0x0209 => {
                // GENERATE_RANDOM_INT_IN_RANGE(min, max, out)
                if args.len() >= 3 {
                    let lo = self.eval_i32(thread, &args[0]);
                    let hi = self.eval_i32(thread, &args[1]);
                    let val = if hi > lo {
                        let span = (hi as i64 - lo as i64) as u64;
                        lo + (self.next_random_u64() % span) as i32
                    } else {
                        lo
                    };
                    self.write_i32(thread, &args[2], val);
                }
                thread.ip = next;
            }
            0x0213 | 0x032B => {
                // CREATE_PICKUP(model, type, x, y, z, out_handle) / WITH_AMMO
                if args.len() >= 6 {
                    let model_id = self.eval_i32(thread, &args[0]);
                    let pickup_type = self.eval_i32(thread, &args[1]);
                    let pos = [
                        self.eval_f32(thread, &args[2]),
                        self.eval_f32(thread, &args[3]),
                        self.eval_f32(thread, &args[4]),
                    ];
                    let handle = self.alloc_handle();
                    self.spawned_pickups.insert(handle, pos);
                    self.write_i32(thread, args.last().unwrap(), handle);
                    self.world_commands.push(ScmWorldCommand::CreatePickup {
                        model_id,
                        pickup_type,
                        pos,
                        handle,
                    });
                }
                thread.ip = next;
            }
            0x0517 | 0x0518 => {
                // CREATE_LOCKED_PROPERTY_PICKUP / CREATE_FORSALE_PROPERTY_PICKUP
                if args.len() >= 5 {
                    let pos = [
                        self.eval_f32(thread, &args[0]),
                        self.eval_f32(thread, &args[1]),
                        self.eval_f32(thread, &args[2]),
                    ];
                    let model_id = if instr.opcode == 0x0517 { 1272 } else { 1273 };
                    let handle = self.alloc_handle();
                    self.spawned_pickups.insert(handle, pos);
                    self.write_i32(thread, args.last().unwrap(), handle);
                    self.world_commands.push(ScmWorldCommand::CreatePickup {
                        model_id,
                        pickup_type: 1,
                        pos,
                        handle,
                    });
                }
                thread.ip = next;
            }
            0x03DC => {
                // ADD_BLIP_FOR_PICKUP(pickup, out_handle)
                if args.len() >= 2 {
                    let pickup_h = self.eval_i32(thread, &args[0]);
                    let handle = self.alloc_handle();
                    self.write_i32(thread, &args[1], handle);
                    if let Some(&pos) = self.spawned_pickups.get(&pickup_h) {
                        self.world_commands.push(ScmWorldCommand::AddBlip {
                            kind: ScmBlipKind::Coord,
                            pos,
                            handle,
                        });
                    }
                }
                thread.ip = next;
            }
            0x0223 => {
                // SET_CHAR_HEALTH(char, health)
                if args.len() >= 2 {
                    let handle = self.eval_i32(thread, &args[0]);
                    let health = self.eval_i32(thread, &args[1]);
                    self.world_commands
                        .push(ScmWorldCommand::SetCharHealth { handle, health });
                }
                thread.ip = next;
            }
            0x0226 => {
                // GET_CHAR_HEALTH(char, out_health)
                if args.len() >= 2 {
                    self.write_i32(thread, &args[1], self.world_snapshot.player_health);
                }
                thread.ip = next;
            }
            0x035F => {
                // ADD_ARMOUR_TO_CHAR(char, amount)
                if args.len() >= 2 {
                    let handle = self.eval_i32(thread, &args[0]);
                    let amount = self.eval_i32(thread, &args[1]);
                    self.world_commands
                        .push(ScmWorldCommand::AddArmourToChar { handle, amount });
                }
                thread.ip = next;
            }
            0x0244 => {
                // SET_CUTSCENE_OFFSET(x, y, z)
                if args.len() >= 3 {
                    let pos = [
                        self.eval_f32(thread, &args[0]),
                        self.eval_f32(thread, &args[1]),
                        self.eval_f32(thread, &args[2]),
                    ];
                    self.world_commands
                        .push(ScmWorldCommand::SetCutsceneOffset(pos));
                }
                thread.ip = next;
            }
            0x0247 => {
                // REQUEST_MODEL
                if let Some(a) = args.first() {
                    let m = self.eval_i32(thread, a);
                    self.world_commands.push(ScmWorldCommand::RequestModel(m));
                }
                thread.ip = next;
            }
            0x023D | 0x0248 | 0x03D0 | 0x03D2 | 0x04EE | 0x06EA | 0x07C1 => {
                // Streaming / mission audio ready checks
                update_compare_flag(thread, true, instr.not_flag);
                thread.ip = next;
            }
            0x0256 => {
                // IS_PLAYER_PLAYING(player_idx)
                update_compare_flag(thread, !self.world_snapshot.player_dead, instr.not_flag);
                thread.ip = next;
            }
            0x09E7 => {
                // IS_PLAYER_CONTROL_ON(player_idx)
                update_compare_flag(thread, self.world_snapshot.player_control, instr.not_flag);
                thread.ip = next;
            }
            0x0299 | 0x02B9 => {
                // ACTIVATE_GARAGE / DEACTIVATE_GARAGE
                if let Some(name) = args.first().and_then(ScmArg::as_str_literal) {
                    self.world_commands.push(ScmWorldCommand::ActivateGarage {
                        name: name.to_string(),
                        active: instr.opcode == 0x0299,
                    });
                }
                thread.ip = next;
            }
            0x03B0 | 0x03B1 => {
                // IS_GARAGE_OPEN / IS_GARAGE_CLOSED
                let open = args
                    .first()
                    .and_then(ScmArg::as_str_literal)
                    .and_then(|s| {
                        self.world_snapshot
                            .garage_open
                            .get(&s.to_ascii_lowercase())
                            .copied()
                    })
                    .unwrap_or(false);
                let cond = if instr.opcode == 0x03B0 { open } else { !open };
                update_compare_flag(thread, cond, instr.not_flag);
                thread.ip = next;
            }
            0x02A3 => {
                // SWITCH_WIDESCREEN(state)
                if let Some(a) = args.first() {
                    let on = self.eval_i32(thread, a) != 0;
                    self.world_commands
                        .push(ScmWorldCommand::ToggleWidescreen(on));
                }
                thread.ip = next;
            }
            0x02A7 | 0x02A8 | 0x04CE | 0x0570 => {
                // ADD_[SHORT_RANGE_]SPRITE_BLIP_FOR_COORD / CONTACT_POINT
                if args.len() >= 5 {
                    let pos = [
                        self.eval_f32(thread, &args[0]),
                        self.eval_f32(thread, &args[1]),
                        self.eval_f32(thread, &args[2]),
                    ];
                    let sprite = self.eval_i32(thread, &args[3]);
                    let handle = self.alloc_handle();
                    self.write_i32(thread, &args[4], handle);
                    self.world_commands.push(ScmWorldCommand::AddBlip {
                        kind: ScmBlipKind::Sprite(sprite),
                        pos,
                        handle,
                    });
                }
                thread.ip = next;
            }
            0x02E4 => {
                // LOAD_CUTSCENE(name)
                if let Some(name) = args.first().and_then(ScmArg::as_str_literal) {
                    self.world_snapshot.cutscene_finished = false;
                    self.world_snapshot.cutscene_time_ms = 0;
                    self.world_commands
                        .push(ScmWorldCommand::LoadCutscene(name.to_string()));
                }
                thread.ip = next;
            }
            0x06B9 => {
                // HAS_CUTSCENE_LOADED
                update_compare_flag(thread, self.world_snapshot.cutscene_loaded, instr.not_flag);
                thread.ip = next;
            }
            0x02E7 => {
                // START_CUTSCENE
                self.world_snapshot.cutscene_finished = false;
                self.world_snapshot.cutscene_time_ms = 0;
                self.world_commands.push(ScmWorldCommand::StartCutscene);
                thread.ip = next;
            }
            0x02E8 => {
                // GET_CUTSCENE_TIME(out_ms)
                if let Some(out) = args.first() {
                    self.write_i32(thread, out, self.world_snapshot.cutscene_time_ms);
                }
                thread.ip = next;
            }
            0x02E9 => {
                // HAS_CUTSCENE_FINISHED
                update_compare_flag(
                    thread,
                    self.world_snapshot.cutscene_finished,
                    instr.not_flag,
                );
                thread.ip = next;
            }
            0x02EA => {
                // CLEAR_CUTSCENE
                self.world_snapshot.cutscene_loaded = false;
                self.world_commands.push(ScmWorldCommand::ClearCutscene);
                thread.ip = next;
            }
            0x056A | 0x08D0 => {
                // WAS_CUTSCENE_SKIPPED / IS_SKIP_CUTSCENE_BUTTON_PRESSED
                update_compare_flag(thread, self.world_snapshot.cutscene_skipped, instr.not_flag);
                thread.ip = next;
            }
            0x0707 => {
                // SKIP_CUTSCENE_START_INTERNAL(label)
                if let Some(a) = args.first() {
                    let target = self.eval_i32(thread, a);
                    thread.scene_skip_ip = Some(self.resolve_thread_jump(thread, target));
                }
                thread.ip = next;
            }
            0x0701 => {
                // SKIP_CUTSCENE_END
                thread.scene_skip_ip = None;
                thread.ip = next;
            }
            0x03A4 => {
                // SCRIPT_NAME
                if let Some(s) = args.first().and_then(ScmArg::as_str_literal) {
                    thread.name = s.to_ascii_lowercase();
                }
                thread.ip = next;
            }
            0x03CB => {
                // LOAD_SCENE(x, y, z)
                if args.len() >= 3 {
                    let pos = [
                        self.eval_f32(thread, &args[0]),
                        self.eval_f32(thread, &args[1]),
                        self.eval_f32(thread, &args[2]),
                    ];
                    self.world_commands.push(ScmWorldCommand::LoadScene(pos));
                }
                thread.ip = next;
            }
            0x03E5 => {
                // PRINT_HELP(key)
                if let Some(key) = args.first().and_then(ScmArg::as_str_literal) {
                    self.world_commands.push(ScmWorldCommand::PrintHelp {
                        key: key.to_string(),
                    });
                }
                thread.ip = next;
            }
            0x03E6 => {
                // CLEAR_HELP
                self.world_commands.push(ScmWorldCommand::ClearHelp);
                thread.ip = next;
            }
            0x0417 => {
                // LOAD_AND_LAUNCH_MISSION_INTERNAL(mission_idx)
                if let Some(a) = args.first() {
                    let m_idx = self.eval_i32(thread, a);
                    if let Ok(idx) = usize::try_from(m_idx)
                        && let Some(&offset) = self
                            .scm_image
                            .as_ref()
                            .and_then(|img| img.mission_offsets.get(idx))
                    {
                        let m_thread =
                            ScmBytecodeThread::new(format!("mission_{idx}"), offset as usize, true);
                        spawned.push(m_thread);
                        trace!(mission = idx, offset, "Launched SCM mission script");
                    }
                }
                thread.ip = next;
            }
            0x0509 => {
                // GET_DISTANCE_BETWEEN_COORDS_2D(x1, y1, x2, y2, out_dist)
                if args.len() >= 5 {
                    let dx = self.eval_f32(thread, &args[2]) - self.eval_f32(thread, &args[0]);
                    let dy = self.eval_f32(thread, &args[3]) - self.eval_f32(thread, &args[1]);
                    self.write_f32(thread, &args[4], (dx * dx + dy * dy).sqrt());
                }
                thread.ip = next;
            }
            0x050A => {
                // GET_DISTANCE_BETWEEN_COORDS_3D(x1, y1, z1, x2, y2, z2, out_dist)
                if args.len() >= 7 {
                    let dx = self.eval_f32(thread, &args[3]) - self.eval_f32(thread, &args[0]);
                    let dy = self.eval_f32(thread, &args[4]) - self.eval_f32(thread, &args[1]);
                    let dz = self.eval_f32(thread, &args[5]) - self.eval_f32(thread, &args[2]);
                    self.write_f32(thread, &args[6], (dx * dx + dy * dy + dz * dz).sqrt());
                }
                thread.ip = next;
            }
            0x0581 => {
                // DISPLAY_RADAR
                if let Some(a) = args.first() {
                    let v = self.eval_i32(thread, a) != 0;
                    self.world_commands.push(ScmWorldCommand::DisplayRadar(v));
                }
                thread.ip = next;
            }
            0x06D5 => {
                // CREATE_CHECKPOINT(type, x, y, z, px, py, pz, radius, out_handle)
                if args.len() >= 9 {
                    let kind = self.eval_i32(thread, &args[0]);
                    let pos = [
                        self.eval_f32(thread, &args[1]),
                        self.eval_f32(thread, &args[2]),
                        self.eval_f32(thread, &args[3]),
                    ];
                    let point_at = [
                        self.eval_f32(thread, &args[4]),
                        self.eval_f32(thread, &args[5]),
                        self.eval_f32(thread, &args[6]),
                    ];
                    let radius = self.eval_f32(thread, &args[7]);
                    let handle = self.alloc_handle();
                    self.write_i32(thread, &args[8], handle);
                    self.world_commands.push(ScmWorldCommand::CreateCheckpoint {
                        kind,
                        pos,
                        point_at,
                        radius,
                        handle,
                    });
                }
                thread.ip = next;
            }
            0x06D6 => {
                // DELETE_CHECKPOINT(handle)
                if let Some(a) = args.first() {
                    let handle = self.eval_i32(thread, a);
                    self.world_commands
                        .push(ScmWorldCommand::DeleteCheckpoint(handle));
                }
                thread.ip = next;
            }
            0x0826 => {
                // DISPLAY_HUD(state)
                if let Some(a) = args.first() {
                    let v = self.eval_i32(thread, a) != 0;
                    self.world_commands.push(ScmWorldCommand::DisplayHud(v));
                }
                thread.ip = next;
            }
            0x0871 => {
                // SWITCH_START(val, num_cases, has_default, default_label, 7 x (case, label))
                if args.len() >= 4 {
                    let val = self.eval_i32(thread, &args[0]);
                    let num_cases = self.eval_i32(thread, &args[1]).max(0) as usize;
                    let default_label = Some(self.eval_i32(thread, &args[3]));
                    let inline_cases = num_cases.min(7).min(args.len().saturating_sub(4) / 2);
                    let mut matched = None;
                    for i in 0..inline_cases {
                        if self.eval_i32(thread, &args[4 + i * 2]) == val {
                            matched = Some(self.eval_i32(thread, &args[4 + i * 2 + 1]));
                            break;
                        }
                    }
                    if let Some(label) = matched {
                        thread.pending_switch = None;
                        let (src, ip) = self.resolve_thread_jump(thread, label);
                        thread.source = src;
                        thread.ip = ip;
                    } else if num_cases <= 7 {
                        thread.pending_switch = None;
                        if let Some(dl) = default_label.filter(|&l| l != 0) {
                            let (src, ip) = self.resolve_thread_jump(thread, dl);
                            thread.source = src;
                            thread.ip = ip;
                        } else {
                            thread.ip = next;
                        }
                    } else {
                        thread.pending_switch = Some(PendingSwitch {
                            value: val,
                            remaining: num_cases - 7,
                            default_label,
                        });
                        thread.ip = next;
                    }
                } else {
                    thread.ip = next;
                }
            }
            0x0872 => {
                // SWITCH_CONTINUED(9 x (case, label))
                if let Some(mut sw) = thread.pending_switch.take() {
                    let chunk = sw.remaining.min(9).min(args.len() / 2);
                    let mut matched = None;
                    for i in 0..chunk {
                        if self.eval_i32(thread, &args[i * 2]) == sw.value {
                            matched = Some(self.eval_i32(thread, &args[i * 2 + 1]));
                            break;
                        }
                    }
                    if let Some(label) = matched {
                        let (src, ip) = self.resolve_thread_jump(thread, label);
                        thread.source = src;
                        thread.ip = ip;
                    } else if sw.remaining > 9 {
                        sw.remaining -= 9;
                        thread.pending_switch = Some(sw);
                        thread.ip = next;
                    } else if let Some(dl) = sw.default_label.filter(|&l| l != 0) {
                        let (src, ip) = self.resolve_thread_jump(thread, dl);
                        thread.source = src;
                        thread.ip = ip;
                    } else {
                        thread.ip = next;
                    }
                } else {
                    thread.ip = next;
                }
            }
            0x08AB => {
                // HAS_STREAMED_SCRIPT_LOADED(stream_id)
                let loaded = args
                    .first()
                    .and_then(|a| usize::try_from(self.eval_i32(thread, a)).ok())
                    .is_some_and(|idx| self.streamed_scripts.contains_key(&idx));
                update_compare_flag(thread, loaded, instr.not_flag);
                thread.ip = next;
            }
            0x0913 => {
                // START_NEW_STREAMED_SCRIPT(stream_id, ...args)
                if let Some(a) = args.first() {
                    let sid = self.eval_i32(thread, a);
                    if let Ok(idx) = usize::try_from(sid)
                        && self.streamed_scripts.contains_key(&idx)
                    {
                        let sname = self
                            .scm_image
                            .as_ref()
                            .and_then(|img| img.streamed_scripts.get(idx))
                            .map(|s| s.name.to_ascii_lowercase())
                            .unwrap_or_else(|| format!("stream_{idx}"));
                        let mut s_thread = ScmBytecodeThread::with_source(
                            sname,
                            0,
                            0,
                            ScmCodeSource::Streamed(idx),
                            false,
                        );
                        for (slot, arg) in args.iter().skip(1).enumerate() {
                            if slot < s_thread.locals.len() {
                                s_thread.locals[slot] = self.eval_raw_word(thread, arg);
                            }
                        }
                        spawned.push(s_thread);
                        trace!(stream_id = idx, "Started streamed SCM script thread");
                    }
                }
                thread.ip = next;
            }
            0x0926 => {
                // GET_NUMBER_OF_INSTANCES_OF_STREAMED_SCRIPT(stream_id, out_count)
                if args.len() >= 2 {
                    let sid = self.eval_i32(thread, &args[0]);
                    let count = if let Ok(idx) = usize::try_from(sid) {
                        let existing = self
                            .bytecode_threads
                            .iter()
                            .filter(|t| {
                                !t.terminated && t.home_source == ScmCodeSource::Streamed(idx)
                            })
                            .count();
                        let queued = spawned
                            .iter()
                            .filter(|t| t.home_source == ScmCodeSource::Streamed(idx))
                            .count();
                        (existing + queued) as i32
                    } else {
                        0
                    };
                    self.write_i32(thread, &args[1], count);
                }
                thread.ip = next;
            }
            0x097A => {
                // REPORT_MISSION_AUDIO_EVENT_AT_POSITION(x, y, z, sound_id)
                if args.len() >= 4 {
                    self.world_commands
                        .push(ScmWorldCommand::ReportMissionAudioEvent {
                            pos: Some([
                                self.eval_f32(thread, &args[0]),
                                self.eval_f32(thread, &args[1]),
                                self.eval_f32(thread, &args[2]),
                            ]),
                            sound_id: self.eval_i32(thread, &args[3]),
                        });
                }
                thread.ip = next;
            }
            0x0952 => {
                // PRELOAD_BEAT_TRACK(id)
                self.beat_track_status = 2;
                thread.ip = next;
            }
            0x0953 => {
                // GET_BEAT_TRACK_STATUS(out_status)
                if let Some(out) = args.first() {
                    self.write_i32(thread, out, self.beat_track_status);
                }
                thread.ip = next;
            }
            0x0954 => {
                // PLAY_BEAT_TRACK
                self.beat_track_status = 3;
                thread.ip = next;
            }
            0x0955 => {
                // STOP_BEAT_TRACK
                self.beat_track_status = 0;
                thread.ip = next;
            }
            0x04D4..=0x04D5 => {
                thread.ip = next;
            }
            _ => {
                if instr.is_condition {
                    update_compare_flag(thread, false, instr.not_flag);
                }
                thread.ip = next;
            }
        }
        true
    }

    fn eval_raw_word(&self, thread: &ScmBytecodeThread, arg: &ScmArg) -> i32 {
        match arg {
            ScmArg::Int(v) => *v,
            ScmArg::Float(f) => f.to_bits() as i32,
            ScmArg::GlobalVar(off) => self.read_global_i32(*off as usize),
            ScmArg::LocalVar(slot) => thread.locals.get(*slot as usize).copied().unwrap_or(0),
            ScmArg::GlobalArray(arr) => {
                let idx = self.read_array_index(thread, arr.index_var, arr.index_is_global);
                self.read_global_i32(usize::from(arr.base_offset) + (idx.max(0) as usize) * 4)
            }
            ScmArg::LocalArray(arr) => {
                let idx = self.read_array_index(thread, arr.index_var, arr.index_is_global);
                thread
                    .locals
                    .get(usize::from(arr.base_offset) + idx.max(0) as usize)
                    .copied()
                    .unwrap_or(0)
            }
            _ => 0,
        }
    }

    fn eval_i32(&self, thread: &ScmBytecodeThread, arg: &ScmArg) -> i32 {
        match arg {
            ScmArg::Float(f) => *f as i32,
            _ => self.eval_raw_word(thread, arg),
        }
    }

    fn eval_f32(&self, thread: &ScmBytecodeThread, arg: &ScmArg) -> f32 {
        match arg {
            ScmArg::Float(f) => *f,
            ScmArg::Int(v) => *v as f32,
            _ => f32::from_bits(self.eval_raw_word(thread, arg) as u32),
        }
    }

    fn read_array_index(&self, thread: &ScmBytecodeThread, var: u16, is_global: bool) -> i32 {
        if is_global {
            self.read_global_i32(usize::from(var))
        } else {
            thread.locals.get(usize::from(var)).copied().unwrap_or(0)
        }
    }

    fn read_global_i32(&self, offset: usize) -> i32 {
        if offset + 4 <= self.scm_globals.len() {
            i32::from_le_bytes(self.scm_globals[offset..offset + 4].try_into().unwrap())
        } else {
            0
        }
    }

    fn write_global_i32(&mut self, offset: usize, val: i32) {
        if offset + 4 <= self.scm_globals.len() {
            self.scm_globals[offset..offset + 4].copy_from_slice(&val.to_le_bytes());
        }
    }

    fn write_i32(&mut self, thread: &mut ScmBytecodeThread, dst: &ScmArg, val: i32) {
        match dst {
            ScmArg::GlobalVar(off) => self.write_global_i32(usize::from(*off), val),
            ScmArg::LocalVar(slot) => {
                if let Some(cell) = thread.locals.get_mut(usize::from(*slot)) {
                    *cell = val;
                }
            }
            ScmArg::GlobalArray(arr) => {
                let idx = self.read_array_index(thread, arr.index_var, arr.index_is_global);
                if idx >= 0 {
                    self.write_global_i32(usize::from(arr.base_offset) + (idx as usize) * 4, val);
                }
            }
            ScmArg::LocalArray(arr) => {
                let idx = self.read_array_index(thread, arr.index_var, arr.index_is_global);
                if idx >= 0
                    && let Some(cell) = thread
                        .locals
                        .get_mut(usize::from(arr.base_offset) + idx as usize)
                {
                    *cell = val;
                }
            }
            _ => {}
        }
    }

    fn write_f32(&mut self, thread: &mut ScmBytecodeThread, dst: &ScmArg, val: f32) {
        self.write_i32(thread, dst, val.to_bits() as i32);
    }
}

fn in_area_2d(pos: [f32; 3], x1: f32, y1: f32, x2: f32, y2: f32) -> bool {
    let (min_x, max_x) = (x1.min(x2), x1.max(x2));
    let (min_y, max_y) = (y1.min(y2), y1.max(y2));
    pos[0] >= min_x && pos[0] <= max_x && pos[1] >= min_y && pos[1] <= max_y
}

fn in_area_3d(pos: [f32; 3], x1: f32, y1: f32, z1: f32, x2: f32, y2: f32, z2: f32) -> bool {
    let (min_z, max_z) = (z1.min(z2), z1.max(z2));
    in_area_2d(pos, x1, y1, x2, y2) && pos[2] >= min_z && pos[2] <= max_z
}

fn locate_2d(pos: [f32; 3], tx: f32, ty: f32, rx: f32, ry: f32) -> bool {
    (pos[0] - tx).abs() <= rx.abs() && (pos[1] - ty).abs() <= ry.abs()
}

fn locate_3d(pos: [f32; 3], tx: f32, ty: f32, tz: f32, rx: f32, ry: f32, rz: f32) -> bool {
    locate_2d(pos, tx, ty, rx, ry) && (pos[2] - tz).abs() <= rz.abs()
}

fn update_compare_flag(thread: &mut ScmBytecodeThread, raw_state: bool, not_flag: bool) {
    let state = if not_flag { !raw_state } else { raw_state };
    match thread.and_or_state {
        0 => thread.cond_result = state,
        1..=8 => {
            thread.cond_result &= state;
            if thread.and_or_state == 1 {
                thread.and_or_state = 0;
            } else {
                thread.and_or_state -= 1;
            }
        }
        21..=28 => {
            thread.cond_result |= state;
            if thread.and_or_state == 21 {
                thread.and_or_state = 0;
            } else {
                thread.and_or_state -= 1;
            }
        }
        _ => {
            thread.and_or_state = 0;
            thread.cond_result = state;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn push_chunk_header(buf: &mut Vec<u8>, next_off: u32, idx: u8) {
        buf.extend_from_slice(&[0x02, 0x00, 0x01]);
        buf.extend_from_slice(&next_off.to_le_bytes());
        buf.push(idx);
    }

    fn build_minimal_scm_header(stream_names: &[&str]) -> Vec<u8> {
        let mut scm = Vec::new();
        // Chunk 0 ('s' = globals placeholder, 8 bytes -> offset 16)
        push_chunk_header(&mut scm, 16, b's');
        scm.extend_from_slice(&[0u8; 8]);
        // Chunk 1: zero-table
        let chunk2_off = 28u32;
        push_chunk_header(&mut scm, chunk2_off, 0);
        scm.extend_from_slice(&0u32.to_le_bytes());
        // Chunk 2: streamed scripts table (28 bytes per entry: 20 name + 4 offset + 4 size)
        let streamed_payload = 4 + 4 + stream_names.len() * 28;
        let chunk3_off = chunk2_off + 8 + streamed_payload as u32;
        push_chunk_header(&mut scm, chunk3_off, 2);
        scm.extend_from_slice(&0u32.to_le_bytes()); // largest size
        scm.extend_from_slice(&(stream_names.len() as u32).to_le_bytes());
        for name in stream_names {
            let mut rec = [0u8; 28];
            let nb = name.as_bytes();
            let copy = nb.len().min(20);
            rec[..copy].copy_from_slice(&nb[..copy]);
            rec[24..28].copy_from_slice(&32u32.to_le_bytes());
            scm.extend_from_slice(&rec);
        }
        // Chunk 4: globals size & main offset
        let main_off = chunk3_off + 16;
        push_chunk_header(&mut scm, main_off, 4);
        scm.extend_from_slice(&16u32.to_le_bytes());
        scm.extend_from_slice(&1u32.to_le_bytes());
        scm
    }

    #[test]
    fn vm_executes_wait_and_effects() {
        let mut vm = ScmVm::default();
        vm.spawn_thread(
            "intro_reward",
            vec![
                ScriptInstruction::PrintMissionBanner("BIG SMOKE".to_string()),
                ScriptInstruction::WaitMs(500),
                ScriptInstruction::AddPlayerMoney(250),
                ScriptInstruction::TerminateThread,
            ],
        );

        vm.tick(0.1);
        assert_eq!(vm.pending_effects.banners, vec!["BIG SMOKE"]);
        assert!(vm.pending_effects.cash_deltas.is_empty());

        vm.tick(0.6);
        assert_eq!(vm.pending_effects.cash_deltas, vec![250]);
        assert!(vm.threads.is_empty());
    }

    #[test]
    fn vm_runs_compiled_sa_bytecode_init_sequence() {
        let mut scm = Vec::new();
        push_chunk_header(&mut scm, 16, b's');
        scm.extend_from_slice(&[0u8; 8]);
        push_chunk_header(&mut scm, 28, 0);
        scm.extend_from_slice(&0u32.to_le_bytes());
        push_chunk_header(&mut scm, 44, 4);
        scm.extend_from_slice(&16u32.to_le_bytes());
        scm.extend_from_slice(&1u32.to_le_bytes());
        assert_eq!(scm.len(), 44);

        // MAIN: SET_TIME_OF_DAY 8 0 -> CREATE_PLAYER 0 (2488.5, -1666.8, 13.0) -> $8 -> WAIT 0
        scm.extend_from_slice(&0x00C0u16.to_le_bytes());
        scm.extend_from_slice(&[4, 8, 4, 0]);
        scm.extend_from_slice(&0x0053u16.to_le_bytes());
        scm.extend_from_slice(&[4, 0, 6]);
        scm.extend_from_slice(&2488.5f32.to_le_bytes());
        scm.push(6);
        scm.extend_from_slice(&(-1666.8f32).to_le_bytes());
        scm.push(6);
        scm.extend_from_slice(&13.0f32.to_le_bytes());
        scm.extend_from_slice(&[2, 8, 0]);
        scm.extend_from_slice(&0x0001u16.to_le_bytes());
        scm.extend_from_slice(&[4, 0]);

        let img = ScmImage::parse(scm).unwrap();
        let mut vm = ScmVm::default();
        vm.load_scm_image(img);

        assert!(
            vm.world_commands
                .iter()
                .any(|c| matches!(c, ScmWorldCommand::SetTimeOfDay { hour: 8, minute: 0 }))
        );
        assert!(
            vm.world_commands
                .iter()
                .any(|c| matches!(c, ScmWorldCommand::CreatePlayer { player_idx: 0, .. }))
        );
        assert!(vm.read_global_i32(8) > 0);
    }

    #[test]
    fn vm_executes_switch_hud_streamed_script_and_locate_conditions() {
        let mut scm = build_minimal_scm_header(&["AMMU"]);
        let main_off = scm.len();
        // 0x0826 DISPLAY_HUD 1
        scm.extend_from_slice(&0x0826u16.to_le_bytes());
        scm.extend_from_slice(&[4, 1]);
        // 0x0053 CREATE_PLAYER 0 (2488.0, -1666.0, 13.0) -> $8
        scm.extend_from_slice(&0x0053u16.to_le_bytes());
        scm.extend_from_slice(&[4, 0, 6]);
        scm.extend_from_slice(&2488.0f32.to_le_bytes());
        scm.push(6);
        scm.extend_from_slice(&(-1666.0f32).to_le_bytes());
        scm.push(6);
        scm.extend_from_slice(&13.0f32.to_le_bytes());
        scm.extend_from_slice(&[2, 8, 0]);
        // 0x0871 SWITCH_START(val=2, num=2, has_def=1, default_lbl=done_lbl, case 1->0, case 2->case2_lbl, ...)
        let switch_pos = scm.len();
        scm.extend_from_slice(&0x0871u16.to_le_bytes());
        scm.extend_from_slice(&[4, 2, 4, 2, 4, 1]);
        let def_patch = scm.len() + 1;
        scm.extend_from_slice(&[1, 0, 0, 0, 0]);
        scm.extend_from_slice(&[4, 1, 1, 0, 0, 0, 0]);
        scm.extend_from_slice(&[4, 2]);
        let case2_patch = scm.len() + 1;
        scm.extend_from_slice(&[1, 0, 0, 0, 0]);
        for _ in 0..5 {
            scm.extend_from_slice(&[4, 0, 1, 0, 0, 0, 0]);
        }
        // case 1 body (skipped): DISPLAY_HUD 0
        scm.extend_from_slice(&0x0826u16.to_le_bytes());
        scm.extend_from_slice(&[4, 0]);
        // case 2 target:
        let case2_off = scm.len() as i32;
        // 0x02E4 LOAD_CUTSCENE "TESTCUT" -> 0x02E7 START_CUTSCENE
        scm.extend_from_slice(&0x02E4u16.to_le_bytes());
        scm.push(0x0E);
        scm.push(7);
        scm.extend_from_slice(b"TESTCUT");
        scm.extend_from_slice(&0x02E7u16.to_le_bytes());
        // Check 0x00FE LOCATE_CHAR_ANY_MEANS_3D($8, 100.0, 200.0, 15.0, 5.0, 5.0, 5.0, 0)
        scm.extend_from_slice(&0x00FEu16.to_le_bytes());
        scm.extend_from_slice(&[2, 8, 0]);
        for v in [100.0f32, 200.0, 15.0, 5.0, 5.0, 5.0] {
            scm.push(6);
            scm.extend_from_slice(&v.to_le_bytes());
        }
        scm.extend_from_slice(&[4, 0]);
        // GOTO_IF_FALSE done_lbl
        scm.extend_from_slice(&0x004Du16.to_le_bytes());
        let done_patch2 = scm.len() + 1;
        scm.extend_from_slice(&[1, 0, 0, 0, 0]);
        // WAIT 0 then TERMINATE
        let done_off = scm.len() as i32;
        scm.extend_from_slice(&0x0001u16.to_le_bytes());
        scm.extend_from_slice(&[4, 0]);

        assert!(switch_pos >= main_off);
        scm[def_patch..def_patch + 4].copy_from_slice(&done_off.to_le_bytes());
        scm[done_patch2..done_patch2 + 4].copy_from_slice(&done_off.to_le_bytes());
        scm[case2_patch..case2_patch + 4].copy_from_slice(&case2_off.to_le_bytes());

        // Streamed script AMMU bytecode: SET_VAR_INT $12 777 -> TERMINATE_THIS_SCRIPT
        let mut ammu = Vec::new();
        ammu.extend_from_slice(&0x0004u16.to_le_bytes());
        ammu.extend_from_slice(&[2, 12, 0]);
        ammu.extend_from_slice(&[5]);
        ammu.extend_from_slice(&777i16.to_le_bytes());
        ammu.extend_from_slice(&0x004Eu16.to_le_bytes());

        let img = ScmImage::parse(scm).unwrap();
        let mut vm = ScmVm::default();
        vm.world_snapshot.player_pos = [101.0, 199.0, 14.0];
        vm.load_scm_image(img);
        assert_eq!(vm.load_streamed_script_bytes("AMMU.scm", ammu), Some(0));

        assert!(
            vm.world_commands
                .iter()
                .any(|c| matches!(c, ScmWorldCommand::DisplayHud(true)))
        );
        assert!(
            !vm.world_commands
                .iter()
                .any(|c| matches!(c, ScmWorldCommand::DisplayHud(false)))
        );
        assert!(
            vm.world_commands
                .iter()
                .any(|c| matches!(c, ScmWorldCommand::LoadCutscene(name) if name == "TESTCUT"))
        );
        assert!(
            vm.world_commands
                .iter()
                .any(|c| matches!(c, ScmWorldCommand::StartCutscene))
        );
        assert!(vm.bytecode_threads[0].cond_result);

        // Manually inject START_NEW_STREAMED_SCRIPT via helper thread to verify streamed execution.
        let mut launcher = Vec::new();
        let hdr = build_minimal_scm_header(&["AMMU"]);
        launcher.extend_from_slice(&hdr);
        launcher.extend_from_slice(&0x0913u16.to_le_bytes());
        launcher.extend_from_slice(&[4, 0, 0]); // script_id=0, terminator arg 0
        launcher.extend_from_slice(&0x004Eu16.to_le_bytes());
        let img2 = ScmImage::parse(launcher).unwrap();
        let mut ammu2 = Vec::new();
        ammu2.extend_from_slice(&0x0004u16.to_le_bytes());
        ammu2.extend_from_slice(&[2, 12, 0, 5]);
        ammu2.extend_from_slice(&777i16.to_le_bytes());
        ammu2.extend_from_slice(&0x004Eu16.to_le_bytes());
        let mut vm2 = ScmVm {
            scm_globals: vec![0u8; 64],
            scm_image: Some(img2),
            ..Default::default()
        };
        vm2.load_streamed_script_bytes("ammu.scm", ammu2);
        let entry = vm2.scm_image.as_ref().unwrap().main_offset;
        vm2.bytecode_threads
            .push(ScmBytecodeThread::new("launcher", entry, false));
        vm2.tick(0.016);
        vm2.tick(0.016);
        assert_eq!(vm2.read_global_i32(12), 777);
    }

    #[test]
    fn retail_main_scm_and_script_img_boot_threads_survive() {
        let Some(root) = std::env::var_os("GTASA_DIR") else {
            return;
        };
        let root = std::path::PathBuf::from(root);
        let scm_bytes = std::fs::read(root.join("data/script/main.scm")).unwrap();
        let img = ScmImage::parse(scm_bytes).unwrap();
        let script_img =
            gtasarust_gta_formats::ImgArchive::open(root.join("data/script/script.img")).unwrap();

        let mut vm = ScmVm::default();
        vm.world_snapshot.player_pos = [2488.5, -1666.8, 13.3];
        vm.load_scm_image(img);
        for entry in script_img.entries() {
            if let Ok(bytes) = script_img.read_entry_data(entry) {
                vm.load_streamed_script_bytes(&entry.name, bytes);
            }
        }
        let mut saw_cutscene = false;
        for _ in 0..200 {
            for cmd in vm.world_commands.drain(..) {
                match cmd {
                    ScmWorldCommand::LoadCutscene(_) => {
                        saw_cutscene = true;
                        vm.world_snapshot.cutscene_loaded = true;
                    }
                    ScmWorldCommand::CreatePlayer { pos, .. }
                    | ScmWorldCommand::SetCharCoordinates { pos, .. } => {
                        vm.world_snapshot.player_pos = pos;
                    }
                    _ => {}
                }
            }
            if vm.world_snapshot.cutscene_loaded {
                vm.world_snapshot.cutscene_time_ms += 400;
                if vm.world_snapshot.cutscene_time_ms >= 2000 {
                    vm.world_snapshot.cutscene_finished = true;
                }
            }
            vm.tick(0.05);
        }
        assert!(saw_cutscene);
        assert!(vm.player_actor_handle > 0);
        assert!(vm.bytecode_threads.len() >= 15);
    }
}
