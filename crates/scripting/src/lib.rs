//! Clean-room SCM bytecode virtual machine supporting typed script variables,
//! cooperative script fibers, timers, and core GTA:SA opcode execution.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

#[derive(Debug, Default)]
pub struct ScmVm {
    pub globals: HashMap<usize, ScriptVal>,
    pub threads: Vec<ScriptThread>,
    pub pending_effects: ScriptEffectQueue,
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

            // Execute up to 256 instructions per fiber per frame to prevent infinite loops
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
