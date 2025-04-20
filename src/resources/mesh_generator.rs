use crate::components::{
    BlockType, ChunkCoord, ChunkData, CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_WIDTH, LOD,
};
use crate::resources::Mesh;
use std::collections::HashMap;

const DOWNSAMPLE_FACTOR: usize = 2;
const LOW_RES_WIDTH: usize = CHUNK_WIDTH / DOWNSAMPLE_FACTOR;
const LOW_RES_HEIGHT: usize = CHUNK_HEIGHT / DOWNSAMPLE_FACTOR;
const LOW_RES_DEPTH: usize = CHUNK_DEPTH / DOWNSAMPLE_FACTOR;

struct FaceParams {
    position: [f32; 3],
    face_index: usize,
    layer_index: f32,
    scale: f32,
}

pub struct MeshGenerator {}

impl MeshGenerator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn generate_chunk_mesh(
        &self,
        chunk_coord: ChunkCoord,
        chunk_data: &ChunkData,
        neighbors: &[Option<ChunkData>; 6],
        texture_layers: &HashMap<String, f32>,
        lod: LOD,
    ) -> Option<Mesh> {
        match lod {
            LOD::High => {
                self.generate_high_lod_mesh(chunk_coord, chunk_data, neighbors, texture_layers)
            }
            LOD::Low => {
                self.generate_low_lod_mesh(chunk_coord, chunk_data, neighbors, texture_layers)
            }
        }
    }

    fn generate_high_lod_mesh(
        &self,
        chunk_coord: ChunkCoord,
        chunk_data: &ChunkData,
        neighbors: &[Option<ChunkData>; 6],
        texture_layers: &HashMap<String, f32>,
    ) -> Option<Mesh> {
        let mut vertices: Vec<f32> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut index_offset: u32 = 0;

        for y in 0..CHUNK_HEIGHT {
            for z in 0..CHUNK_DEPTH {
                for x in 0..CHUNK_WIDTH {
                    let current_block_type = chunk_data.get_block(x, y, z);

                    if current_block_type == BlockType::Air || !current_block_type.is_solid() {
                        continue;
                    }

                    let face_textures = match current_block_type.get_face_textures() {
                        Some(textures) => textures,
                        None => continue,
                    };

                    let x_usize = x;
                    let y_usize = y;
                    let z_usize = z;

                    for face_index in 0..6 {
                        let (nx, ny, nz) =
                            Self::get_neighbor_coords(x_usize, y_usize, z_usize, face_index);

                        let mut should_draw_face = false;
                        if nx < 0
                            || nx >= CHUNK_WIDTH as i32
                            || ny < 0
                            || ny >= CHUNK_HEIGHT as i32
                            || nz < 0
                            || nz >= CHUNK_DEPTH as i32
                        {
                            should_draw_face = true;
                        } else {
                            let neighbor_block_type =
                                chunk_data.get_block(nx as usize, ny as usize, nz as usize);
                            if !neighbor_block_type.is_solid() {
                                should_draw_face = true;
                            }
                        }

                        if should_draw_face {
                            let texture_name = face_textures[Self::face_texture_index(face_index)];
                            let layer_index = match texture_layers.get(texture_name) {
                                Some(layer) => *layer,
                                None => {
                                    eprintln!(
                                        "Warning: Layer index not found for texture '{}' at chunk {:?}, block ({},{},{}), LOD: High",
                                        texture_name, chunk_coord, x, y, z
                                    );
                                    0.0
                                }
                            };
                            Self::add_scaled_face(
                                FaceParams {
                                    position: [x as f32, y as f32, z as f32],
                                    face_index,
                                    layer_index,
                                    scale: 1.0,
                                },
                                &mut vertices,
                                &mut indices,
                                &mut index_offset,
                            );
                        }
                    }
                }
            }
        }

        if vertices.is_empty() {
            None
        } else {
            Some(Mesh { vertices, indices })
        }
    }

    fn generate_low_lod_mesh(
        &self,
        chunk_coord: ChunkCoord,
        chunk_data: &ChunkData,
        _neighbors: &[Option<ChunkData>; 6],
        texture_layers: &HashMap<String, f32>,
    ) -> Option<Mesh> {
        let mut vertices: Vec<f32> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut index_offset: u32 = 0;

        let low_res_data = self.downsample_chunk(chunk_data);

        for ly in 0..LOW_RES_HEIGHT {
            for lz in 0..LOW_RES_DEPTH {
                for lx in 0..LOW_RES_WIDTH {
                    let low_res_index =
                        lx + lz * LOW_RES_WIDTH + ly * LOW_RES_WIDTH * LOW_RES_DEPTH;
                    let current_block_type = low_res_data[low_res_index];

                    if current_block_type == BlockType::Air {
                        continue;
                    }

                    let face_textures = match current_block_type.get_face_textures() {
                        Some(textures) => textures,
                        None => {
                            continue;
                        }
                    };

                    let texture_layer_indices: [f32; 6] = core::array::from_fn(|i| {
                        let texture_name = face_textures[Self::face_texture_index(i)];
                        *texture_layers.get(texture_name).unwrap_or_else(|| {
                            eprintln!(
                                "Warning: Layer index not found for texture '{}' (LOD::Low, Block: {:?}, Chunk: {:?})",
                                texture_name, current_block_type, chunk_coord
                            );
                            &0.0
                        })
                    });

                    for face_index in 0..6 {
                        let (nlx, nly, nlz) =
                            Self::get_low_res_neighbor_coords(lx, ly, lz, face_index);

                        let mut should_draw_face = false;

                        if nlx < 0
                            || nlx >= LOW_RES_WIDTH as i32
                            || nly < 0
                            || nly >= LOW_RES_HEIGHT as i32
                            || nlz < 0
                            || nlz >= LOW_RES_DEPTH as i32
                        {
                            should_draw_face = true;
                        } else {
                            let neighbor_index = nlx as usize
                                + (nlz as usize) * LOW_RES_WIDTH
                                + (nly as usize) * LOW_RES_WIDTH * LOW_RES_DEPTH;
                            if !low_res_data[neighbor_index].is_solid() {
                                should_draw_face = true;
                            }
                        }

                        if should_draw_face {
                            let layer_index = texture_layer_indices[face_index];
                            let pos_x = lx as f32 * DOWNSAMPLE_FACTOR as f32
                                + (DOWNSAMPLE_FACTOR as f32 / 2.0)
                                - 0.5;
                            let pos_y = ly as f32 * DOWNSAMPLE_FACTOR as f32
                                + (DOWNSAMPLE_FACTOR as f32 / 2.0)
                                - 0.5;
                            let pos_z = lz as f32 * DOWNSAMPLE_FACTOR as f32
                                + (DOWNSAMPLE_FACTOR as f32 / 2.0)
                                - 0.5;

                            Self::add_scaled_face(
                                FaceParams {
                                    position: [pos_x, pos_y, pos_z],
                                    face_index,
                                    layer_index,
                                    scale: DOWNSAMPLE_FACTOR as f32,
                                },
                                &mut vertices,
                                &mut indices,
                                &mut index_offset,
                            );
                        }
                    }
                }
            }
        }

        if vertices.is_empty() {
            None
        } else {
            Some(Mesh { vertices, indices })
        }
    }

    fn downsample_chunk(&self, chunk_data: &ChunkData) -> Vec<BlockType> {
        let mut low_res_data = vec![BlockType::Air; LOW_RES_WIDTH * LOW_RES_HEIGHT * LOW_RES_DEPTH];
        let factor = DOWNSAMPLE_FACTOR;

        for ly in 0..LOW_RES_HEIGHT {
            for lz in 0..LOW_RES_DEPTH {
                for lx in 0..LOW_RES_WIDTH {
                    let x_start = lx * factor;
                    let y_start = ly * factor;
                    let z_start = lz * factor;

                    let representative_block =
                        Self::calculate_representative_block(chunk_data, x_start, y_start, z_start);

                    let index = lx + lz * LOW_RES_WIDTH + ly * LOW_RES_WIDTH * LOW_RES_DEPTH;
                    low_res_data[index] = representative_block;
                }
            }
        }
        low_res_data
    }

    #[inline]
    fn calculate_representative_block(
        chunk_data: &ChunkData,
        x_start: usize,
        y_start: usize,
        z_start: usize,
    ) -> BlockType {
        let mut exposed_block_counts: HashMap<BlockType, usize> = HashMap::new();
        let mut internal_block_counts: HashMap<BlockType, usize> = HashMap::new();
        let factor = DOWNSAMPLE_FACTOR;

        for y_offset in 0..factor {
            for z_offset in 0..factor {
                for x_offset in 0..factor {
                    let x = x_start + x_offset;
                    let y = y_start + y_offset;
                    let z = z_start + z_offset;

                    if x < CHUNK_WIDTH && y < CHUNK_HEIGHT && z < CHUNK_DEPTH {
                        let high_res_block = chunk_data.get_block(x, y, z);

                        if high_res_block.is_solid() {
                            let mut is_exposed = false;
                            for face_index in 0..6 {
                                let (nx, ny, nz) = Self::get_neighbor_coords(x, y, z, face_index);
                                let neighbor_block =
                                    Self::get_block_within_chunk(chunk_data, nx, ny, nz);
                                if neighbor_block == BlockType::Air {
                                    is_exposed = true;
                                    break;
                                }
                            }

                            if is_exposed {
                                *exposed_block_counts.entry(high_res_block).or_insert(0) += 1;
                            } else {
                                *internal_block_counts.entry(high_res_block).or_insert(0) += 1;
                            }
                        }
                    }
                }
            }
        }

        let exposed_choice = Self::find_most_frequent_stable(&exposed_block_counts);
        let internal_choice = Self::find_most_frequent_stable(&internal_block_counts);

        exposed_choice.or(internal_choice).unwrap_or(BlockType::Air)
    }

    #[inline]
    fn get_neighbor_coords(x: usize, y: usize, z: usize, face_index: usize) -> (i32, i32, i32) {
        let (x, y, z) = (x as i32, y as i32, z as i32);
        match face_index {
            0 => (x + 1, y, z), // Right (+X)
            1 => (x - 1, y, z), // Left (-X)
            2 => (x, y + 1, z), // Top (+Y)
            3 => (x, y - 1, z), // Bottom (-Y)
            4 => (x, y, z + 1), // Front (+Z)
            5 => (x, y, z - 1), // Back (-Z)
            _ => (x, y, z),
        }
    }

    #[inline]
    fn get_low_res_neighbor_coords(
        lx: usize,
        ly: usize,
        lz: usize,
        face_index: usize,
    ) -> (i32, i32, i32) {
        let (lx, ly, lz) = (lx as i32, ly as i32, lz as i32);
        match face_index {
            0 => (lx + 1, ly, lz), // Right (+X)
            1 => (lx - 1, ly, lz), // Left (-X)
            2 => (lx, ly + 1, lz), // Top (+Y)
            3 => (lx, ly - 1, lz), // Bottom (-Y)
            4 => (lx, ly, lz + 1), // Front (+Z)
            5 => (lx, ly, lz - 1), // Back (-Z)
            _ => (lx, ly, lz),
        }
    }

    fn find_most_frequent_stable(counts: &HashMap<BlockType, usize>) -> Option<BlockType> {
        if counts.is_empty() {
            return None;
        }

        let max_count = match counts.values().max() {
            Some(&c) => c,
            None => return None,
        };

        if max_count == 0 {
            return None;
        }

        let mut candidates: Vec<BlockType> = counts
            .iter()
            .filter(|&(_, &count)| count == max_count)
            .map(|(&block_type, _)| block_type)
            .collect();

        candidates.sort_unstable();

        candidates.first().copied()
    }

    fn get_block_within_chunk(chunk_data: &ChunkData, x: i32, y: i32, z: i32) -> BlockType {
        if x >= 0
            && x < CHUNK_WIDTH as i32
            && y >= 0
            && y < CHUNK_HEIGHT as i32
            && z >= 0
            && z < CHUNK_DEPTH as i32
        {
            chunk_data.get_block(x as usize, y as usize, z as usize)
        } else {
            BlockType::Air
        }
    }

    #[inline]
    fn face_texture_index(face_index: usize) -> usize {
        match face_index {
            0 => 2, // Right (+X) -> Index 2
            1 => 3, // Left (-X)  -> Index 3
            2 => 0, // Top (+Y)   -> Index 0
            3 => 1, // Bottom (-Y)-> Index 1
            4 => 4, // Front (+Z) -> Index 4
            5 => 5, // Back (-Z)  -> Index 5
            _ => 0, // Default fallback
        }
    }

    fn add_scaled_face(
        params: FaceParams,
        vertices: &mut Vec<f32>,
        indices: &mut Vec<u32>,
        index_offset: &mut u32,
    ) {
        let (cx, cy, cz) = (params.position[0], params.position[1], params.position[2]);
        let layer = params.layer_index;
        let scale = params.scale;
        let half_scale = scale / 2.0;

        let uv = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

        let p = [
            [cx - half_scale, cy - half_scale, cz - half_scale], // 0: Back-Bottom-Left
            [cx + half_scale, cy - half_scale, cz - half_scale], // 1: Back-Bottom-Right
            [cx + half_scale, cy + half_scale, cz - half_scale], // 2: Back-Top-Right
            [cx - half_scale, cy + half_scale, cz - half_scale], // 3: Back-Top-Left
            [cx - half_scale, cy - half_scale, cz + half_scale], // 4: Front-Bottom-Left
            [cx + half_scale, cy - half_scale, cz + half_scale], // 5: Front-Bottom-Right
            [cx + half_scale, cy + half_scale, cz + half_scale], // 6: Front-Top-Right
            [cx - half_scale, cy + half_scale, cz + half_scale], // 7: Front-Top-Left
        ];

        let (vertex_indices, uv_indices): ([usize; 4], [usize; 4]) = match params.face_index {
            0 => ([1, 2, 6, 5], [0, 3, 2, 1]), // Right (+X) - UVs flipped horizontally? Check texture orientation
            1 => ([4, 7, 3, 0], [0, 3, 2, 1]), // Left (-X)  - UVs flipped horizontally?
            2 => ([3, 7, 6, 2], [0, 1, 2, 3]), // Top (+Y)
            3 => ([1, 5, 4, 0], [0, 1, 2, 3]), // Bottom (-Y)
            4 => ([4, 5, 6, 7], [0, 1, 2, 3]), // Front (+Z)
            5 => ([1, 0, 3, 2], [0, 1, 2, 3]), // Back (-Z)
            _ => unreachable!(),
        };

        for i in 0..4 {
            vertices.extend_from_slice(&p[vertex_indices[i]]);
            vertices.extend_from_slice(&uv[uv_indices[i]]);
            vertices.push(layer);
        }

        indices.extend_from_slice(&[
            *index_offset,
            *index_offset + 1,
            *index_offset + 2,
            *index_offset,
            *index_offset + 2,
            *index_offset + 3,
        ]);
        *index_offset += 4;
    }
}

impl Default for MeshGenerator {
    fn default() -> Self {
        Self::new()
    }
}
