use crate::components::{BlockType, ChunkCoord, ChunkData, CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_WIDTH};
use crate::resources::Config;
use noise::{Fbm, MultiFractal, NoiseFn, Simplex};

const SEA_LEVEL: i32 = 62;
const SNOW_LEVEL: i32 = 95;
const DIRT_DEPTH: i32 = 3;

const BASE_FREQ: f64 = 1.0 / 512.0;
const MOUNTAIN_FREQ: f64 = 1.0 / 384.0;
const ROUGHNESS_FREQ: f64 = 1.0 / 96.0;

const BASE_AMP: f64 = 30.0;
const MOUNTAIN_AMP: f64 = 70.0;
const ROUGHNESS_AMP: f64 = 5.0;

pub struct WorldGenerator {
    base_height_noise: Fbm<Simplex>,
    mountain_noise: Fbm<Simplex>,
    roughness_noise: Fbm<Simplex>,
}

impl WorldGenerator {
    pub fn new(config: &Config) -> Self {
        let seed = config.seed;
        let base_height_noise = Fbm::<Simplex>::new(seed)
            .set_frequency(BASE_FREQ)
            .set_octaves(4)
            .set_lacunarity(2.0)
            .set_persistence(0.5);

        let mountain_noise = Fbm::<Simplex>::new(seed.wrapping_add(1))
            .set_frequency(MOUNTAIN_FREQ)
            .set_octaves(6)
            .set_lacunarity(2.0)
            .set_persistence(0.5);

        let roughness_noise = Fbm::<Simplex>::new(seed.wrapping_add(2))
            .set_frequency(ROUGHNESS_FREQ)
            .set_octaves(3)
            .set_lacunarity(2.0)
            .set_persistence(0.5);

        Self {
            base_height_noise,
            mountain_noise,
            roughness_noise,
        }
    }

    pub fn generate_chunk_data(&self, chunk_coord: ChunkCoord) -> ChunkData {
        let mut chunk_data = ChunkData::new();
        let ChunkCoord(cx, cy, cz) = chunk_coord;

        let chunk_origin_x = cx * CHUNK_WIDTH as i32;
        let chunk_origin_z = cz * CHUNK_DEPTH as i32;

        for local_x in 0..CHUNK_WIDTH {
            for local_z in 0..CHUNK_DEPTH {
                let world_x = chunk_origin_x + local_x as i32;
                let world_z = chunk_origin_z + local_z as i32;

                let terrain_height = self.calculate_terrain_height(world_x, world_z);

                for local_y in 0..CHUNK_HEIGHT {
                    let world_y = cy * CHUNK_HEIGHT as i32 + local_y as i32;

                    let block_type = if world_y > terrain_height {
                        if world_y <= SEA_LEVEL {
                            BlockType::Water
                        } else {
                            BlockType::Air
                        }
                    } else if world_y == terrain_height {
                        if world_y <= SEA_LEVEL {
                            BlockType::Sand
                        } else if world_y >= SNOW_LEVEL {
                            BlockType::Snow
                        } else {
                            BlockType::Grass
                        }
                    } else if world_y > terrain_height - DIRT_DEPTH {
                        BlockType::Dirt
                    } else {
                        BlockType::Stone
                    };

                    chunk_data.set_block(local_x, local_y, local_z, block_type);
                }
            }
        }

        chunk_data
    }

    fn calculate_terrain_height(&self, world_x: i32, world_z: i32) -> i32 {
        let coords = [world_x as f64, world_z as f64];

        let base_noise_val = self.base_height_noise.get(coords);
        let base_h = (SEA_LEVEL - DIRT_DEPTH) as f64 + base_noise_val * BASE_AMP;

        let m_factor = ((base_h - (SEA_LEVEL - 10) as f64) / BASE_AMP).clamp(0.0, 1.0);
        let mountain_noise_val = self.mountain_noise.get(coords).abs();
        let mountain_h = mountain_noise_val * MOUNTAIN_AMP * m_factor;

        let roughness_noise_val = self.roughness_noise.get(coords);
        let roughness_h = roughness_noise_val * ROUGHNESS_AMP;

        let final_height = base_h + mountain_h + roughness_h;

        final_height.round() as i32
    }
}
