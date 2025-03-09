use std::collections::HashSet;
use winit::keyboard::Key;

pub struct InputState {
    pub pressed_keys: HashSet<Key>,
    pub pressed_mouse_buttons: HashSet<u32>,
    pub mouse_position: (f32, f32),
    pub mouse_delta: (f32, f32),
}

impl InputState {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            pressed_mouse_buttons: HashSet::new(),
            mouse_position: (0.0, 0.0),
            mouse_delta: (0.0, 0.0),
        }
    }

    pub fn is_key_pressed(&self, physical_key: &Key) -> bool {
        self.pressed_keys.contains(physical_key)
    }

    pub fn is_mouse_button_pressed(&self, button: u32) -> bool {
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
