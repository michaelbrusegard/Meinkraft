use std::collections::HashMap;
use winit::keyboard::{Key, NamedKey};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameAction {
    MoveForward,
    MoveBackward,
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
}

pub struct Config {
    key_bindings: HashMap<GameAction, Key>,
    pub move_speed: f32,
    pub mouse_sensitivity: f32,
    pub load_distance: i32,
    pub render_distance: i32,
    pub world_seed: u32,
}

impl Config {
    pub fn new() -> Self {
        let mut key_bindings = HashMap::new();

        key_bindings.insert(GameAction::MoveForward, Key::Character("w".into()));
        key_bindings.insert(GameAction::MoveBackward, Key::Character("s".into()));
        key_bindings.insert(GameAction::MoveLeft, Key::Character("a".into()));
        key_bindings.insert(GameAction::MoveRight, Key::Character("d".into()));
        key_bindings.insert(GameAction::MoveUp, Key::Named(NamedKey::Space));
        key_bindings.insert(GameAction::MoveDown, Key::Named(NamedKey::Shift));

        Self {
            key_bindings,
            move_speed: 1.0,
            mouse_sensitivity: 0.003,
            load_distance: 8,
            render_distance: 16,
            world_seed: 42069,
        }
    }

    pub fn get_key(&self, action: &GameAction) -> Option<&Key> {
        self.key_bindings.get(action)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
