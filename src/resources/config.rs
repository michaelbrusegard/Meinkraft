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

#[derive(Clone)]
pub struct Config {
    key_bindings: HashMap<GameAction, Key>,
    pub move_speed: f32,
    pub mouse_sensitivity: f32,
    pub load_distance: i32,
    pub lod2_distance: i32,
    pub lod4_distance: i32,
    pub lod8_distance: i32,
    pub render_distance: i32,
    pub world_seed: u32,
    pub day_cycle_speed: f32,
    pub chunk_width: usize,
    pub chunk_height: usize,
    pub chunk_depth: usize,
    pub chunk_size: usize,
    pub min_chunk_y: i32,
    pub max_chunk_y: i32,
    pub min_light_level: f32,
    pub max_light_level: f32,
    pub sunrise_center_time: f32,
    pub sunset_center_time: f32,
    pub day_night_transition_duration: f32,
    pub midnight_color: glam::Vec3,
    pub noon_color: glam::Vec3,
    pub sunrise_peak_color: glam::Vec3,
    pub sunset_peak_color: glam::Vec3,
    pub min_ambient_intensity: f32,
    pub max_ambient_intensity: f32,
    pub min_absolute_ambient: f32,
    pub material_shininess: f32,
    pub sea_level: i32,
    pub snow_level: i32,
    pub dirt_depth: i32,
    pub base_freq: f64,
    pub mountain_freq: f64,
    pub roughness_freq: f64,
    pub stone_variation_freq: f64,
    pub seabed_gravel_freq: f64,
    pub ice_patch_freq: f64,
    pub base_amp: f64,
    pub mountain_amp: f64,
    pub roughness_amp: f64,
    pub exposed_stone_threshold: f64,
    pub seabed_gravel_threshold: f64,
    pub ice_patch_threshold: f64,
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
            load_distance: 8,    // 12
            lod2_distance: 12,   // 24
            lod8_distance: 14,   // 28
            lod4_distance: 15,   // 30
            render_distance: 16, // 32
            world_seed: 42069,
            day_cycle_speed: 0.01,
            chunk_width: 16,
            chunk_height: 16,
            chunk_depth: 16,
            chunk_size: 16 * 16 * 16,
            min_chunk_y: 0,
            max_chunk_y: 15,
            min_light_level: 0.15,
            max_light_level: 1.0,
            sunrise_center_time: 0.22,
            sunset_center_time: 0.72,
            day_night_transition_duration: 0.05,
            midnight_color: glam::Vec3::new(0.01, 0.01, 0.05),
            noon_color: glam::Vec3::new(0.5, 0.8, 1.0),
            sunrise_peak_color: glam::Vec3::new(0.9, 0.6, 0.3),
            sunset_peak_color: glam::Vec3::new(0.9, 0.5, 0.3),
            min_ambient_intensity: 0.25,
            max_ambient_intensity: 1.0,
            min_absolute_ambient: 0.15,
            material_shininess: 32.0,
            sea_level: 15,
            snow_level: 127,
            dirt_depth: 3,
            base_freq: 1.0 / 700.0,
            mountain_freq: 1.0 / 350.0,
            roughness_freq: 1.0 / 60.0,
            stone_variation_freq: 1.0 / 48.0,
            seabed_gravel_freq: 1.0 / 32.0,
            ice_patch_freq: 1.0 / 20.0,
            base_amp: 25.0,
            mountain_amp: 800.0,
            roughness_amp: 25.0,
            exposed_stone_threshold: 0.6,
            seabed_gravel_threshold: 0.2,
            ice_patch_threshold: 0.4,
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
