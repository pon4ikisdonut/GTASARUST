//! Complete vehicle simulation supporting cars, motorcycles, bicycles, trucks,
//! buses, boats, aircraft, helicopters, and emergency vehicles driven by `handling.cfg`.

use glam::{Quat, Vec3};
use gtasarust_entities::{PlayerEntity, VehicleId};
use gtasarust_gta_formats::dat::{HandlingCfg, VehicleHandling};
use gtasarust_input::{ActionFrameState, ControlMode};
use gtasarust_physics::PhysicsWorld;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VehicleClass {
    Car,
    Motorcycle,
    Bicycle,
    Truck,
    Bus,
    Boat,
    Plane,
    Helicopter,
    Emergency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleParams {
    pub class: VehicleClass,
    pub max_speed_mps: f32,
    pub acceleration: f32,
    pub brake_deceleration: f32,
    pub steering_rate: f32,
    pub drag: f32,
    pub mass_kg: f32,
    pub suspension_height: f32,
}

impl VehicleParams {
    pub fn from_handling(class: VehicleClass, h: &VehicleHandling) -> Self {
        Self {
            class,
            max_speed_mps: (h.max_velocity / 3.6).max(12.0),
            acceleration: (h.engine_acceleration * 0.8).max(4.0),
            brake_deceleration: h.brake_deceleration.max(5.0),
            steering_rate: (h.steering_lock.to_radians() * 2.8).max(1.2),
            drag: (h.drag_mult * 0.25).clamp(0.15, 2.5),
            mass_kg: h.mass.max(50.0),
            suspension_height: 0.5,
        }
    }

    pub fn preset(class: VehicleClass) -> Self {
        match class {
            VehicleClass::Car => Self {
                class,
                max_speed_mps: 48.0,
                acceleration: 18.0,
                brake_deceleration: 22.0,
                steering_rate: 2.2,
                drag: 0.35,
                mass_kg: 1450.0,
                suspension_height: 0.5,
            },
            VehicleClass::Motorcycle => Self {
                class,
                max_speed_mps: 52.0,
                acceleration: 24.0,
                brake_deceleration: 26.0,
                steering_rate: 3.1,
                drag: 0.28,
                mass_kg: 240.0,
                suspension_height: 0.45,
            },
            VehicleClass::Bicycle => Self {
                class,
                max_speed_mps: 18.0,
                acceleration: 9.0,
                brake_deceleration: 16.0,
                steering_rate: 3.4,
                drag: 0.55,
                mass_kg: 22.0,
                suspension_height: 0.4,
            },
            VehicleClass::Truck | VehicleClass::Bus => Self {
                class,
                max_speed_mps: 32.0,
                acceleration: 8.5,
                brake_deceleration: 13.0,
                steering_rate: 1.35,
                drag: 0.65,
                mass_kg: 5200.0,
                suspension_height: 0.7,
            },
            VehicleClass::Boat => Self {
                class,
                max_speed_mps: 38.0,
                acceleration: 12.0,
                brake_deceleration: 8.0,
                steering_rate: 1.8,
                drag: 0.75,
                mass_kg: 1800.0,
                suspension_height: 0.2,
            },
            VehicleClass::Plane => Self {
                class,
                max_speed_mps: 85.0,
                acceleration: 22.0,
                brake_deceleration: 12.0,
                steering_rate: 1.6,
                drag: 0.18,
                mass_kg: 3200.0,
                suspension_height: 0.8,
            },
            VehicleClass::Helicopter => Self {
                class,
                max_speed_mps: 58.0,
                acceleration: 16.0,
                brake_deceleration: 14.0,
                steering_rate: 2.0,
                drag: 0.4,
                mass_kg: 2600.0,
                suspension_height: 0.7,
            },
            VehicleClass::Emergency => Self {
                class,
                max_speed_mps: 50.0,
                acceleration: 20.0,
                brake_deceleration: 24.0,
                steering_rate: 2.4,
                drag: 0.32,
                mass_kg: 1700.0,
                suspension_height: 0.55,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleEntity {
    pub id: VehicleId,
    pub model_id: u32,
    pub name: String,
    pub position: Vec3,
    pub velocity: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
    pub health: f32,
    pub engine_on: bool,
    pub lights_on: bool,
    pub siren_on: bool,
    pub horn_active: bool,
    pub color1: [f32; 3],
    pub color2: [f32; 3],
    pub occupied_by_player: bool,
    /// Accumulated wheel rolling rotation in radians.
    #[serde(default)]
    pub wheel_spin: f32,
    /// Accumulated helicopter rotor rotation in radians.
    #[serde(default)]
    pub rotor_spin: f32,
    /// Current front wheel steering angle in radians.
    #[serde(default)]
    pub steer_angle: f32,
    #[serde(default)]
    pub exploded: bool,
    pub params: VehicleParams,
}

impl VehicleEntity {
    pub fn collider_half_extents(&self) -> Vec3 {
        match self.params.class {
            VehicleClass::Bicycle | VehicleClass::Motorcycle => Vec3::new(0.95, 0.36, 0.55),
            VehicleClass::Truck | VehicleClass::Bus => Vec3::new(3.2, 1.25, 1.3),
            VehicleClass::Boat => Vec3::new(2.6, 1.15, 0.8),
            VehicleClass::Plane | VehicleClass::Helicopter => Vec3::new(3.0, 1.4, 1.2),
            _ => Vec3::new(2.1, 0.96, 0.74),
        }
    }

    pub fn rotation_quat(&self) -> Quat {
        Quat::from_euler(glam::EulerRot::ZYX, self.yaw, self.pitch, self.roll)
    }

    pub fn forward(&self) -> Vec3 {
        Vec3::new(self.yaw.cos(), self.yaw.sin(), 0.0).normalize()
    }

    pub fn speed_mps(&self) -> f32 {
        self.velocity.length()
    }

    pub fn apply_damage(&mut self, dmg: f32) {
        self.health = (self.health - dmg.max(0.0)).max(0.0);
        if self.health <= 0.0 {
            self.engine_on = false;
        }
    }

    /// Integrate vehicle dynamics using class-appropriate physics (cars vs bikes vs planes vs boats).
    pub fn step_physics(
        &mut self,
        dt: f32,
        actions: Option<&ActionFrameState>,
        physics: &PhysicsWorld,
    ) {
        let _ = self.step_physics_with_context(dt, actions, physics, None, None);
    }

    /// Integrate vehicle dynamics with optional collider exclusion tag, water level,
    /// and burning-to-explosion detection (`health < 250.0` catches fire, returns `true` on detonation).
    pub fn step_physics_with_context(
        &mut self,
        dt: f32,
        actions: Option<&ActionFrameState>,
        physics: &PhysicsWorld,
        excluded_tag: Option<u128>,
        water_z: Option<f32>,
    ) -> bool {
        let mut detonated = false;
        let in_deep_water = water_z.is_some_and(|wz| self.position.z < wz - 0.65);

        if !self.exploded && self.health < 250.0 && !in_deep_water {
            self.health = (self.health - 45.0 * dt).max(0.0);
        }
        if !self.exploded && self.health <= 0.0 {
            self.exploded = true;
            self.engine_on = false;
            self.lights_on = false;
            self.siren_on = false;
            self.velocity.z += 5.5;
            self.color1 = [0.12, 0.12, 0.12];
            self.color2 = [0.10, 0.10, 0.10];
            detonated = true;
        }
        if in_deep_water && !matches!(self.params.class, VehicleClass::Boat) {
            self.engine_on = false;
        }

        let forward = self.forward();
        let right = Vec3::new(forward.y, -forward.x, 0.0);
        let mut forward_speed = self.velocity.dot(forward);

        if let Some(act) = actions
            && !self.exploded
            && self.health > 0.0
            && (!in_deep_water || self.params.class == VehicleClass::Boat)
        {
            self.engine_on = true;
            self.horn_active = act.crouch_or_horn;

            let throttle = act.move_axis.y;
            let steer = -act.move_axis.x;

            match self.params.class {
                VehicleClass::Plane => {
                    if throttle > 0.0 {
                        forward_speed += throttle * self.params.acceleration * dt;
                    } else if throttle < 0.0 {
                        forward_speed += throttle * self.params.brake_deceleration * dt;
                    }
                    self.yaw += steer * self.params.steering_rate * dt;
                    // Lift once airspeed exceeds takeoff threshold (22 m/s)
                    if forward_speed > 22.0 && act.jump_or_handbrake {
                        self.velocity.z = (self.velocity.z + 12.0 * dt).clamp(-10.0, 14.0);
                    } else if self.position.z > 1.0 {
                        self.velocity.z -= 4.5 * dt;
                    }
                }
                VehicleClass::Helicopter => {
                    if throttle != 0.0 {
                        forward_speed += throttle * self.params.acceleration * dt;
                    }
                    self.yaw += steer * self.params.steering_rate * dt;
                    if act.sprint {
                        self.velocity.z = (self.velocity.z + 16.0 * dt).clamp(-12.0, 12.0);
                    } else if act.jump_or_handbrake {
                        self.velocity.z = (self.velocity.z - 14.0 * dt).clamp(-12.0, 12.0);
                    } else {
                        self.velocity.z *= (1.0 - 2.5 * dt).max(0.0);
                    }
                    let target_pitch = -throttle * 0.22;
                    let target_roll = steer * 0.28;
                    self.pitch += (target_pitch - self.pitch) * (5.0 * dt).min(1.0);
                    self.roll += (target_roll - self.roll) * (5.0 * dt).min(1.0);
                }
                _ => {
                    // Ground / water vehicles
                    if throttle > 0.05 {
                        forward_speed += throttle * self.params.acceleration * dt;
                    } else if throttle < -0.05 {
                        if forward_speed > 1.0 {
                            forward_speed -= self.params.brake_deceleration * dt;
                        } else {
                            forward_speed += throttle * self.params.acceleration * 0.55 * dt;
                        }
                    }

                    if act.jump_or_handbrake {
                        forward_speed *= (1.0 - 4.5 * dt).max(0.0);
                    }

                    // Turn only proportionally when moving (cars/trucks/bikes)
                    let speed_factor = (forward_speed.abs() / 6.0).clamp(0.0, 1.0);
                    let turn_sign = if forward_speed >= -0.1 { 1.0 } else { -1.0 };
                    self.yaw += steer * self.params.steering_rate * speed_factor * turn_sign * dt;

                    // Lean motorcycles/bicycles visually during turns
                    if matches!(
                        self.params.class,
                        VehicleClass::Motorcycle | VehicleClass::Bicycle
                    ) {
                        self.roll = steer * 0.35 * speed_factor;
                    } else {
                        self.roll = 0.0;
                    }
                }
            }
        } else {
            self.horn_active = false;
            if self.params.class == VehicleClass::Helicopter {
                self.pitch *= (1.0 - 4.0 * dt).max(0.0);
                self.roll *= (1.0 - 4.0 * dt).max(0.0);
            }
        }

        if self.params.class == VehicleClass::Helicopter && !self.exploded {
            let spin_speed = if self.occupied_by_player {
                22.0
            } else if self.engine_on {
                10.0
            } else {
                0.0
            };
            self.rotor_spin = (self.rotor_spin + spin_speed * dt).rem_euclid(std::f32::consts::TAU);
        }

        // Apply aerodynamic drag and clamp to max velocity
        forward_speed *= (1.0 - self.params.drag * dt).max(0.0);
        forward_speed =
            forward_speed.clamp(-self.params.max_speed_mps * 0.35, self.params.max_speed_mps);

        // Lateral tire friction kills sideways drift cleanly
        let mut lateral = self.velocity.dot(right);
        let grip = if actions.is_some_and(|a| a.jump_or_handbrake) {
            2.0
        } else {
            9.5
        };
        lateral *= (1.0 - grip * dt).max(0.0);

        let vz = self.velocity.z;
        self.velocity = forward * forward_speed + right * lateral;
        self.velocity.z = vz;

        if !matches!(
            self.params.class,
            VehicleClass::Plane | VehicleClass::Helicopter | VehicleClass::Boat
        ) {
            self.velocity.z -= 9.81 * dt;
        }

        // Horizontal wall & vehicle impact detection against colliders (excluding self)
        let ex_tag = excluded_tag.unwrap_or(0);
        if !matches!(
            self.params.class,
            VehicleClass::Plane | VehicleClass::Helicopter | VehicleClass::Boat
        ) {
            let horiz = Vec3::new(self.velocity.x, self.velocity.y, 0.0);
            let h_speed = horiz.length();
            if h_speed > 0.1 {
                let dir = horiz / h_speed;
                let bumper_origin = self.position + Vec3::new(0.0, 0.0, 0.85);
                let bumper_dist = 1.6 + h_speed * dt;
                if let Some((toi, hit_n, _)) =
                    physics.raycast_excluding(bumper_origin, dir, bumper_dist, ex_tag)
                    && hit_n.z.abs() < 0.5
                {
                    let horiz_n = Vec3::new(hit_n.x, hit_n.y, 0.0).normalize_or_zero();
                    let into = horiz.dot(horiz_n);
                    if into < 0.0 {
                        self.apply_damage((-into - 5.0).max(0.0) * 3.0);
                        let deflected = horiz - horiz_n * (into * 1.25);
                        self.velocity.x = deflected.x;
                        self.velocity.y = deflected.y;
                    }
                    if toi < 1.5 {
                        self.position += horiz_n * (1.5 - toi);
                    }
                }
            }
        }

        self.position += self.velocity * dt;

        if self.params.class == VehicleClass::Boat {
            let target_z = water_z.unwrap_or(0.0) - 0.15;
            if self.position.z < target_z {
                self.position.z = target_z;
                self.velocity.z = 0.0;
            }
        } else {
            // Ground / water vehicles
            self.wheel_spin =
                (self.wheel_spin + (forward_speed / 0.36) * dt).rem_euclid(std::f32::consts::TAU);
            if let Some(act) = actions {
                let target_steer =
                    -act.move_axis.x * (self.params.steering_rate * 0.22).clamp(0.25, 0.58);
                self.steer_angle += (target_steer - self.steer_angle) * (10.0 * dt).min(1.0);
            } else {
                self.steer_angle *= (1.0 - 8.0 * dt).max(0.0);
            }

            if in_deep_water {
                // Water drag damps sinking cars smoothly
                self.velocity *= (1.0 - 2.8 * dt).max(0.0);
            }

            let probe = self.position + Vec3::new(0.0, 0.0, 1.5);
            let probe_dist = 2.5 + (-self.velocity.z * dt).max(0.0);
            if let Some((toi, _, _)) =
                physics.raycast_excluding(probe, Vec3::NEG_Z, probe_dist, ex_tag)
            {
                let ground_z = probe.z - toi;
                if ground_z <= self.position.z + 0.8
                    && self.position.z <= ground_z + 0.08
                    && self.velocity.z <= 0.0
                {
                    self.position.z = ground_z;
                    self.velocity.z = 0.0;
                }
            } else if self.position.z < 0.0 && water_z.is_none() {
                self.position.z = 0.0;
                self.velocity.z = 0.0;
            }

            // Sample front/rear & left/right wheel contact heights against `.col` to pitch/roll smoothly with hills
            if !matches!(
                self.params.class,
                VehicleClass::Motorcycle | VehicleClass::Bicycle
            ) {
                let fwd = self.forward();
                let pos = self.position;
                let probe_offset = |local_x: f32, local_y: f32| -> Option<f32> {
                    let world_off = fwd * local_y + right * local_x;
                    let origin = pos + world_off + Vec3::new(0.0, 0.0, 1.4);
                    physics
                        .raycast_excluding(origin, Vec3::NEG_Z, 2.4, ex_tag)
                        .map(|(d, _, _)| origin.z - d)
                };
                if let (Some(zf), Some(zb)) = (probe_offset(0.0, 1.25), probe_offset(0.0, -1.25)) {
                    let target_pitch = ((zb - zf) / 2.5).atan().clamp(-0.55, 0.55);
                    self.pitch += (target_pitch - self.pitch) * (9.0 * dt).min(1.0);
                }
                if let (Some(zl), Some(zr)) = (probe_offset(-0.8, 0.0), probe_offset(0.8, 0.0)) {
                    let target_roll = ((zl - zr) / 1.6).atan().clamp(-0.45, 0.45);
                    self.roll += (target_roll - self.roll) * (9.0 * dt).min(1.0);
                }
            }
        }
        detonated
    }
}

/// Spawn parameters for creating a vehicle entity in [`VehicleManager::spawn`].
#[derive(Debug, Clone)]
pub struct VehicleSpawnSpec<'a> {
    pub model_id: u32,
    pub name: &'a str,
    pub class: VehicleClass,
    pub position: Vec3,
    pub yaw: f32,
    pub color1: [f32; 3],
    pub handling: Option<&'a HandlingCfg>,
}

#[derive(Debug, Default)]
pub struct VehicleManager {
    pub vehicles: Vec<VehicleEntity>,
    next_id: VehicleId,
}

impl VehicleManager {
    pub fn spawn(&mut self, spec: VehicleSpawnSpec<'_>) -> VehicleId {
        self.next_id += 1;
        let id = self.next_id;
        let name_s = spec.name.to_string();
        let params = spec
            .handling
            .and_then(|cfg| cfg.vehicles.get(&name_s.to_ascii_uppercase()))
            .map(|h| VehicleParams::from_handling(spec.class, h))
            .unwrap_or_else(|| VehicleParams::preset(spec.class));

        self.vehicles.push(VehicleEntity {
            id,
            model_id: spec.model_id,
            name: name_s,
            position: spec.position,
            velocity: Vec3::ZERO,
            yaw: spec.yaw,
            pitch: 0.0,
            roll: 0.0,
            health: 1000.0,
            engine_on: false,
            lights_on: false,
            siren_on: spec.class == VehicleClass::Emergency,
            horn_active: false,
            color1: spec.color1,
            color2: [0.9, 0.9, 0.9],
            occupied_by_player: false,
            wheel_spin: 0.0,
            rotor_spin: 0.0,
            steer_angle: 0.0,
            exploded: false,
            params,
        });
        id
    }

    pub fn get_mut(&mut self, id: VehicleId) -> Option<&mut VehicleEntity> {
        self.vehicles.iter_mut().find(|v| v.id == id)
    }

    pub fn get(&self, id: VehicleId) -> Option<&VehicleEntity> {
        self.vehicles.iter().find(|v| v.id == id)
    }

    /// Toggle player entering nearest vehicle (within 4m) or exiting current vehicle.
    pub fn toggle_player_entry(&mut self, player: &mut PlayerEntity) -> bool {
        if let Some(vid) = player.current_vehicle {
            if let Some(v) = self.get_mut(vid) {
                v.occupied_by_player = false;
                let left_offset = Vec3::new(-v.yaw.sin(), v.yaw.cos(), 0.0) * 2.2;
                player.position = v.position + left_offset;
            }
            player.current_vehicle = None;
            player.control_mode = ControlMode::OnFoot;
            return true;
        }

        let mut best: Option<(usize, f32)> = None;
        for (idx, v) in self.vehicles.iter().enumerate() {
            if v.exploded {
                continue;
            }
            let d = v.position.distance(player.position);
            if d <= 4.5 && best.is_none_or(|(_, bd)| d < bd) {
                best = Some((idx, d));
            }
        }

        if let Some((idx, _)) = best {
            let v = &mut self.vehicles[idx];
            v.occupied_by_player = true;
            v.engine_on = true;
            player.current_vehicle = Some(v.id);
            player.control_mode = ControlMode::InVehicle;
            player.position = v.position;
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entering_vehicle_changes_control_mode_and_moves_vehicle() {
        let physics = PhysicsWorld::new();
        let mut mgr = VehicleManager::default();
        let vid = mgr.spawn(VehicleSpawnSpec {
            model_id: 492,
            name: "GREENWOO",
            class: VehicleClass::Car,
            position: Vec3::new(2497.0, -1688.0, 0.0),
            yaw: 0.0,
            color1: [0.15, 0.45, 0.22],
            handling: None,
        });
        let mut player = PlayerEntity {
            position: Vec3::new(2495.5, -1688.0, 0.0),
            ..Default::default()
        };

        assert!(mgr.toggle_player_entry(&mut player));
        assert_eq!(player.control_mode, ControlMode::InVehicle);
        assert_eq!(player.current_vehicle, Some(vid));

        let drive = ActionFrameState {
            move_axis: glam::Vec2::new(0.0, 1.0),
            ..Default::default()
        };
        let v = mgr.get_mut(vid).unwrap();
        v.step_physics(0.5, Some(&drive), &physics);
        assert!(v.speed_mps() > 2.0);
    }

    #[test]
    fn burning_vehicle_below_250_hp_detonates_once() {
        let physics = PhysicsWorld::new();
        let mut mgr = VehicleManager::default();
        let vid = mgr.spawn(VehicleSpawnSpec {
            model_id: 411,
            name: "INFERNUS",
            class: VehicleClass::Car,
            position: Vec3::ZERO,
            yaw: 0.0,
            color1: [0.8, 0.1, 0.1],
            handling: None,
        });
        let v = mgr.get_mut(vid).unwrap();
        v.apply_damage(800.0); // 200.0 HP left (< 250 catches fire)
        let mut explosions = 0;
        for _ in 0..80 {
            if v.step_physics_with_context(0.1, None, &physics, None, None) {
                explosions += 1;
            }
        }
        assert_eq!(explosions, 1);
        assert!(v.exploded);
        assert_eq!(v.health, 0.0);
    }
}
