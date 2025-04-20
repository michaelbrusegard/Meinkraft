use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BlockType {
    Air,
    Dirt,
    GrassyDirt,
    Stone,
    Snow,
    SnowyDirt,
    Sand,
    Glass,
    Log,
    Planks,
    Leaves,
    Water,
    Ice,
    Gravel,
    Andesite,
    Granite,
    Diorite,
    Cobblestone,
}

impl BlockType {
    pub fn is_solid(&self) -> bool {
        !matches!(self, BlockType::Air)
    }

    pub fn get_face_textures(&self) -> Option<[&'static str; 6]> {
        match self {
            BlockType::Air => None,
            BlockType::Dirt => Some(["dirt"; 6]),
            BlockType::GrassyDirt => Some([
                "grassy_dirt_top",
                "dirt",
                "grassy_dirt_side",
                "grassy_dirt_side",
                "grassy_dirt_side",
                "grassy_dirt_side",
            ]),
            BlockType::Snow => Some(["snow"; 6]),
            BlockType::SnowyDirt => Some([
                "snow",
                "dirt",
                "snowy_dirt_side",
                "snowy_dirt_side",
                "snowy_dirt_side",
                "snowy_dirt_side",
            ]),
            BlockType::Stone => Some(["stone"; 6]),
            BlockType::Sand => Some(["sand"; 6]),
            BlockType::Glass => Some(["glass"; 6]),
            BlockType::Log => Some(["log_top", "log_top", "log", "log", "log", "log"]),
            BlockType::Planks => Some(["planks"; 6]),
            BlockType::Leaves => Some(["leaves"; 6]),
            BlockType::Water => Some(["water"; 6]),
            BlockType::Ice => Some(["ice"; 6]),
            BlockType::Gravel => Some(["gravel"; 6]),
            BlockType::Andesite => Some(["andesite"; 6]),
            BlockType::Granite => Some(["granite"; 6]),
            BlockType::Diorite => Some(["diorite"; 6]),
            BlockType::Cobblestone => Some(["cobblestone"; 6]),
        }
    }
}
