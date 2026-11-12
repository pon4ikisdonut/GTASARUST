//! Skeletal animation state machine and IFP clip integration.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AnimState {
    #[default]
    Idle,
    Walk,
    Run,
    Sprint,
    Jump,
    Fall,
    Crouch,
    Swim,
    MeleePunch,
    AimWeapon,
    Drive,
    EnterVehicle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationController {
    pub current: AnimState,
    pub previous: AnimState,
    pub state_time: f32,
    pub blend_alpha: f32,
    pub blend_duration: f32,
}

impl Default for AnimationController {
    fn default() -> Self {
        Self {
            current: AnimState::Idle,
            previous: AnimState::Idle,
            state_time: 0.0,
            blend_alpha: 1.0,
            blend_duration: 0.15,
        }
    }
}

impl AnimationController {
    pub fn transition(&mut self, next: AnimState) {
        if self.current != next {
            self.previous = self.current;
            self.current = next;
            self.state_time = 0.0;
            self.blend_alpha = 0.0;
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.state_time += dt;
        if self.blend_alpha < 1.0 {
            self.blend_alpha = (self.blend_alpha + dt / self.blend_duration).min(1.0);
        }
    }

    /// Map state to original `ped.ifp` animation clip identifier.
    pub fn ifp_clip_name(&self) -> &'static str {
        match self.current {
            AnimState::Idle => "idle_stance",
            AnimState::Walk => "walk_civi",
            AnimState::Run => "run_civi",
            AnimState::Sprint => "sprint_civi",
            AnimState::Jump => "jump_launch",
            AnimState::Fall => "fall_fall",
            AnimState::Crouch => "crouch_roll_l",
            AnimState::Swim => "swim_breast",
            AnimState::MeleePunch => "fighta_1",
            AnimState::AimWeapon => "gun_stand",
            AnimState::Drive => "car_sit",
            AnimState::EnterVehicle => "car_open_lhs",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn animation_blends_on_transition() {
        let mut ctrl = AnimationController::default();
        ctrl.transition(AnimState::Sprint);
        assert_eq!(ctrl.previous, AnimState::Idle);
        assert_eq!(ctrl.blend_alpha, 0.0);
        ctrl.update(0.15);
        assert!((ctrl.blend_alpha - 1.0).abs() < 0.001);
        assert_eq!(ctrl.ifp_clip_name(), "sprint_civi");
    }
}
