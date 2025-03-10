use crate::resources::{Config, GameAction, InputState};
use crate::window_manager::WindowManager;
use glam::Vec3;
use hecs::World;
use winit::event::{DeviceEvent, ElementState, WindowEvent};
use winit::keyboard::{Key, NamedKey};

pub struct InputSystem {
    config: Config,
    yaw: f32,
    pitch: f32,
    cursor_grabbed: bool,
}

impl InputSystem {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            yaw: -90.0_f32.to_radians(),
            pitch: 0.0,
            cursor_grabbed: true,
        }
    }

    pub fn update(
        &mut self,
        _world: &mut World,
        input_state: &InputState,
        camera: &mut crate::resources::Camera,
    ) {
        let (mouse_dx, mouse_dy) = input_state.mouse_delta;
        self.handle_mouse_look(camera, mouse_dx, mouse_dy);

        self.handle_movement(input_state, camera);
    }

    pub fn handle_window_event(
        &mut self,
        event: &WindowEvent,
        input_state: &mut InputState,
        window_manager: &mut WindowManager,
    ) -> bool {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                match event.state {
                    ElementState::Pressed => {
                        input_state.pressed_keys.insert(event.logical_key.clone());

                        if let Key::Named(NamedKey::Escape) = event.logical_key {
                            self.release_cursor(window_manager);
                        }
                    }
                    ElementState::Released => {
                        input_state.remove_key(&event.logical_key);
                    }
                }
                true
            }
            WindowEvent::MouseInput { state, button, .. } => {
                match state {
                    ElementState::Pressed => {
                        input_state.pressed_mouse_buttons.insert(*button);

                        if !self.cursor_grabbed {
                            self.grab_cursor(window_manager);
                        }
                    }
                    ElementState::Released => {
                        input_state.pressed_mouse_buttons.remove(button);
                    }
                }
                true
            }
            _ => false,
        }
    }

    pub fn handle_device_event(&mut self, event: &DeviceEvent, input_state: &mut InputState) {
        if !self.cursor_grabbed {
            return;
        }

        if let DeviceEvent::MouseMotion { delta } = event {
            input_state.mouse_delta = (delta.0 as f32, delta.1 as f32);
        }
    }

    pub fn is_cursor_grabbed(&self) -> bool {
        self.cursor_grabbed
    }

    pub fn grab_cursor(&mut self, window_manager: &mut WindowManager) {
        if !self.cursor_grabbed {
            self.cursor_grabbed = true;
            window_manager.set_cursor_grabbed(true);
        }
    }

    pub fn release_cursor(&mut self, window_manager: &mut WindowManager) {
        if self.cursor_grabbed {
            self.cursor_grabbed = false;
            window_manager.set_cursor_grabbed(false);
        }
    }

    fn handle_mouse_look(&mut self, camera: &mut crate::resources::Camera, dx: f32, dy: f32) {
        self.yaw += dx * self.config.mouse_sensitivity;
        self.pitch -= dy * self.config.mouse_sensitivity;

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

    fn handle_movement(&self, input_state: &InputState, camera: &mut crate::resources::Camera) {
        let mut movement = Vec3::ZERO;

        let forward = (camera.target - camera.position).normalize();
        let forward_horizontal = Vec3::new(forward.x, 0.0, forward.z).normalize();
        let right = forward.cross(camera.up).normalize();

        if input_state.is_key_pressed(self.config.get_key(&GameAction::MoveForward).unwrap()) {
            movement += forward_horizontal;
        }
        if input_state.is_key_pressed(self.config.get_key(&GameAction::MoveBackward).unwrap()) {
            movement -= forward_horizontal;
        }
        if input_state.is_key_pressed(self.config.get_key(&GameAction::MoveLeft).unwrap()) {
            movement -= right;
        }
        if input_state.is_key_pressed(self.config.get_key(&GameAction::MoveRight).unwrap()) {
            movement += right;
        }
        if input_state.is_key_pressed(self.config.get_key(&GameAction::MoveUp).unwrap()) {
            movement += Vec3::new(0.0, 1.0, 0.0);
        }
        if input_state.is_key_pressed(self.config.get_key(&GameAction::MoveDown).unwrap()) {
            movement -= Vec3::new(0.0, 1.0, 0.0);
        }

        if movement != Vec3::ZERO {
            movement = movement.normalize() * self.config.move_speed;

            camera.position += movement;
            camera.target += movement;
        }
    }
}

impl Default for InputSystem {
    fn default() -> Self {
        Self::new(Config::default())
    }
}
