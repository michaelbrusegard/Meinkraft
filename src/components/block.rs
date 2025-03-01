pub struct Block {
    pub block_type: BlockType,
}

pub enum BlockType {
    Dirt,
    Stone,
    Grass,
}

impl Block {
    pub fn new(block_type: BlockType) -> Self {
        Self { block_type }
    }
}
