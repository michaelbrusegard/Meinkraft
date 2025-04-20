use crate::input::InputManager;
use crate::resources::{Camera, Config, GameAction, InputState};
use glam::Vec3;
use hecs::World;

pub struct InputSystem {
    yaw: f32,
    pitch: f32,
}

impl InputSystem {
    pub fn new() -> Self {
        Self {
            yaw: -90.0_f32.to_radians(),
            pitch: 0.0,
        }
    }

    pub fn update(
        &mut self,
        config: &Config,
        _world: &mut World,
        input_state: &InputState,
        camera: &mut Camera,
        input_manager: &InputManager,
    ) {
        if !input_manager.is_cursor_grabbed() {
            return;
        }

        let current_position = camera.position;

        let (mouse_dx, mouse_dy) = input_state.mouse_delta;
        let look_direction = self.handle_mouse_look(config, mouse_dx, mouse_dy);

        let new_position = Self::handle_movement(config, input_state, camera, current_position);

        let new_target = new_position + look_direction;

        camera.set_position_target(new_position, new_target);
    }

    fn handle_mouse_look(&mut self, config: &Config, dx: f32, dy: f32) -> Vec3 {
        self.yaw += dx * config.mouse_sensitivity;
        self.yaw = self.yaw.rem_euclid(std::f32::consts::TAU);

        self.pitch -= dy * config.mouse_sensitivity;

        self.pitch = self
            .pitch
            .clamp(-89.0_f32.to_radians(), 89.0_f32.to_radians());

        Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize()
    }

    fn handle_movement(
        config: &Config,
        input_state: &InputState,
        camera: &Camera,
        current_position: Vec3,
    ) -> Vec3 {
        let mut movement_input = Vec3::ZERO;

        let forward = (camera.target - camera.position).normalize();
        let forward_horizontal = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
        let right = forward.cross(camera.up).normalize();

        if let Some(key) = config.get_key(&GameAction::MoveForward) {
            if input_state.is_key_pressed(key) {
                movement_input += forward_horizontal;
            }
        }
        if let Some(key) = config.get_key(&GameAction::MoveBackward) {
            if input_state.is_key_pressed(key) {
                movement_input -= forward_horizontal;
            }
        }
        if let Some(key) = config.get_key(&GameAction::MoveLeft) {
            if input_state.is_key_pressed(key) {
                movement_input -= right;
            }
        }
        if let Some(key) = config.get_key(&GameAction::MoveRight) {
            if input_state.is_key_pressed(key) {
                movement_input += right;
            }
        }
        if let Some(key) = config.get_key(&GameAction::MoveUp) {
            if input_state.is_key_pressed(key) {
                movement_input += Vec3::Y;
            }
        }
        if let Some(key) = config.get_key(&GameAction::MoveDown) {
            if input_state.is_key_pressed(key) {
                movement_input -= Vec3::Y;
            }
        }

        if movement_input.length_squared() > f32::EPSILON {
            let movement_delta = movement_input.normalize() * config.move_speed;
            current_position + movement_delta
        } else {
            current_position
        }
    }
}

impl Default for InputSystem {
    fn default() -> Self {
        Self::new()
    }
}
