use std::collections::HashSet;
use winit::keyboard::Key;

pub struct InputState {
    pub pressed_keys: HashSet<Key>,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
        }
    }

    pub fn is_key_pressed(&self, key: &Key) -> bool {
        self.pressed_keys.contains(key)
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}
