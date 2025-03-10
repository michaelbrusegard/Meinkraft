use std::collections::HashSet;
use winit::event::MouseButton;
use winit::keyboard::Key;

pub struct InputState {
    pub pressed_keys: HashSet<Key>,
    pub pressed_mouse_buttons: HashSet<MouseButton>,
    pub mouse_delta: (f32, f32),
}

impl InputState {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            pressed_mouse_buttons: HashSet::new(),
            mouse_delta: (0.0, 0.0),
        }
    }

    pub fn is_key_pressed(&self, key: &Key) -> bool {
        if self.pressed_keys.contains(key) {
            return true;
        }

        if let Key::Character(c) = key {
            let lowercase = c.to_lowercase();
            for pressed in &self.pressed_keys {
                if let Key::Character(pressed_c) = pressed {
                    if pressed_c.to_lowercase() == lowercase {
                        return true;
                    }
                }
            }
        }

        false
    }

    pub fn remove_key(&mut self, key: &Key) {
        if let Key::Character(c) = key {
            let lowercase = c.to_lowercase();
            self.pressed_keys.retain(|pressed| {
                if let Key::Character(pressed_c) = pressed {
                    pressed_c.to_lowercase() != lowercase
                } else {
                    true
                }
            });
        } else {
            self.pressed_keys.remove(key);
        }
    }

    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.pressed_mouse_buttons.contains(&button)
    }

    pub fn reset_frame_state(&mut self) {
        self.mouse_delta = (0.0, 0.0);
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}
