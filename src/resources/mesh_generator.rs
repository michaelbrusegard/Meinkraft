use crate::components::{
    BlockType, ChunkCoord, ChunkData, CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_WIDTH, LOD,
};
use crate::resources::{ChunkMeshData, Mesh};
use std::collections::HashMap;

trait EffectiveBlockDataSource {
    fn get_effective_block(
        &self,
        x: usize,
        y: usize,
        z: usize,
        eff_w: usize,
        eff_d: usize,
    ) -> BlockType;
}

impl EffectiveBlockDataSource for ChunkData {
    #[inline]
    fn get_effective_block(
        &self,
        x: usize,
        y: usize,
        z: usize,
        _eff_w: usize,
        _eff_d: usize,
    ) -> BlockType {
        self.get_block(x, y, z)
    }
}

impl EffectiveBlockDataSource for Vec<BlockType> {
    #[inline]
    fn get_effective_block(
        &self,
        x: usize,
        y: usize,
        z: usize,
        eff_w: usize,
        eff_d: usize,
    ) -> BlockType {
        let index = x + z * eff_w + y * eff_w * eff_d;
        *self.get(index).unwrap_or(&BlockType::Air)
    }
}

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
    ) -> Option<ChunkMeshData> {
        let mut opaque_vertices: Vec<f32> = Vec::new();
        let mut opaque_indices: Vec<u32> = Vec::new();
        let mut opaque_index_offset: u32 = 0;

        let mut transparent_vertices: Vec<f32> = Vec::new();
        let mut transparent_indices: Vec<u32> = Vec::new();
        let mut transparent_index_offset: u32 = 0;

        let scale_factor = lod.scale_factor();
        let downsample_factor = lod.downsample_factor();

        if CHUNK_WIDTH % downsample_factor != 0
            || CHUNK_HEIGHT % downsample_factor != 0
            || CHUNK_DEPTH % downsample_factor != 0
        {
            eprintln!("Warning: Chunk dimensions ({},{},{}) not divisible by downsample factor {} for LOD {:?}. Skipping mesh generation for {:?}.",
                CHUNK_WIDTH, CHUNK_HEIGHT, CHUNK_DEPTH, downsample_factor, lod, chunk_coord);
            return None;
        }

        let effective_width = CHUNK_WIDTH / downsample_factor;
        let effective_height = CHUNK_HEIGHT / downsample_factor;
        let effective_depth = CHUNK_DEPTH / downsample_factor;

        let downsampled_data;
        let data_to_mesh: &dyn EffectiveBlockDataSource = if downsample_factor > 1 {
            if effective_width == 0 || effective_height == 0 || effective_depth == 0 {
                eprintln!("Warning: Effective dimensions are zero after downsampling for LOD {:?} in chunk {:?}. Skipping.", lod, chunk_coord);
                return None;
            }
            downsampled_data = self.downsample_chunk(chunk_data, downsample_factor);
            &downsampled_data
        } else {
            chunk_data
        };

        for ey in 0..effective_height {
            for ez in 0..effective_depth {
                for ex in 0..effective_width {
                    let current_block_type = data_to_mesh.get_effective_block(
                        ex,
                        ey,
                        ez,
                        effective_width,
                        effective_depth,
                    );

                    if current_block_type == BlockType::Air {
                        continue;
                    }

                    let face_textures = match current_block_type.get_face_textures() {
                        Some(textures) => textures,
                        None => continue,
                    };

                    for face_index in 0..6 {
                        let (nex, ney, nez) =
                            Self::get_effective_neighbor_coords(ex, ey, ez, face_index);

                        let neighbor_block_type = if nex < 0
                            || nex >= effective_width as i32
                            || ney < 0
                            || ney >= effective_height as i32
                            || nez < 0
                            || nez >= effective_depth as i32
                        {
                            let neighbor_chunk_index = Self::face_to_neighbor_index(face_index);
                            match &neighbors[neighbor_chunk_index] {
                                Some(neighbor_chunk_data) => {
                                    let (nnex, nney, nnez) = Self::wrap_effective_neighbor_coords(
                                        nex,
                                        ney,
                                        nez,
                                        effective_width,
                                        effective_height,
                                        effective_depth,
                                    );

                                    if downsample_factor > 1 {
                                        let x_start = nnex * downsample_factor;
                                        let y_start = nney * downsample_factor;
                                        let z_start = nnez * downsample_factor;
                                        Self::calculate_representative_block(
                                            neighbor_chunk_data,
                                            x_start,
                                            y_start,
                                            z_start,
                                            downsample_factor,
                                        )
                                    } else {
                                        neighbor_chunk_data.get_block(nnex, nney, nnez)
                                    }
                                }
                                None => BlockType::Air,
                            }
                        } else {
                            data_to_mesh.get_effective_block(
                                nex as usize,
                                ney as usize,
                                nez as usize,
                                effective_width,
                                effective_depth,
                            )
                        };

                        let should_draw_face = match neighbor_block_type {
                            BlockType::Air => true,
                            neighbor if !neighbor.is_culled_by() => {
                                current_block_type.is_culled_by()
                                    || current_block_type != neighbor_block_type
                            }
                            _ => !current_block_type.is_culled_by(),
                        };

                        if should_draw_face {
                            let pos_x =
                                (ex * downsample_factor) as f32 + (scale_factor / 2.0) - 0.5;
                            let pos_y =
                                (ey * downsample_factor) as f32 + (scale_factor / 2.0) - 0.5;
                            let pos_z =
                                (ez * downsample_factor) as f32 + (scale_factor / 2.0) - 0.5;

                            let texture_name = face_textures[Self::face_texture_index(face_index)];
                            let layer_index = *texture_layers.get(texture_name).unwrap_or_else(|| {
                                eprintln!(
                                    "Warning: Layer index not found for texture '{}' (LOD::{:?}, Block: {:?}, Chunk: {:?})",
                                    texture_name, lod, current_block_type, chunk_coord
                                );
                                &0.0
                            });

                            let is_transparent = !current_block_type.is_culled_by();
                            let (target_vertices, target_indices, target_index_offset) =
                                if is_transparent {
                                    (
                                        &mut transparent_vertices,
                                        &mut transparent_indices,
                                        &mut transparent_index_offset,
                                    )
                                } else {
                                    (
                                        &mut opaque_vertices,
                                        &mut opaque_indices,
                                        &mut opaque_index_offset,
                                    )
                                };

                            Self::add_scaled_face(
                                FaceParams {
                                    position: [pos_x, pos_y, pos_z],
                                    face_index,
                                    layer_index,
                                    scale: scale_factor,
                                },
                                target_vertices,
                                target_indices,
                                target_index_offset,
                            );
                        }
                    }
                }
            }
        }

        let opaque_mesh = if !opaque_vertices.is_empty() {
            Some(Mesh {
                vertices: opaque_vertices,
                indices: opaque_indices,
            })
        } else {
            None
        };

        let transparent_mesh = if !transparent_vertices.is_empty() {
            Some(Mesh {
                vertices: transparent_vertices,
                indices: transparent_indices,
            })
        } else {
            None
        };

        if opaque_mesh.is_some() || transparent_mesh.is_some() {
            Some(ChunkMeshData {
                opaque: opaque_mesh,
                transparent: transparent_mesh,
            })
        } else {
            None
        }
    }

    fn downsample_chunk(&self, chunk_data: &ChunkData, factor: usize) -> Vec<BlockType> {
        if factor <= 1 {
            panic!("Downsample factor must be > 1, got {}", factor);
        }
        let eff_w = CHUNK_WIDTH / factor;
        let eff_h = CHUNK_HEIGHT / factor;
        let eff_d = CHUNK_DEPTH / factor;
        let mut low_res_data = vec![BlockType::Air; eff_w * eff_h * eff_d];

        for ey in 0..eff_h {
            for ez in 0..eff_d {
                for ex in 0..eff_w {
                    let x_start = ex * factor;
                    let y_start = ey * factor;
                    let z_start = ez * factor;

                    let representative_block = Self::calculate_representative_block(
                        chunk_data, x_start, y_start, z_start, factor,
                    );

                    let index = ex + ez * eff_w + ey * eff_w * eff_d;
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
        factor: usize,
    ) -> BlockType {
        let mut exposed_block_counts: HashMap<BlockType, usize> = HashMap::new();
        let mut internal_block_counts: HashMap<BlockType, usize> = HashMap::new();

        for y_offset in 0..factor {
            for z_offset in 0..factor {
                for x_offset in 0..factor {
                    let x = x_start + x_offset;
                    let y = y_start + y_offset;
                    let z = z_start + z_offset;

                    if x < CHUNK_WIDTH && y < CHUNK_HEIGHT && z < CHUNK_DEPTH {
                        let high_res_block = chunk_data.get_block(x, y, z);

                        if high_res_block != BlockType::Air {
                            let mut is_exposed = false;
                            for face_index in 0..6 {
                                let (nx, ny, nz) =
                                    Self::get_high_res_neighbor_coords(x, y, z, face_index);

                                let neighbor_block = if nx >= 0
                                    && nx < CHUNK_WIDTH as i32
                                    && ny >= 0
                                    && ny < CHUNK_HEIGHT as i32
                                    && nz >= 0
                                    && nz < CHUNK_DEPTH as i32
                                {
                                    chunk_data.get_block(nx as usize, ny as usize, nz as usize)
                                } else {
                                    BlockType::Air
                                };

                                if !neighbor_block.is_culled_by() {
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

        Self::find_most_frequent_stable(&exposed_block_counts)
            .or_else(|| Self::find_most_frequent_stable(&internal_block_counts))
            .unwrap_or(BlockType::Air)
    }

    const FACE_OFFSETS: [[i32; 3]; 6] = [
        [1, 0, 0],
        [-1, 0, 0],
        [0, 1, 0],
        [0, -1, 0],
        [0, 0, 1],
        [0, 0, -1],
    ];

    #[inline]
    fn get_high_res_neighbor_coords(
        x: usize,
        y: usize,
        z: usize,
        face_index: usize,
    ) -> (i32, i32, i32) {
        let offset = Self::FACE_OFFSETS[face_index];
        (
            x as i32 + offset[0],
            y as i32 + offset[1],
            z as i32 + offset[2],
        )
    }

    #[inline]
    fn get_effective_neighbor_coords(
        ex: usize,
        ey: usize,
        ez: usize,
        face_index: usize,
    ) -> (i32, i32, i32) {
        let offset = Self::FACE_OFFSETS[face_index];
        (
            ex as i32 + offset[0],
            ey as i32 + offset[1],
            ez as i32 + offset[2],
        )
    }

    #[inline]
    fn face_to_neighbor_index(face_index: usize) -> usize {
        face_index
    }

    #[inline]
    fn wrap_effective_neighbor_coords(
        nex: i32,
        ney: i32,
        nez: i32,
        eff_w: usize,
        eff_h: usize,
        eff_d: usize,
    ) -> (usize, usize, usize) {
        let wrapped_x = nex.rem_euclid(eff_w as i32) as usize;
        let wrapped_y = ney.rem_euclid(eff_h as i32) as usize;
        let wrapped_z = nez.rem_euclid(eff_d as i32) as usize;
        (wrapped_x, wrapped_y, wrapped_z)
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

        let uv = [[0.0, 0.0], [scale, 0.0], [scale, scale], [0.0, scale]];

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
            0 => ([1, 2, 6, 5], [0, 3, 2, 1]), // Right (+X)
            1 => ([4, 7, 3, 0], [0, 3, 2, 1]), // Left (-X)
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
