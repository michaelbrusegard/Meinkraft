#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockType {
    Air,
    Dirt,
    Stone,
    Grass,
    Sand,
    Glass,
    Log,
    Planks,
}

impl BlockType {
    pub fn is_solid(&self) -> bool {
        !matches!(self, BlockType::Air)
    }

    pub fn get_face_textures(&self) -> Option<[&'static str; 6]> {
        match self {
            BlockType::Air => None,
            BlockType::Dirt => Some(["dirt", "dirt", "dirt", "dirt", "dirt", "dirt"]),
            BlockType::Stone => Some(["stone", "stone", "stone", "stone", "stone", "stone"]),
            BlockType::Grass => Some([
                "grass_side",
                "grass_side",
                "grass_side",
                "grass_side",
                "grass_top",
                "dirt",
            ]),
            BlockType::Sand => Some(["sand", "sand", "sand", "sand", "sand", "sand"]),
            BlockType::Glass => Some(["glass", "glass", "glass", "glass", "glass", "glass"]),
            BlockType::Log => Some(["log", "log", "log", "log", "log_top", "log_top"]),
            BlockType::Planks => Some(["planks", "planks", "planks", "planks", "planks", "planks"]),
        }
    }
}
