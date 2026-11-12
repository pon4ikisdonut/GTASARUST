//! Clean-room SCM bytecode virtual machine supporting both high-level script fibers
//! and compiled GTA: San Andreas `main.scm` chunk/opcode execution.

use gtasarust_gta_formats::scm::{ScmArg, ScmDecodedInstruction, ScmImage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

/// Active GTA:SA `CRunningScript` execution fiber inside a loaded `ScmImage`.
#[derive(Debug, Clone)]
pub struct ScmBytecodeThread {
    pub name: String,
    pub ip: usize,
    pub base_ip: usize,
    pub stack: Vec<usize>,
    pub locals: Vec<i32>,
    pub wait_seconds: f32,
    pub cond_result: bool,
    pub and_or_state: u8,
    pub is_mission: bool,
    pub terminated: bool,
    pub instructions_executed: u64,
}

impl ScmBytecodeThread {
    fn new(name: impl Into<String>, entry_ip: usize, is_mission: bool) -> Self {
        Self {
            name: name.into(),
            ip: entry_ip,
            base_ip: entry_ip,
            stack: Vec::with_capacity(16),
            locals: vec![0i32; 128],
            wait_seconds: 0.0,
            cond_result: false,
            and_or_state: 0,
            is_mission,
            terminated: false,
            instructions_executed: 0,
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
    PrintText {
        key: String,
        duration_ms: i32,
        style: i32,
    },
    LoadScene([f32; 3]),
    DisplayHud(bool),
    DisplayRadar(bool),
    RequestModel(i32),
}

#[derive(Debug)]
pub struct ScmVm {
    pub globals: HashMap<usize, ScriptVal>,
    pub threads: Vec<ScriptThread>,
    pub pending_effects: ScriptEffectQueue,
    pub scm_image: Option<ScmImage>,
    pub scm_globals: Vec<u8>,
    pub bytecode_threads: Vec<ScmBytecodeThread>,
    pub world_commands: Vec<ScmWorldCommand>,
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
            bytecode_threads: Vec::new(),
            world_commands: Vec::new(),
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
        self.bytecode_threads.clear();
        self.bytecode_threads
            .push(ScmBytecodeThread::new("main", entry_ip, false));
        // Execute initial boot commands immediately so world time / player spawn are queued.
        self.step_bytecode_threads(0.0);
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

        let mut newly_spawned: Vec<ScmBytecodeThread> = Vec::new();
        let thread_count = self.bytecode_threads.len();

        for idx in 0..thread_count {
            let mut thread = self.bytecode_threads[idx].clone();
            if thread.terminated {
                continue;
            }
            if thread.wait_seconds > 0.0 {
                thread.wait_seconds -= dt;
                if thread.wait_seconds > 0.0 {
                    self.bytecode_threads[idx] = thread;
                    continue;
                }
            }

            for _ in 0..256 {
                let decoded = {
                    let Some(image) = &self.scm_image else {
                        break;
                    };
                    image.decode_instruction_at(thread.ip)
                };
                let instr = match decoded {
                    Ok(i) => i,
                    Err(err) => {
                        debug!(
                            thread = %thread.name,
                            ip = thread.ip,
                            error = %err,
                            "Suspending SCM bytecode thread on unmapped opcode"
                        );
                        thread.terminated = true;
                        break;
                    }
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
                thread.ip = resolve_jump_target(thread.base_ip, target);
            }
            0x0004 | 0x0006 | 0x0084 | 0x0085 | 0x008A | 0x008B | 0x04AE => {
                // SET int
                if args.len() >= 2 {
                    let v = self.eval_i32(thread, &args[1]);
                    self.write_i32(thread, &args[0], v);
                }
                thread.ip = next;
            }
            0x0005 | 0x0007 | 0x0086 | 0x0087 | 0x008C | 0x008D => {
                // SET float
                if args.len() >= 2 {
                    let v = self.eval_f32(thread, &args[1]);
                    self.write_f32(thread, &args[0], v);
                }
                thread.ip = next;
            }
            0x008E | 0x008F | 0x0092 | 0x0093 => {
                // CSET int from float
                if args.len() >= 2 {
                    let v = self.eval_f32(thread, &args[1]) as i32;
                    self.write_i32(thread, &args[0], v);
                }
                thread.ip = next;
            }
            0x0090 | 0x0091 | 0x0088 | 0x0089 => {
                // CSET float from int
                if args.len() >= 2 {
                    let v = self.eval_i32(thread, &args[1]) as f32;
                    self.write_f32(thread, &args[0], v);
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
            0x0038..=0x003C => {
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
                thread.ip = if thread.cond_result {
                    resolve_jump_target(thread.base_ip, target)
                } else {
                    next
                };
            }
            0x004D => {
                // GOTO_IF_FALSE
                let target = args.first().map(|a| self.eval_i32(thread, a)).unwrap_or(0);
                thread.ip = if !thread.cond_result {
                    resolve_jump_target(thread.base_ip, target)
                } else {
                    next
                };
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
                    let entry_ip = resolve_jump_target(thread.base_ip, target);
                    let mut child = ScmBytecodeThread::new(
                        format!("scr_{entry_ip:05x}"),
                        entry_ip,
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
                    thread.stack.push(next);
                    thread.ip = resolve_jump_target(thread.base_ip, target);
                } else {
                    thread.ip = next;
                }
            }
            0x0051 => {
                // RETURN
                if let Some(ret_ip) = thread.stack.pop() {
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
                    let x = self.eval_f32(thread, &args[1]);
                    let y = self.eval_f32(thread, &args[2]);
                    let z = self.eval_f32(thread, &args[3]);
                    let handle = self.alloc_handle();
                    self.write_i32(thread, &args[4], handle);
                    self.world_commands.push(ScmWorldCommand::CreatePlayer {
                        player_idx,
                        pos: [x, y, z],
                        handle,
                    });
                }
                thread.ip = next;
            }
            0x009A => {
                // CREATE_CHAR(ped_type, model_id, x, y, z, out_handle)
                if args.len() >= 6 {
                    let ped_type = self.eval_i32(thread, &args[0]);
                    let model_id = self.eval_i32(thread, &args[1]);
                    let x = self.eval_f32(thread, &args[2]);
                    let y = self.eval_f32(thread, &args[3]);
                    let z = self.eval_f32(thread, &args[4]);
                    let handle = self.alloc_handle();
                    self.write_i32(thread, &args[5], handle);
                    self.world_commands.push(ScmWorldCommand::CreateChar {
                        ped_type,
                        model_id,
                        pos: [x, y, z],
                        handle,
                    });
                }
                thread.ip = next;
            }
            0x00A5 => {
                // CREATE_CAR(model_id, x, y, z, out_handle)
                if args.len() >= 5 {
                    let model_id = self.eval_i32(thread, &args[0]);
                    let x = self.eval_f32(thread, &args[1]);
                    let y = self.eval_f32(thread, &args[2]);
                    let z = self.eval_f32(thread, &args[3]);
                    let handle = self.alloc_handle();
                    self.write_i32(thread, &args[4], handle);
                    self.world_commands.push(ScmWorldCommand::CreateCar {
                        model_id,
                        pos: [x, y, z],
                        handle,
                    });
                }
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
            0x0109 => {
                // ADD_SCORE(player, amount)
                if args.len() >= 2 {
                    let delta = self.eval_i32(thread, &args[1]);
                    self.pending_effects.cash_deltas.push(delta);
                }
                thread.ip = next;
            }
            0x014B => {
                // CREATE_CAR_GENERATOR(x, y, z, angle, model, c1, c2, force, alarm, door_lock, min_delay, max_delay, out_handle)
                if args.len() >= 13 {
                    let x = self.eval_f32(thread, &args[0]);
                    let y = self.eval_f32(thread, &args[1]);
                    let z = self.eval_f32(thread, &args[2]);
                    let angle_deg = self.eval_f32(thread, &args[3]);
                    let model_id = self.eval_i32(thread, &args[4]);
                    let handle = self.alloc_handle();
                    self.write_i32(thread, &args[12], handle);
                    self.world_commands
                        .push(ScmWorldCommand::CreateCarGenerator {
                            pos: [x, y, z],
                            angle_deg,
                            model_id,
                        });
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
            0x018A => {
                // ADD_BLIP_FOR_COORD(x, y, z, out_handle)
                if args.len() >= 4 {
                    let x = self.eval_f32(thread, &args[0]);
                    let y = self.eval_f32(thread, &args[1]);
                    let z = self.eval_f32(thread, &args[2]);
                    let handle = self.alloc_handle();
                    self.write_i32(thread, &args[3], handle);
                    self.world_commands.push(ScmWorldCommand::AddBlip {
                        kind: ScmBlipKind::Coord,
                        pos: [x, y, z],
                        handle,
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
            0x0213 | 0x032B => {
                // CREATE_PICKUP(model, type, x, y, z, out_handle) / WITH_AMMO
                if args.len() >= 6 {
                    let model_id = self.eval_i32(thread, &args[0]);
                    let pickup_type = self.eval_i32(thread, &args[1]);
                    let x = self.eval_f32(thread, &args[2]);
                    let y = self.eval_f32(thread, &args[3]);
                    let z = self.eval_f32(thread, &args[4]);
                    let handle = self.alloc_handle();
                    self.write_i32(thread, args.last().unwrap(), handle);
                    self.world_commands.push(ScmWorldCommand::CreatePickup {
                        model_id,
                        pickup_type,
                        pos: [x, y, z],
                        handle,
                    });
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
            0x0248 | 0x04EE | 0x0256 => {
                // HAS_MODEL_LOADED / HAS_ANIMATION_LOADED / IS_PLAYER_PLAYING
                update_compare_flag(thread, true, instr.not_flag);
                thread.ip = next;
            }
            0x02A7 | 0x02A8 | 0x04CE | 0x0570 => {
                // ADD_[SHORT_RANGE_]SPRITE_BLIP_FOR_COORD / CONTACT_POINT
                if args.len() >= 5 {
                    let x = self.eval_f32(thread, &args[0]);
                    let y = self.eval_f32(thread, &args[1]);
                    let z = self.eval_f32(thread, &args[2]);
                    let sprite = self.eval_i32(thread, &args[3]);
                    let handle = self.alloc_handle();
                    self.write_i32(thread, &args[4], handle);
                    self.world_commands.push(ScmWorldCommand::AddBlip {
                        kind: ScmBlipKind::Sprite(sprite),
                        pos: [x, y, z],
                        handle,
                    });
                }
                thread.ip = next;
            }
            0x03A4 => {
                // SCRIPT_NAME
                if let Some(s) = args.first().and_then(ScmArg::as_str_literal) {
                    thread.name = s.to_ascii_lowercase();
                }
                thread.ip = next;
            }
            0x03A6 => {
                // DISPLAY_HUD
                if let Some(a) = args.first() {
                    let v = self.eval_i32(thread, a) != 0;
                    self.world_commands.push(ScmWorldCommand::DisplayHud(v));
                }
                thread.ip = next;
            }
            0x03CB => {
                // LOAD_SCENE(x, y, z)
                if args.len() >= 3 {
                    let x = self.eval_f32(thread, &args[0]);
                    let y = self.eval_f32(thread, &args[1]);
                    let z = self.eval_f32(thread, &args[2]);
                    self.world_commands
                        .push(ScmWorldCommand::LoadScene([x, y, z]));
                }
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
                        let mut m_thread =
                            ScmBytecodeThread::new(format!("mission_{idx}"), offset as usize, true);
                        m_thread.base_ip = offset as usize;
                        spawned.push(m_thread);
                        trace!(mission = idx, offset, "Launched SCM mission script");
                    }
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
            0x04D4..=0x04D5 => {
                thread.ip = next;
            }
            0x0913 => {
                // START_NEW_STREAMED_SCRIPT(stream_id, ...args)
                if let Some(a) = args.first() {
                    let sid = self.eval_i32(thread, a);
                    if let Ok(idx) = usize::try_from(sid)
                        && let Some(def) = self
                            .scm_image
                            .as_ref()
                            .and_then(|img| img.streamed_scripts.get(idx))
                        && def.offset > 0
                    {
                        trace!(stream = %def.name, offset = def.offset, "Streamed script start");
                    }
                }
                thread.ip = next;
            }
            _ => {
                // Conditional commands update compare flag to avoid unexpected deadlocks.
                if instr.opcode >= 0x00F0 && instr.name.starts_with("IS_") {
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

fn resolve_jump_target(base_ip: usize, target: i32) -> usize {
    if target >= 0 {
        target as usize
    } else {
        base_ip.saturating_add(target.unsigned_abs() as usize)
    }
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
}
