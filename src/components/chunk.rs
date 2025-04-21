use crate::components::BlockType;
use crate::resources::Config;
use glam::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChunkCoord(pub i32, pub i32, pub i32);

#[derive(Clone, Serialize, Deserialize)]
pub struct ChunkData {
    pub blocks: Vec<BlockType>,
}

impl ChunkData {
    pub fn new(config: &Config) -> Self {
        Self {
            blocks: vec![BlockType::Air; config.chunk_size],
        }
    }

    #[inline]
    fn local_coords_to_index(config: &Config, x: usize, y: usize, z: usize) -> Option<usize> {
        if x < config.chunk_width && y < config.chunk_height && z < config.chunk_depth {
            Some(y * (config.chunk_width * config.chunk_depth) + z * config.chunk_width + x)
        } else {
            None
        }
    }

    pub fn get_block(&self, config: &Config, x: usize, y: usize, z: usize) -> BlockType {
        Self::local_coords_to_index(config, x, y, z)
            .and_then(|index| self.blocks.get(index).copied())
            .unwrap_or(BlockType::Air)
    }

    pub fn set_block(
        &mut self,
        config: &Config,
        x: usize,
        y: usize,
        z: usize,
        block_type: BlockType,
    ) {
        if let Some(index) = Self::local_coords_to_index(config, x, y, z) {
            if index < self.blocks.len() {
                self.blocks[index] = block_type;
            }
        }
    }
}

pub struct ChunkDirty;
pub struct ChunkModified;

#[inline]
pub fn world_to_chunk_coords(
    config: &Config,
    world_x: i32,
    world_y: i32,
    world_z: i32,
) -> ChunkCoord {
    ChunkCoord(
        world_x.div_euclid(config.chunk_width as i32),
        world_y.div_euclid(config.chunk_height as i32),
        world_z.div_euclid(config.chunk_depth as i32),
    )
}

#[inline]
pub fn world_to_local_coords(
    config: &Config,
    world_x: i32,
    world_y: i32,
    world_z: i32,
) -> (usize, usize, usize) {
    (
        world_x.rem_euclid(config.chunk_width as i32) as usize,
        world_y.rem_euclid(config.chunk_height as i32) as usize,
        world_z.rem_euclid(config.chunk_depth as i32) as usize,
    )
}

#[inline]
pub fn chunk_coord_to_world_pos(config: &Config, coord: ChunkCoord) -> Vec3 {
    Vec3::new(
        (coord.0 * config.chunk_width as i32) as f32,
        (coord.1 * config.chunk_height as i32) as f32,
        (coord.2 * config.chunk_depth as i32) as f32,
    )
}

#[inline]
pub fn chunk_coord_to_aabb_center(config: &Config, coord: ChunkCoord) -> Vec3 {
    let half_width = config.chunk_width as f32 * 0.5;
    let half_height = config.chunk_height as f32 * 0.5;
    let half_depth = config.chunk_depth as f32 * 0.5;
    let chunk_extents = Vec3::new(half_width, half_height, half_depth);
    chunk_coord_to_world_pos(config, coord) + chunk_extents
}

#[inline]
pub fn get_chunk_extents(config: &Config) -> Vec3 {
    Vec3::new(
        config.chunk_width as f32 * 0.5,
        config.chunk_height as f32 * 0.5,
        config.chunk_depth as f32 * 0.5,
    )
}
