//! Unified keyboard, mouse, and gamepad (`gilrs`) input system with
//! San Andreas action contexts (`OnFoot`, `InVehicle`, `Menu`).

use gilrs::{Axis, Button, EventType, Gilrs};
use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::{info, warn};
use winit::event::{ElementState, MouseButton};
use winit::keyboard::{KeyCode, PhysicalKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlMode {
    OnFoot,
    InVehicle,
    Menu,
}

#[derive(Debug, Clone, Default)]
pub struct ActionFrameState {
    pub move_axis: Vec2,  // x = strafe/steer, y = forward/throttle
    pub look_delta: Vec2, // mouse/right-stick camera delta
    pub sprint: bool,
    pub jump_or_handbrake: bool,
    pub crouch_or_horn: bool,
    pub fire_or_attack: bool,
    pub aim_or_reverse: bool,
    pub enter_exit_vehicle: bool,
    pub enter_exit_just_pressed: bool,
    pub skip_cutscene_just_pressed: bool,
    pub next_weapon: bool,
    pub prev_weapon: bool,
    pub radio_next: bool,
    pub toggle_pause: bool,
    pub toggle_dev_overlay: bool,
    pub toggle_map: bool,
}

pub struct InputManager {
    pressed_keys: HashSet<KeyCode>,
    just_pressed_keys: HashSet<KeyCode>,
    pressed_mouse: HashSet<MouseButton>,
    mouse_delta: Vec2,
    gamepad_move: Vec2,
    gamepad_look: Vec2,
    gamepad_deadzone: f32,
    gilrs: Option<Gilrs>,
}

impl InputManager {
    pub fn new(gamepad_deadzone: f32) -> Self {
        let gilrs = match Gilrs::new() {
            Ok(g) => {
                info!("Gamepad input initialized via gilrs");
                Some(g)
            }
            Err(err) => {
                warn!("Gamepad subsystem unavailable ({err}); continuing with keyboard/mouse");
                None
            }
        };

        Self {
            pressed_keys: HashSet::new(),
            just_pressed_keys: HashSet::new(),
            pressed_mouse: HashSet::new(),
            mouse_delta: Vec2::ZERO,
            gamepad_move: Vec2::ZERO,
            gamepad_look: Vec2::ZERO,
            gamepad_deadzone: gamepad_deadzone.clamp(0.0, 0.5),
            gilrs,
        }
    }

    pub fn handle_key_event(&mut self, physical_key: PhysicalKey, state: ElementState) {
        if let PhysicalKey::Code(code) = physical_key {
            match state {
                ElementState::Pressed => {
                    if self.pressed_keys.insert(code) {
                        self.just_pressed_keys.insert(code);
                    }
                }
                ElementState::Released => {
                    self.pressed_keys.remove(&code);
                }
            }
        }
    }

    pub fn handle_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        match state {
            ElementState::Pressed => {
                self.pressed_mouse.insert(button);
            }
            ElementState::Released => {
                self.pressed_mouse.remove(&button);
            }
        }
    }

    pub fn add_mouse_delta(&mut self, dx: f32, dy: f32) {
        self.mouse_delta += Vec2::new(dx, dy);
    }

    pub fn is_down(&self, code: KeyCode) -> bool {
        self.pressed_keys.contains(&code)
    }

    pub fn was_just_pressed(&self, code: KeyCode) -> bool {
        self.just_pressed_keys.contains(&code)
    }

    /// Poll gamepad events and produce the unified frame action snapshot.
    pub fn sample_actions(&mut self, mouse_sensitivity: f32, invert_y: bool) -> ActionFrameState {
        if let Some(gilrs) = &mut self.gilrs {
            while let Some(ev) = gilrs.next_event() {
                if let EventType::AxisChanged(axis, val, _) = ev.event {
                    let filtered = if val.abs() < self.gamepad_deadzone {
                        0.0
                    } else {
                        val
                    };
                    match axis {
                        Axis::LeftStickX => self.gamepad_move.x = filtered,
                        Axis::LeftStickY => self.gamepad_move.y = filtered,
                        Axis::RightStickX => self.gamepad_look.x = filtered * 8.0,
                        Axis::RightStickY => self.gamepad_look.y = filtered * 8.0,
                        _ => {}
                    }
                } else if let EventType::ButtonPressed(btn, _) = ev.event {
                    match btn {
                        Button::North => {
                            self.just_pressed_keys.insert(KeyCode::KeyF);
                        }
                        Button::Start => {
                            self.just_pressed_keys.insert(KeyCode::Escape);
                        }
                        _ => {}
                    }
                }
            }
        }

        let mut move_x = 0.0f32;
        let mut move_y = 0.0f32;
        if self.is_down(KeyCode::KeyA) || self.is_down(KeyCode::ArrowLeft) {
            move_x -= 1.0;
        }
        if self.is_down(KeyCode::KeyD) || self.is_down(KeyCode::ArrowRight) {
            move_x += 1.0;
        }
        if self.is_down(KeyCode::KeyW) || self.is_down(KeyCode::ArrowUp) {
            move_y += 1.0;
        }
        if self.is_down(KeyCode::KeyS) || self.is_down(KeyCode::ArrowDown) {
            move_y -= 1.0;
        }

        let kb_move = Vec2::new(move_x, move_y);
        let combined_move = if kb_move.length_squared() > 0.01 {
            kb_move.normalize_or_zero()
        } else {
            self.gamepad_move
        };

        let mut look = self.mouse_delta * mouse_sensitivity * 0.2 + self.gamepad_look;
        if invert_y {
            look.y = -look.y;
        }

        let actions = ActionFrameState {
            move_axis: combined_move,
            look_delta: look,
            sprint: self.is_down(KeyCode::ShiftLeft) || self.is_down(KeyCode::ShiftRight),
            jump_or_handbrake: self.is_down(KeyCode::Space),
            crouch_or_horn: self.is_down(KeyCode::KeyC) || self.is_down(KeyCode::KeyH),
            fire_or_attack: self.pressed_mouse.contains(&MouseButton::Left)
                || self.is_down(KeyCode::ControlLeft),
            aim_or_reverse: self.pressed_mouse.contains(&MouseButton::Right),
            enter_exit_vehicle: self.is_down(KeyCode::KeyF) || self.is_down(KeyCode::Enter),
            enter_exit_just_pressed: self.was_just_pressed(KeyCode::KeyF)
                || self.was_just_pressed(KeyCode::Enter),
            skip_cutscene_just_pressed: self.was_just_pressed(KeyCode::Space)
                || self.was_just_pressed(KeyCode::Enter),
            next_weapon: self.was_just_pressed(KeyCode::KeyE),
            prev_weapon: self.was_just_pressed(KeyCode::KeyQ),
            radio_next: self.was_just_pressed(KeyCode::KeyR),
            toggle_pause: self.was_just_pressed(KeyCode::Escape),
            toggle_dev_overlay: self.was_just_pressed(KeyCode::F1),
            toggle_map: self.was_just_pressed(KeyCode::KeyM),
        };

        self.mouse_delta = Vec2::ZERO;
        self.just_pressed_keys.clear();
        actions
    }
}
