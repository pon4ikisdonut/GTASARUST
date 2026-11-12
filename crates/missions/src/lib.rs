//! Campaign mission triggers, active objectives, checkpoints, failure checks,
//! and story progression integrated with the open world and player state.

use glam::Vec3;
use gtasarust_entities::{PlayerEntity, PlayerLifeState};
use gtasarust_scripting::ScmVm;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionTrigger {
    pub id: String,
    pub title: String,
    pub giver: String,
    pub position: Vec3,
    pub radius: f32,
    pub reward_cash: i64,
    pub reward_respect: i32,
    pub prerequisite: Option<String>,
    pub checkpoints: Vec<MissionCheckpoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionCheckpoint {
    pub description: String,
    pub target_pos: Vec3,
    pub radius: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissionBannerState {
    None,
    Passed {
        title: String,
        reward: i64,
        timer: u32,
    },
    Failed {
        title: String,
        reason: String,
        timer: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveMission {
    pub id: String,
    pub title: String,
    pub checkpoint_idx: usize,
    pub checkpoints: Vec<MissionCheckpoint>,
    pub reward_cash: i64,
    pub reward_respect: i32,
}

#[derive(Debug)]
pub struct MissionManager {
    pub triggers: Vec<MissionTrigger>,
    pub active: Option<ActiveMission>,
    pub completed: Vec<String>,
    pub banner_text: Option<String>,
    pub banner_timer: f32,
}

impl Default for MissionManager {
    fn default() -> Self {
        Self::new_san_andreas_campaign()
    }
}

impl MissionManager {
    pub fn new_san_andreas_campaign() -> Self {
        let triggers = vec![
            MissionTrigger {
                id: "SWEET_01_BIG_SMOKE".to_string(),
                title: "Big Smoke / Sweet & Kendl".to_string(),
                giver: "Sweet".to_string(),
                position: Vec3::new(2508.0, -1671.0, 0.0),
                radius: 3.5,
                reward_cash: 250,
                reward_respect: 5,
                prerequisite: None,
                checkpoints: vec![
                    MissionCheckpoint {
                        description: "Ride back to ~y~Grove Street~w~ with Sweet and Ryder."
                            .to_string(),
                        target_pos: Vec3::new(2495.0, -1688.0, 0.0),
                        radius: 4.5,
                    },
                    MissionCheckpoint {
                        description: "Enter the ~y~Johnson House~w~ marker.".to_string(),
                        target_pos: Vec3::new(2495.3, -1690.5, 0.0),
                        radius: 3.0,
                    },
                ],
            },
            MissionTrigger {
                id: "RYDER_01_RYDER".to_string(),
                title: "Ryder".to_string(),
                giver: "Ryder".to_string(),
                position: Vec3::new(2462.0, -1690.0, 0.0),
                radius: 3.5,
                reward_cash: 350,
                reward_respect: 10,
                prerequisite: Some("SWEET_01_BIG_SMOKE".to_string()),
                checkpoints: vec![
                    MissionCheckpoint {
                        description: "Drive to the ~y~Barber Shop~w~ in Idlewood.".to_string(),
                        target_pos: Vec3::new(2074.0, -1793.0, 0.0),
                        radius: 6.0,
                    },
                    MissionCheckpoint {
                        description: "Drive back to ~y~Grove Street~w~.".to_string(),
                        target_pos: Vec3::new(2490.0, -1684.0, 0.0),
                        radius: 5.0,
                    },
                ],
            },
            MissionTrigger {
                id: "SWEET_02_TAGGING_UP_TURF".to_string(),
                title: "Tagging Up Turf".to_string(),
                giver: "Sweet".to_string(),
                position: Vec3::new(2512.0, -1671.0, 0.0),
                radius: 3.5,
                reward_cash: 200,
                reward_respect: 15,
                prerequisite: Some("RYDER_01_RYDER".to_string()),
                checkpoints: vec![
                    MissionCheckpoint {
                        description: "Spray over the Ballas tag in ~y~Idlewood~w~.".to_string(),
                        target_pos: Vec3::new(2125.0, -1750.0, 0.0),
                        radius: 4.5,
                    },
                    MissionCheckpoint {
                        description: "Return to ~y~Grove Street~w~.".to_string(),
                        target_pos: Vec3::new(2495.0, -1688.0, 0.0),
                        radius: 5.0,
                    },
                ],
            },
        ];

        Self {
            triggers,
            active: None,
            completed: Vec::new(),
            banner_text: None,
            banner_timer: 0.0,
        }
    }

    pub fn available_triggers(&self) -> Vec<&MissionTrigger> {
        self.triggers
            .iter()
            .filter(|t| {
                !self.completed.contains(&t.id)
                    && t.prerequisite
                        .as_ref()
                        .is_none_or(|req| self.completed.contains(req))
            })
            .collect()
    }

    pub fn current_objective_text(&self) -> Option<&str> {
        self.active
            .as_ref()
            .and_then(|m| m.checkpoints.get(m.checkpoint_idx))
            .map(|c| c.description.as_str())
    }

    pub fn current_target_position(&self) -> Option<Vec3> {
        self.active
            .as_ref()
            .and_then(|m| m.checkpoints.get(m.checkpoint_idx))
            .map(|c| c.target_pos)
    }

    pub fn tick(&mut self, dt: f32, player: &mut PlayerEntity, vm: &mut ScmVm) {
        if self.banner_timer > 0.0 {
            self.banner_timer = (self.banner_timer - dt).max(0.0);
            if self.banner_timer == 0.0 {
                self.banner_text = None;
            }
        }

        // Apply pending script VM cash/banner effects
        for delta in vm.pending_effects.cash_deltas.drain(..) {
            player.money = (player.money + i64::from(delta)).max(0);
        }
        if let Some(stars) = vm.pending_effects.wanted_override.take() {
            player.wanted_stars = stars;
        }
        for b in vm.pending_effects.banners.drain(..) {
            self.banner_text = Some(b);
            self.banner_timer = 4.0;
        }

        // Handle active mission failure / checkpoint progression
        if let Some(active) = &mut self.active {
            if player.life_state != PlayerLifeState::Alive {
                let reason = match player.life_state {
                    PlayerLifeState::Wasted => "WASTED",
                    PlayerLifeState::Busted => "BUSTED",
                    PlayerLifeState::Alive => "",
                };
                self.banner_text = Some(format!("MISSION FAILED: {} ({reason})", active.title));
                self.banner_timer = 5.0;
                self.active = None;
                return;
            }

            if let Some(cp) = active.checkpoints.get(active.checkpoint_idx) {
                let flat_dist = (player.position.truncate() - cp.target_pos.truncate()).length();
                if flat_dist <= cp.radius {
                    active.checkpoint_idx += 1;
                    if active.checkpoint_idx >= active.checkpoints.len() {
                        player.money += active.reward_cash;
                        player.respect += active.reward_respect;
                        let title = active.title.clone();
                        let cash = active.reward_cash;
                        let id = active.id.clone();
                        self.completed.push(id);
                        self.banner_text = Some(format!("MISSION PASSED! {title} (+${cash})"));
                        self.banner_timer = 5.0;
                        self.active = None;
                    }
                }
            }
            return;
        }

        // Check if player walked into an available mission trigger marker
        if player.life_state == PlayerLifeState::Alive {
            let mut started: Option<MissionTrigger> = None;
            for trig in self.available_triggers() {
                if (player.position.truncate() - trig.position.truncate()).length() <= trig.radius {
                    started = Some(trig.clone());
                    break;
                }
            }
            if let Some(trig) = started {
                self.banner_text = Some(format!("MISSION STARTED: {}", trig.title));
                self.banner_timer = 3.5;
                self.active = Some(ActiveMission {
                    id: trig.id,
                    title: trig.title,
                    checkpoint_idx: 0,
                    checkpoints: trig.checkpoints,
                    reward_cash: trig.reward_cash,
                    reward_respect: trig.reward_respect,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mission_starts_advances_checkpoints_and_rewards_player() {
        let mut mgr = MissionManager::new_san_andreas_campaign();
        let mut vm = ScmVm::default();
        let mut player = PlayerEntity::default();
        let start_money = player.money;

        // Walk into SWEET_01_BIG_SMOKE marker
        player.position = Vec3::new(2508.0, -1671.0, 0.0);
        mgr.tick(0.016, &mut player, &mut vm);
        assert!(mgr.active.is_some());

        // Reach checkpoint 1 & 2
        player.position = Vec3::new(2495.0, -1688.0, 0.0);
        mgr.tick(0.016, &mut player, &mut vm);
        player.position = Vec3::new(2495.3, -1690.5, 0.0);
        mgr.tick(0.016, &mut player, &mut vm);

        assert!(mgr.active.is_none());
        assert_eq!(player.money, start_money + 250);
        assert!(mgr.completed.contains(&"SWEET_01_BIG_SMOKE".to_string()));
    }
}
