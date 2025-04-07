use crate::input::InputManager;
use crate::resources::{Config, GameAction, InputState};
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
        camera: &mut crate::resources::Camera,
        input_manager: &InputManager,
    ) {
        if !input_manager.is_cursor_grabbed() {
            return;
        }

        let (mouse_dx, mouse_dy) = input_state.mouse_delta;
        self.handle_mouse_look(config, camera, mouse_dx, mouse_dy);
        Self::handle_movement(config, input_state, camera);
    }

    fn handle_mouse_look(
        &mut self,
        config: &Config,
        camera: &mut crate::resources::Camera,
        dx: f32,
        dy: f32,
    ) {
        self.yaw += dx * config.mouse_sensitivity;
        self.pitch -= dy * config.mouse_sensitivity;

        self.pitch = self
            .pitch
            .clamp(-89.0_f32.to_radians(), 89.0_f32.to_radians());

        let forward = Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize();

        camera.target = camera.position + forward;
    }

    fn handle_movement(
        config: &Config,
        input_state: &InputState,
        camera: &mut crate::resources::Camera,
    ) {
        let mut movement = Vec3::ZERO;

        let forward = (camera.target - camera.position).normalize();
        let forward_horizontal = Vec3::new(forward.x, 0.0, forward.z).normalize();
        let right = forward.cross(camera.up).normalize();

        if input_state.is_key_pressed(config.get_key(&GameAction::MoveForward).unwrap()) {
            movement += forward_horizontal;
        }
        if input_state.is_key_pressed(config.get_key(&GameAction::MoveBackward).unwrap()) {
            movement -= forward_horizontal;
        }
        if input_state.is_key_pressed(config.get_key(&GameAction::MoveLeft).unwrap()) {
            movement -= right;
        }
        if input_state.is_key_pressed(config.get_key(&GameAction::MoveRight).unwrap()) {
            movement += right;
        }
        if input_state.is_key_pressed(config.get_key(&GameAction::MoveUp).unwrap()) {
            movement += Vec3::new(0.0, 1.0, 0.0);
        }
        if input_state.is_key_pressed(config.get_key(&GameAction::MoveDown).unwrap()) {
            movement -= Vec3::new(0.0, 1.0, 0.0);
        }

        if movement != Vec3::ZERO {
            movement = movement.normalize() * config.move_speed;

            camera.position += movement;
            camera.target += movement;
        }
    }
}

impl Default for InputSystem {
    fn default() -> Self {
        Self::new()
    }
}
