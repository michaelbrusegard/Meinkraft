use crate::components::BlockType;

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_HEIGHT: usize = 16;
pub const CHUNK_DEPTH: usize = 16;
pub const CHUNK_SIZE: usize = CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_DEPTH;
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct ChunkCoord(pub i32, pub i32, pub i32);

pub struct ChunkData {
    pub blocks: Box<[BlockType; CHUNK_SIZE]>,
}

impl ChunkData {
    pub fn new() -> Self {
        Self {
            blocks: Box::new([BlockType::Air; CHUNK_SIZE]),
        }
    }

    #[inline]
    fn local_coords_to_index(x: usize, y: usize, z: usize) -> Option<usize> {
        if x < CHUNK_WIDTH && y < CHUNK_HEIGHT && z < CHUNK_DEPTH {
            Some(x + y * CHUNK_WIDTH + z * CHUNK_WIDTH * CHUNK_HEIGHT)
        } else {
            None
        }
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> BlockType {
        Self::local_coords_to_index(x, y, z)
            .map(|index| self.blocks[index])
            .unwrap_or(BlockType::Air)
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block_type: BlockType) {
        if let Some(index) = Self::local_coords_to_index(x, y, z) {
            self.blocks[index] = block_type;
        }
    }
}

impl Default for ChunkData {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ChunkDirty;

#[inline]
pub fn world_to_chunk_coords(world_x: i32, world_y: i32, world_z: i32) -> ChunkCoord {
    ChunkCoord(
        world_x.div_euclid(CHUNK_WIDTH as i32),
        world_y.div_euclid(CHUNK_HEIGHT as i32),
        world_z.div_euclid(CHUNK_DEPTH as i32),
    )
}

#[inline]
pub fn world_to_local_coords(world_x: i32, world_y: i32, world_z: i32) -> (usize, usize, usize) {
    (
        world_x.rem_euclid(CHUNK_WIDTH as i32) as usize,
        world_y.rem_euclid(CHUNK_HEIGHT as i32) as usize,
        world_z.rem_euclid(CHUNK_DEPTH as i32) as usize,
    )
}

#[inline]
pub fn chunk_coord_to_world_pos(coord: ChunkCoord) -> glam::Vec3 {
    glam::Vec3::new(
        (coord.0 * CHUNK_WIDTH as i32) as f32,
        (coord.1 * CHUNK_HEIGHT as i32) as f32,
        (coord.2 * CHUNK_DEPTH as i32) as f32,
    )
}
