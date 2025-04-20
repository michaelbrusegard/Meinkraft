use crate::components::{BlockType, ChunkCoord, ChunkData, CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_WIDTH};
use crate::resources::Config;
use noise::{Fbm, MultiFractal, NoiseFn, Perlin, Simplex};

const SEA_LEVEL: i32 = 15;
const SNOW_LEVEL: i32 = 127;
const DIRT_DEPTH: i32 = 3;

const BASE_FREQ: f64 = 1.0 / 700.0;
const MOUNTAIN_FREQ: f64 = 1.0 / 350.0;
const ROUGHNESS_FREQ: f64 = 1.0 / 60.0;
const STONE_VARIATION_FREQ: f64 = 1.0 / 48.0;
const SEABED_GRAVEL_FREQ: f64 = 1.0 / 32.0;
const ICE_PATCH_FREQ: f64 = 1.0 / 20.0;

const BASE_AMP: f64 = 25.0;
const MOUNTAIN_AMP: f64 = 800.0;
const ROUGHNESS_AMP: f64 = 25.0;

const EXPOSED_STONE_THRESHOLD: f64 = 0.6;
const SEABED_GRAVEL_THRESHOLD: f64 = 0.2;
const ICE_PATCH_THRESHOLD: f64 = 0.4;

pub struct WorldGenerator {
    base_height_noise: Fbm<Simplex>,
    mountain_noise: Fbm<Simplex>,
    roughness_noise: Fbm<Simplex>,
    stone_variation_noise: Fbm<Simplex>,
    seabed_gravel_noise: Perlin,
    ice_patch_noise: Perlin,
}

impl WorldGenerator {
    pub fn new(config: &Config) -> Self {
        let seed = config.world_seed;
        let base_height_noise = Fbm::<Simplex>::new(seed)
            .set_frequency(BASE_FREQ)
            .set_octaves(4)
            .set_lacunarity(2.0)
            .set_persistence(0.5);

        let mountain_noise = Fbm::<Simplex>::new(seed.wrapping_add(1))
            .set_frequency(MOUNTAIN_FREQ)
            .set_octaves(7)
            .set_lacunarity(2.1)
            .set_persistence(0.5);

        let roughness_noise = Fbm::<Simplex>::new(seed.wrapping_add(2))
            .set_frequency(ROUGHNESS_FREQ)
            .set_octaves(4)
            .set_lacunarity(2.0)
            .set_persistence(0.5);

        let stone_variation_noise = Fbm::<Simplex>::new(seed.wrapping_add(3))
            .set_frequency(STONE_VARIATION_FREQ)
            .set_octaves(3)
            .set_lacunarity(2.0)
            .set_persistence(0.5);

        let seabed_gravel_noise = Perlin::new(seed.wrapping_add(4));

        let ice_patch_noise = Perlin::new(seed.wrapping_add(5));

        Self {
            base_height_noise,
            mountain_noise,
            roughness_noise,
            stone_variation_noise,
            seabed_gravel_noise,
            ice_patch_noise,
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
                let roughness_val = self.get_roughness_value(world_x, world_z);

                let height_nx = self.calculate_terrain_height(world_x + 1, world_z);
                let height_nz = self.calculate_terrain_height(world_x, world_z + 1);
                let diff_x = (terrain_height - height_nx).abs();
                let diff_z = (terrain_height - height_nz).abs();
                let max_height_diff = diff_x.max(diff_z);

                let stone_coords = [
                    world_x as f64,
                    world_z as f64,
                    (cy * CHUNK_HEIGHT as i32) as f64 * 0.1,
                ];
                let seabed_coords = [world_x as f64, world_z as f64];

                for local_y in 0..CHUNK_HEIGHT {
                    let world_y = cy * CHUNK_HEIGHT as i32 + local_y as i32;

                    let block_type = if world_y > terrain_height {
                        if world_y == terrain_height + 1 && terrain_height >= SNOW_LEVEL {
                            let ice_coords = [
                                world_x as f64 * ICE_PATCH_FREQ,
                                world_z as f64 * ICE_PATCH_FREQ,
                            ];
                            let ice_noise_val = self.ice_patch_noise.get(ice_coords);
                            if ice_noise_val > ICE_PATCH_THRESHOLD {
                                BlockType::Ice
                            } else {
                                BlockType::Air
                            }
                        } else if world_y <= SEA_LEVEL {
                            BlockType::Water
                        } else {
                            BlockType::Air
                        }
                    } else {
                        let is_surface = world_y == terrain_height;
                        let is_dirt_layer =
                            world_y > terrain_height - DIRT_DEPTH && world_y < terrain_height;

                        let is_rough =
                            roughness_val * ROUGHNESS_AMP > ROUGHNESS_AMP * EXPOSED_STONE_THRESHOLD;
                        let is_steep = max_height_diff > DIRT_DEPTH;
                        let should_expose_stone = (is_rough || is_steep) && world_y > SEA_LEVEL + 1;

                        if is_surface {
                            if world_y >= SNOW_LEVEL {
                                BlockType::Snow
                            } else if should_expose_stone {
                                BlockType::Stone
                            } else if world_y > SEA_LEVEL {
                                BlockType::GrassyDirt
                            } else {
                                let scaled_seabed_coords = [
                                    seabed_coords[0] * SEABED_GRAVEL_FREQ,
                                    seabed_coords[1] * SEABED_GRAVEL_FREQ,
                                ];
                                let gravel_noise =
                                    self.seabed_gravel_noise.get(scaled_seabed_coords);
                                if gravel_noise > SEABED_GRAVEL_THRESHOLD {
                                    BlockType::Gravel
                                } else {
                                    BlockType::Sand
                                }
                            }
                        } else if is_dirt_layer {
                            if world_y == terrain_height - 1 && terrain_height >= SNOW_LEVEL {
                                BlockType::SnowyDirt
                            } else if should_expose_stone {
                                BlockType::Stone
                            } else {
                                BlockType::Dirt
                            }
                        } else {
                            let scaled_stone_coords = [
                                stone_coords[0] * STONE_VARIATION_FREQ,
                                stone_coords[1] * STONE_VARIATION_FREQ,
                                stone_coords[2] * STONE_VARIATION_FREQ,
                            ];
                            let stone_noise_val =
                                self.stone_variation_noise.get(scaled_stone_coords);
                            if stone_noise_val > 0.3 {
                                BlockType::Andesite
                            } else if stone_noise_val > -0.1 {
                                BlockType::Stone
                            } else if stone_noise_val > -0.5 {
                                BlockType::Granite
                            } else {
                                BlockType::Diorite
                            }
                        }
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
        let base_h = SEA_LEVEL as f64 + base_noise_val * BASE_AMP;

        let m_factor_base = (base_h - SEA_LEVEL as f64) / (BASE_AMP * 0.6);
        let m_factor = m_factor_base.clamp(0.0, 1.0).powi(2);

        let mountain_noise_val = self.mountain_noise.get(coords).abs();
        let mountain_h = mountain_noise_val * MOUNTAIN_AMP * m_factor;

        let roughness_noise_val = self.roughness_noise.get(coords);
        let roughness_h = roughness_noise_val * ROUGHNESS_AMP;

        let final_height = base_h + mountain_h + roughness_h;

        final_height.round().clamp(1.0, 255.0) as i32
    }

    fn get_roughness_value(&self, world_x: i32, world_z: i32) -> f64 {
        self.roughness_noise.get([world_x as f64, world_z as f64])
    }
}
