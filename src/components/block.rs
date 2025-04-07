use enum_iterator::Sequence;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sequence)]
pub enum BlockType {
    Dirt,
    Stone,
    Grass,
    Sand,
    Glass,
    Log,
    Planks,
}

pub struct Block {
    pub block_type: BlockType,
}

impl Block {
    pub fn new(block_type: BlockType) -> Self {
        Self { block_type }
    }
}

impl BlockType {
    pub fn get_face_textures(&self) -> [&'static str; 6] {
        match self {
            BlockType::Dirt => ["dirt", "dirt", "dirt", "dirt", "dirt", "dirt"],
            BlockType::Stone => ["stone", "stone", "stone", "stone", "stone", "stone"],
            BlockType::Grass => [
                "grass_side",
                "grass_side",
                "grass_side",
                "grass_side",
                "grass_top",
                "dirt",
            ],
            BlockType::Sand => ["sand", "sand", "sand", "sand", "sand", "sand"],
            BlockType::Glass => ["glass", "glass", "glass", "glass", "glass", "glass"],
            BlockType::Log => ["log", "log", "log", "log", "log_top", "log_top"],
            BlockType::Planks => ["planks", "planks", "planks", "planks", "planks", "planks"],
        }
    }
}
