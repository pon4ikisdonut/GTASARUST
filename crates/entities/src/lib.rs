//! Player character (CJ), pedestrians, pickups, and on-foot physics/camera movement.

use glam::Vec3;
use gtasarust_animation::{AnimState, AnimationController};
use gtasarust_input::{ActionFrameState, ControlMode};
use gtasarust_physics::PhysicsWorld;
use gtasarust_weapons::WeaponInventory;
use serde::{Deserialize, Serialize};

pub type VehicleId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerLifeState {
    Alive,
    Wasted,
    Busted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerEntity {
    pub position: Vec3,
    pub velocity: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub health: f32,
    pub max_health: f32,
    pub armor: f32,
    pub stamina: f32,
    pub max_stamina: f32,
    pub money: i64,
    pub respect: i32,
    pub wanted_stars: u8,
    pub grounded: bool,
    pub crouched: bool,
    pub in_water: bool,
    pub control_mode: ControlMode,
    pub current_vehicle: Option<VehicleId>,
    pub life_state: PlayerLifeState,
    pub weapons: WeaponInventory,
    pub anim: AnimationController,
}

impl Default for PlayerEntity {
    fn default() -> Self {
        Self {
            // Grove Street cul-de-sac spawn
            position: Vec3::new(2495.3, -1688.2, 1.0),
            velocity: Vec3::ZERO,
            yaw: 0.0,
            pitch: -0.15,
            health: 100.0,
            max_health: 100.0,
            armor: 0.0,
            stamina: 100.0,
            max_stamina: 100.0,
            money: 350,
            respect: 0,
            wanted_stars: 0,
            grounded: true,
            crouched: false,
            in_water: false,
            control_mode: ControlMode::OnFoot,
            current_vehicle: None,
            life_state: PlayerLifeState::Alive,
            weapons: WeaponInventory::default(),
            anim: AnimationController::default(),
        }
    }
}

impl PlayerEntity {
    /// Update camera yaw/pitch, on-foot locomotion, jumping, swimming, crouching, and weapon timers.
    pub fn update_on_foot(&mut self, dt: f32, actions: &ActionFrameState, physics: &PhysicsWorld) {
        self.weapons.tick(dt);
        self.anim.update(dt);

        if self.life_state != PlayerLifeState::Alive {
            return;
        }

        // Camera orbit look
        self.yaw -= actions.look_delta.x * 0.005;
        self.pitch = (self.pitch - actions.look_delta.y * 0.005).clamp(-1.25, 1.15);

        if actions.next_weapon {
            self.weapons.cycle_next();
        }
        if actions.prev_weapon {
            self.weapons.cycle_prev();
        }
        if actions.crouch_or_horn && self.grounded {
            self.crouched = actions.crouch_or_horn;
        } else {
            self.crouched = false;
        }

        if self.control_mode != ControlMode::OnFoot {
            self.anim.transition(AnimState::Drive);
            return;
        }

        // Camera-relative horizontal basis in Z-up world coordinates
        let forward = Vec3::new(self.yaw.cos(), self.yaw.sin(), 0.0).normalize();
        let right = Vec3::new(forward.y, -forward.x, 0.0);
        let wish_dir =
            (forward * actions.move_axis.y + right * actions.move_axis.x).normalize_or_zero();

        let sprinting = actions.sprint && wish_dir.length_squared() > 0.1 && self.stamina > 2.0;
        let speed = if self.in_water {
            3.2
        } else if self.crouched {
            2.4
        } else if sprinting {
            self.stamina = (self.stamina - dt * 18.0).max(0.0);
            8.2
        } else {
            self.stamina = (self.stamina + dt * 14.0).min(self.max_stamina);
            4.8
        };

        self.velocity.x = wish_dir.x * speed;
        self.velocity.y = wish_dir.y * speed;

        if self.grounded && actions.jump_or_handbrake && !self.in_water {
            self.velocity.z = 5.4;
            self.grounded = false;
            self.anim.transition(AnimState::Jump);
        }

        if !self.grounded {
            self.velocity.z -= 18.0 * dt;
        }

        self.position += self.velocity * dt;

        // Ground collision probe using Rapier3D physics world
        let ray_origin = self.position + Vec3::new(0.0, 0.0, 1.1);
        if let Some((toi, _normal)) = physics.raycast(ray_origin, Vec3::NEG_Z, 2.2) {
            let ground_z = ray_origin.z - toi;
            if self.position.z <= ground_z + 0.05 && self.velocity.z <= 0.0 {
                self.position.z = ground_z;
                self.velocity.z = 0.0;
                self.grounded = true;
            } else {
                self.grounded = false;
            }
        } else if self.position.z < 0.0 {
            self.position.z = 0.0;
            self.velocity.z = 0.0;
            self.grounded = true;
        }

        // Update animation state machine
        if self.in_water {
            self.anim.transition(AnimState::Swim);
        } else if !self.grounded {
            if self.velocity.z < -2.0 {
                self.anim.transition(AnimState::Fall);
            }
        } else if actions.aim_or_reverse {
            self.anim.transition(AnimState::AimWeapon);
        } else if self.crouched {
            self.anim.transition(AnimState::Crouch);
        } else if sprinting {
            self.anim.transition(AnimState::Sprint);
        } else if wish_dir.length_squared() > 0.1 {
            self.anim.transition(AnimState::Run);
        } else {
            self.anim.transition(AnimState::Idle);
        }
    }

    pub fn apply_damage(&mut self, amount: f32) {
        if self.life_state != PlayerLifeState::Alive || amount <= 0.0 {
            return;
        }
        let armor_absorbed = self.armor.min(amount);
        self.armor -= armor_absorbed;
        let rem = amount - armor_absorbed;
        self.health = (self.health - rem).max(0.0);
        if self.health <= 0.0 {
            self.life_state = PlayerLifeState::Wasted;
        }
    }

    pub fn respawn_at_hospital(&mut self, pos: Vec3) {
        self.position = pos;
        self.velocity = Vec3::ZERO;
        self.health = self.max_health;
        self.wanted_stars = 0;
        self.money = (self.money - 100).max(0);
        self.control_mode = ControlMode::OnFoot;
        self.current_vehicle = None;
        self.life_state = PlayerLifeState::Alive;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PickupEntity {
    pub id: u64,
    pub position: Vec3,
    pub kind: PickupKind,
    pub respawn_timer: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PickupKind {
    Health,
    Armor,
    Cash(i64),
    Weapon { name: String, ammo: u32 },
    BribeStar,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_moves_forward_and_takes_damage_through_armor() {
        let physics = PhysicsWorld::new();
        let mut p = PlayerEntity::default();
        let start = p.position;
        let actions = ActionFrameState {
            move_axis: glam::Vec2::new(0.0, 1.0),
            sprint: true,
            ..Default::default()
        };
        p.update_on_foot(0.5, &actions, &physics);
        assert!(p.position.x > start.x);
        assert_eq!(p.anim.current, AnimState::Sprint);

        p.armor = 50.0;
        p.apply_damage(70.0);
        assert_eq!(p.armor, 0.0);
        assert_eq!(p.health, 80.0);
    }
}
