//! Pedestrian AI, traffic lane AI, and 6-star San Andreas Police/Wanted escalation.

use glam::{Vec2, Vec3};
use gtasarust_entities::{PlayerEntity, PlayerLifeState};
use gtasarust_gta_formats::dat::HandlingCfg;
use gtasarust_input::ActionFrameState;
use gtasarust_physics::PhysicsWorld;
use gtasarust_vehicles::{VehicleClass, VehicleEntity, VehicleManager, VehicleSpawnSpec};
use gtasarust_world::PathGraph;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    #[serde(default)]
    pub path_node: Option<(u16, u16)>,
    #[serde(default)]
    pub prev_path_node: Option<(u16, u16)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficDriver {
    pub vehicle_id: u64,
    pub current_node: (u16, u16),
    pub prev_node: Option<(u16, u16)>,
    pub target_speed_mps: f32,
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
    pub traffic_drivers: HashMap<u64, TrafficDriver>,
    pub wanted_heat: f32,
    pub wanted_stars: u8,
    pub wanted_evasion_timer: f32,
    pub arrest_progress: f32,
    next_ped_id: u64,
    spawn_counter: u64,
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
            path_node: None,
            prev_path_node: None,
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
                    if flat.length() < 1.0 && ped.path_node.is_none() {
                        let offset = ped.position - Vec3::new(15.0, 0.0, 0.0);
                        ped.target_point = offset;
                    } else if flat.length() >= 1.0 {
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

    /// Notify AI when player enters a vehicle (ejecting & panicking any ambient NPC driver).
    pub fn on_vehicle_entered(&mut self, vehicle_id: u64, player: &mut PlayerEntity) {
        if self.traffic_drivers.remove(&vehicle_id).is_some() {
            let side = Vec3::new(-player.yaw.sin(), player.yaw.cos(), 0.0) * 1.8;
            self.spawn_ped(105, player.position + side, false, PedBehavior::FleePanic);
            self.report_crime(CrimeEvent::VehicleStolen, player);
        }
    }

    /// Compute steering and throttle inputs for an AI-driven traffic vehicle following `NODES*.DAT`.
    pub fn traffic_action_for(
        &mut self,
        veh: &VehicleEntity,
        player_pos: Vec3,
        physics: &PhysicsWorld,
        graph: &PathGraph,
    ) -> Option<ActionFrameState> {
        let driver = self.traffic_drivers.get_mut(&veh.id)?;
        let mut target_node = graph.nodes.get(&driver.current_node)?;
        let mut to_target = (target_node.position - veh.position).truncate();
        if to_target.length_squared() < 5.5 * 5.5 {
            self.spawn_counter = self.spawn_counter.wrapping_add(1);
            if let Some(next_key) = graph.pick_next_node(
                driver.current_node,
                driver.prev_node,
                driver.vehicle_id.wrapping_add(self.spawn_counter),
                false,
            ) && let Some(next_node) = graph.nodes.get(&next_key)
            {
                driver.prev_node = Some(driver.current_node);
                driver.current_node = next_key;
                target_node = next_node;
                to_target = (target_node.position - veh.position).truncate();
            }
        }

        let desired_yaw = to_target.y.atan2(to_target.x);
        let mut angle_err = desired_yaw - veh.yaw;
        while angle_err > std::f32::consts::PI {
            angle_err -= std::f32::consts::TAU;
        }
        while angle_err < -std::f32::consts::PI {
            angle_err += std::f32::consts::TAU;
        }

        // `VehicleEntity::step_physics` applies `steer = -move_axis.x`
        let steer_axis = -(angle_err / 0.6).clamp(-1.0, 1.0);
        let fwd = veh.forward();
        let blocked_by_player = {
            let d = player_pos - veh.position;
            let dist = d.truncate().length();
            dist < 11.0 && d.truncate().normalize_or_zero().dot(fwd.truncate()) > 0.72
        };
        let blocked_by_wall = physics
            .raycast(
                veh.position + fwd * 2.5 + Vec3::new(0.0, 0.0, 0.9),
                fwd,
                9.0,
            )
            .is_some_and(|(_, n)| n.z.abs() < 0.5);

        let speed = veh.speed_mps();
        let corner_limit = if angle_err.abs() > 0.55 {
            driver.target_speed_mps * 0.5
        } else {
            driver.target_speed_mps
        };

        let throttle = if blocked_by_player || blocked_by_wall {
            if speed > 0.8 { -0.8 } else { 0.0 }
        } else if speed < corner_limit {
            0.55
        } else {
            0.0
        };

        Some(ActionFrameState {
            move_axis: Vec2::new(steer_axis, throttle),
            jump_or_handbrake: (blocked_by_player || blocked_by_wall) && speed < 2.0,
            ..Default::default()
        })
    }

    /// Update pedestrians on `NODES*.DAT` sidewalks, ground them to `.col` surfaces, and stream ambient traffic.
    pub fn tick_world(
        &mut self,
        dt: f32,
        player: &mut PlayerEntity,
        vehicles: &mut VehicleManager,
        handling: Option<&HandlingCfg>,
        physics: &PhysicsWorld,
        graph: &PathGraph,
    ) {
        self.tick(dt, player, vehicles);

        for ped in &mut self.pedestrians {
            if ped.health <= 0.0 {
                continue;
            }
            if ped.behavior == PedBehavior::Patrol
                && let Some(cur) = ped.path_node
                && ped
                    .position
                    .truncate()
                    .distance_squared(ped.target_point.truncate())
                    < 1.8 * 1.8
            {
                self.spawn_counter = self.spawn_counter.wrapping_add(1);
                if let Some(next_key) = graph.pick_next_node(
                    cur,
                    ped.prev_path_node,
                    ped.id.wrapping_add(self.spawn_counter),
                    true,
                ) && let Some(next_node) = graph.nodes.get(&next_key)
                {
                    ped.prev_path_node = Some(cur);
                    ped.path_node = Some(next_key);
                    ped.target_point = next_node.position;
                }
            }
            if let Some(gz) = physics.ground_height_below(
                ped.position.x,
                ped.position.y,
                ped.position.z + 1.5,
                4.0,
            ) {
                ped.position.z = gz + 0.05;
            }
        }

        if graph.nodes.is_empty() {
            return;
        }

        // Prune distant ambient traffic & sidewalk pedestrians cleanly
        let active_traffic_ids: std::collections::HashSet<u64> =
            self.traffic_drivers.keys().copied().collect();
        vehicles.vehicles.retain(|v| {
            if v.occupied_by_player || !active_traffic_ids.contains(&v.id) {
                return true;
            }
            v.position
                .truncate()
                .distance_squared(player.position.truncate())
                <= 220.0 * 220.0
        });
        let live_veh_ids: std::collections::HashSet<u64> =
            vehicles.vehicles.iter().map(|v| v.id).collect();
        self.traffic_drivers
            .retain(|vid, _| live_veh_ids.contains(vid));

        self.pedestrians.retain(|p| {
            p.path_node.is_none()
                || p.position
                    .truncate()
                    .distance_squared(player.position.truncate())
                    <= 150.0 * 150.0
        });

        // Spawn ambient sidewalk pedestrians
        let path_peds = self
            .pedestrians
            .iter()
            .filter(|p| p.path_node.is_some())
            .count();
        if path_peds < 8 {
            let candidates = graph.nodes_near(player.position, 90.0, true);
            if !candidates.is_empty() {
                self.spawn_counter = self.spawn_counter.wrapping_add(1);
                let k = candidates[(self.spawn_counter as usize) % candidates.len()];
                if let Some(node) = graph.nodes.get(&k)
                    && node
                        .position
                        .truncate()
                        .distance_squared(player.position.truncate())
                        >= 18.0 * 18.0
                    && let Some(next_k) = graph.pick_next_node(k, None, self.spawn_counter, true)
                    && let Some(next_node) = graph.nodes.get(&next_k)
                {
                    let mut spawn_pos = node.position;
                    if let Some(gz) = physics.ground_height_below(
                        spawn_pos.x,
                        spawn_pos.y,
                        spawn_pos.z + 2.0,
                        4.0,
                    ) {
                        spawn_pos.z = gz + 0.05;
                    }
                    let ped_models = [105u32, 106, 107, 9, 14, 21];
                    let mid = ped_models[(self.spawn_counter as usize) % ped_models.len()];
                    let pid = self.spawn_ped(mid, spawn_pos, false, PedBehavior::Patrol);
                    if let Some(ped) = self.pedestrians.last_mut()
                        && ped.id == pid
                    {
                        ped.path_node = Some(next_k);
                        ped.prev_path_node = Some(k);
                        ped.target_point = next_node.position;
                    }
                }
            }
        }

        // Spawn ambient road traffic following `NODES*.DAT` lanes
        if self.traffic_drivers.len() < 6 {
            let candidates = graph.nodes_near(player.position, 130.0, false);
            if !candidates.is_empty() {
                self.spawn_counter = self.spawn_counter.wrapping_add(1);
                let k = candidates[(self.spawn_counter as usize * 7) % candidates.len()];
                if let Some(node) = graph.nodes.get(&k)
                    && node
                        .position
                        .truncate()
                        .distance_squared(player.position.truncate())
                        >= 42.0 * 42.0
                    && !vehicles.vehicles.iter().any(|v| {
                        v.position
                            .truncate()
                            .distance_squared(node.position.truncate())
                            < 16.0 * 16.0
                    })
                    && let Some(next_k) = graph.pick_next_node(k, None, self.spawn_counter, false)
                    && let Some(next_node) = graph.nodes.get(&next_k)
                {
                    const TRAFFIC_MODELS: [(u32, &str, [f32; 3]); 6] = [
                        (420, "TAXI", [0.85, 0.65, 0.12]),
                        (405, "SENTINEL", [0.22, 0.28, 0.38]),
                        (426, "PREMIER", [0.72, 0.72, 0.74]),
                        (445, "ADMIRAL", [0.42, 0.16, 0.18]),
                        (467, "OCEANIC", [0.18, 0.42, 0.30]),
                        (517, "MAJESTIC", [0.25, 0.25, 0.28]),
                    ];
                    let (model_id, name, color1) =
                        TRAFFIC_MODELS[(self.spawn_counter as usize) % TRAFFIC_MODELS.len()];
                    let dir = (next_node.position - node.position).truncate();
                    let yaw = dir.y.atan2(dir.x);
                    let mut pos = node.position;
                    if let Some(gz) = physics.ground_height_below(pos.x, pos.y, pos.z + 2.5, 5.0) {
                        pos.z = gz + 0.08;
                    } else {
                        pos.z += 0.2;
                    }
                    let vid = vehicles.spawn(VehicleSpawnSpec {
                        model_id,
                        name,
                        class: VehicleClass::Car,
                        position: pos,
                        yaw,
                        color1,
                        handling,
                    });
                    self.traffic_drivers.insert(
                        vid,
                        TrafficDriver {
                            vehicle_id: vid,
                            current_node: next_k,
                            prev_node: Some(k),
                            target_speed_mps: 12.0,
                        },
                    );
                }
            }
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

    #[test]
    fn traffic_driver_steers_toward_path_node() {
        let mut graph = PathGraph::default();
        graph.nodes.insert(
            (15, 0),
            gtasarust_world::WorldPathNode {
                position: Vec3::new(2480.0, -1668.0, 12.5),
                neighbors: vec![(15, 1)],
                is_ped: false,
                is_water: false,
                is_highway: false,
            },
        );
        graph.nodes.insert(
            (15, 1),
            gtasarust_world::WorldPathNode {
                position: Vec3::new(2520.0, -1668.0, 12.5),
                neighbors: vec![(15, 0)],
                is_ped: false,
                is_water: false,
                is_highway: false,
            },
        );
        graph.areas.insert(15, vec![(15, 0), (15, 1)]);

        let mut vehicles = VehicleManager::default();
        let vid = vehicles.spawn(VehicleSpawnSpec {
            model_id: 420,
            name: "TAXI",
            class: VehicleClass::Car,
            position: Vec3::new(2480.0, -1668.0, 0.2),
            yaw: 0.4,
            color1: [0.85, 0.65, 0.12],
            handling: None,
        });
        let mut ai = AiSimulation::default();
        ai.traffic_drivers.insert(
            vid,
            TrafficDriver {
                vehicle_id: vid,
                current_node: (15, 1),
                prev_node: Some((15, 0)),
                target_speed_mps: 12.0,
            },
        );

        let physics = PhysicsWorld::new();
        let act = ai
            .traffic_action_for(&vehicles.vehicles[0], Vec3::ZERO, &physics, &graph)
            .expect("traffic action should compute");
        assert!(act.move_axis.y > 0.2);
        // desired_yaw=0, current yaw=0.4 -> negative yaw delta -> positive move_axis.x -> negative steer
        assert!(act.move_axis.x > 0.1);
    }
}
