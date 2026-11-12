//! Pedestrian AI, traffic lane AI, and 6-star San Andreas Police/Wanted escalation.

use glam::Vec3;
use gtasarust_entities::{PlayerEntity, PlayerLifeState};
use gtasarust_vehicles::{VehicleClass, VehicleManager, VehicleSpawnSpec};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PedBehavior {
    Idle,
    Patrol,
    FleePanic,
    PolicePursuit,
    GangHostile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcPedestrian {
    pub id: u64,
    pub model_id: u32,
    pub position: Vec3,
    pub yaw: f32,
    pub health: f32,
    pub is_cop: bool,
    pub behavior: PedBehavior,
    pub target_point: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrimeEvent {
    WeaponDischarged,
    PedestrianAssaulted,
    VehicleStolen,
    OfficerAttacked,
    Explosion,
}

#[derive(Debug, Clone, Default)]
pub struct AiSimulation {
    pub pedestrians: Vec<NpcPedestrian>,
    pub wanted_heat: f32,
    pub wanted_stars: u8,
    pub wanted_evasion_timer: f32,
    pub arrest_progress: f32,
    next_ped_id: u64,
}

impl AiSimulation {
    pub fn spawn_ped(
        &mut self,
        model_id: u32,
        position: Vec3,
        is_cop: bool,
        behavior: PedBehavior,
    ) -> u64 {
        self.next_ped_id += 1;
        let id = self.next_ped_id;
        self.pedestrians.push(NpcPedestrian {
            id,
            model_id,
            position,
            yaw: 0.0,
            health: 100.0,
            is_cop,
            behavior,
            target_point: position + Vec3::new(15.0, 0.0, 0.0),
        });
        id
    }

    pub fn report_crime(&mut self, crime: CrimeEvent, player: &mut PlayerEntity) {
        let delta = match crime {
            CrimeEvent::WeaponDischarged => 35.0,
            CrimeEvent::PedestrianAssaulted => 75.0,
            CrimeEvent::VehicleStolen => 65.0,
            CrimeEvent::OfficerAttacked => 150.0,
            CrimeEvent::Explosion => 200.0,
        };
        self.wanted_heat = (self.wanted_heat + delta).min(1200.0);
        self.wanted_evasion_timer = 20.0;
        self.recompute_stars(player);
    }

    pub fn alert_gunshot_near(&mut self, origin: Vec3, radius: f32) {
        for ped in &mut self.pedestrians {
            if ped.health <= 0.0 {
                continue;
            }
            if ped.position.distance(origin) <= radius {
                if ped.is_cop {
                    ped.behavior = PedBehavior::PolicePursuit;
                } else {
                    ped.behavior = PedBehavior::FleePanic;
                }
            }
        }
    }

    pub fn tick(&mut self, dt: f32, player: &mut PlayerEntity, vehicles: &mut VehicleManager) {
        // Decay wanted level if player stays unseen / out of cop proximity
        if self.wanted_stars > 0 {
            let cop_near = self
                .pedestrians
                .iter()
                .any(|p| p.is_cop && p.health > 0.0 && p.position.distance(player.position) < 45.0);

            if cop_near {
                self.wanted_evasion_timer = 15.0;
            } else {
                self.wanted_evasion_timer -= dt;
                if self.wanted_evasion_timer <= 0.0 {
                    self.wanted_heat = (self.wanted_heat - 25.0 * dt).max(0.0);
                    self.recompute_stars(player);
                }
            }

            // Escalate police presence up to target count per wanted star
            let active_cops = self
                .pedestrians
                .iter()
                .filter(|p| p.is_cop && p.health > 0.0)
                .count();
            let desired_cops = usize::from(self.wanted_stars) * 2;
            if active_cops < desired_cops {
                let angle = (self.next_ped_id as f32) * 1.3;
                let spawn_pos =
                    player.position + Vec3::new(angle.cos() * 38.0, angle.sin() * 38.0, 0.0);
                self.spawn_ped(280, spawn_pos, true, PedBehavior::PolicePursuit);
                if self.wanted_stars >= 2 && vehicles.vehicles.len() < 16 {
                    vehicles.spawn(VehicleSpawnSpec {
                        model_id: 596,
                        name: "COPCARLA",
                        class: VehicleClass::Emergency,
                        position: spawn_pos + Vec3::new(4.0, 0.0, 0.0),
                        yaw: angle + std::f32::consts::PI,
                        color1: [0.08, 0.08, 0.08],
                        handling: None,
                    });
                }
            }
        }

        // Update pedestrian movement & police arrest logic
        let mut busted = false;
        for ped in &mut self.pedestrians {
            if ped.health <= 0.0 {
                continue;
            }
            match ped.behavior {
                PedBehavior::Idle => {}
                PedBehavior::Patrol => {
                    let to_target = ped.target_point - ped.position;
                    let flat = Vec3::new(to_target.x, to_target.y, 0.0);
                    if flat.length() < 1.0 {
                        let offset = ped.position - Vec3::new(15.0, 0.0, 0.0);
                        ped.target_point = offset;
                    } else {
                        let dir = flat.normalize();
                        ped.position += dir * 1.6 * dt;
                        ped.yaw = dir.y.atan2(dir.x);
                    }
                }
                PedBehavior::FleePanic => {
                    let away = (ped.position - player.position)
                        .truncate()
                        .normalize_or_zero();
                    ped.position.x += away.x * 5.2 * dt;
                    ped.position.y += away.y * 5.2 * dt;
                    ped.yaw = away.y.atan2(away.x);
                }
                PedBehavior::PolicePursuit | PedBehavior::GangHostile => {
                    let to_player = (player.position - ped.position).truncate();
                    let dist = to_player.length();
                    if dist > 1.8 {
                        let dir = to_player / dist;
                        ped.position.x += dir.x * 4.4 * dt;
                        ped.position.y += dir.y * 4.4 * dt;
                        ped.yaw = dir.y.atan2(dir.x);
                    } else if ped.is_cop
                        && self.wanted_stars > 0
                        && player.control_mode == gtasarust_input::ControlMode::OnFoot
                    {
                        self.arrest_progress += dt;
                        if self.arrest_progress >= 2.5 {
                            busted = true;
                        }
                    }
                }
            }
        }

        if busted {
            player.life_state = PlayerLifeState::Busted;
            self.wanted_heat = 0.0;
            self.wanted_stars = 0;
            self.arrest_progress = 0.0;
            player.wanted_stars = 0;
        }
    }

    fn recompute_stars(&mut self, player: &mut PlayerEntity) {
        self.wanted_stars = if self.wanted_heat >= 950.0 {
            6
        } else if self.wanted_heat >= 700.0 {
            5
        } else if self.wanted_heat >= 480.0 {
            4
        } else if self.wanted_heat >= 280.0 {
            3
        } else if self.wanted_heat >= 140.0 {
            2
        } else if self.wanted_heat >= 50.0 {
            1
        } else {
            0
        };
        player.wanted_stars = self.wanted_stars;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crimes_escalate_wanted_stars_and_spawn_police() {
        let mut ai = AiSimulation::default();
        let mut player = PlayerEntity::default();
        let mut vehicles = VehicleManager::default();

        ai.report_crime(CrimeEvent::OfficerAttacked, &mut player);
        assert!(ai.wanted_stars >= 2);
        assert_eq!(player.wanted_stars, ai.wanted_stars);

        ai.tick(0.1, &mut player, &mut vehicles);
        assert!(ai.pedestrians.iter().any(|p| p.is_cop));
    }
}
