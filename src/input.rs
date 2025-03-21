use crate::resources::InputState;
use crate::window::WindowManager;
use winit::event::{DeviceEvent, ElementState, WindowEvent};
use winit::keyboard::{Key, NamedKey};

pub struct InputManager {
    cursor_grabbed: bool,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            cursor_grabbed: true,
        }
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

    fn grab_cursor(&mut self, window_manager: &mut WindowManager) {
        if !self.cursor_grabbed {
            self.cursor_grabbed = true;
            window_manager.set_cursor_grabbed(true);
        }
    }

    fn release_cursor(&mut self, window_manager: &mut WindowManager) {
        if self.cursor_grabbed {
            self.cursor_grabbed = false;
            window_manager.set_cursor_grabbed(false);
        }
    }
}
