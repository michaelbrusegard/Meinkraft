use crate::components::{ChunkCoord, ChunkData, CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_WIDTH};
use crate::resources::{Mesh, TextureManager};
use crate::state::GameState;

struct FaceParams {
    position: [f32; 3],
    face_index: usize,
    uvs: [f32; 4],
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
        game_state: &GameState,
        texture_manager: &TextureManager,
    ) -> Option<Mesh> {
        let mut vertices: Vec<f32> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut index_offset: u32 = 0;

        let ChunkCoord(cx, cy, cz) = chunk_coord;
        let chunk_origin_x = cx * CHUNK_WIDTH as i32;
        let chunk_origin_y = cy * CHUNK_HEIGHT as i32;
        let chunk_origin_z = cz * CHUNK_DEPTH as i32;

        for y in 0..CHUNK_HEIGHT {
            for z in 0..CHUNK_DEPTH {
                for x in 0..CHUNK_WIDTH {
                    let current_block_type = chunk_data.get_block(x, y, z);

                    if !current_block_type.is_solid() {
                        continue;
                    }

                    let world_x = chunk_origin_x + x as i32;
                    let world_y = chunk_origin_y + y as i32;
                    let world_z = chunk_origin_z + z as i32;

                    let face_textures = match current_block_type.get_face_textures() {
                        Some(textures) => textures,
                        None => continue,
                    };

                    let neighbors = [
                        game_state.get_block_world(world_x + 1, world_y, world_z), // Right (+X)
                        game_state.get_block_world(world_x - 1, world_y, world_z), // Left (-X)
                        game_state.get_block_world(world_x, world_y + 1, world_z), // Top (+Y)
                        game_state.get_block_world(world_x, world_y - 1, world_z), // Bottom (-Y)
                        game_state.get_block_world(world_x, world_y, world_z + 1), // Front (+Z)
                        game_state.get_block_world(world_x, world_y, world_z - 1), // Back (-Z)
                    ];

                    for face_index in 0..6 {
                        if !neighbors[face_index].is_solid() {
                            let texture_name = face_textures[Self::face_texture_index(face_index)];
                            let uvs = match texture_manager.get_uvs(texture_name) {
                                Some(uv) => uv,
                                None => continue,
                            };
                            Self::add_face(
                                FaceParams {
                                    position: [x as f32, y as f32, z as f32],
                                    face_index,
                                    uvs,
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

    #[inline]
    fn face_texture_index(face_index: usize) -> usize {
        match face_index {
            0 => 2,
            1 => 3,
            2 => 4,
            3 => 5,
            4 => 0,
            5 => 1,
            _ => 0,
        }
    }

    fn add_face(
        params: FaceParams,
        vertices: &mut Vec<f32>,
        indices: &mut Vec<u32>,
        index_offset: &mut u32,
    ) {
        let (x, y, z) = (params.position[0], params.position[1], params.position[2]);
        let (u_min, v_min, u_max, v_max) =
            (params.uvs[0], params.uvs[1], params.uvs[2], params.uvs[3]);

        let p = [
            [x - 0.5, y - 0.5, z - 0.5], // 0: Back-Bottom-Left
            [x + 0.5, y - 0.5, z - 0.5], // 1: Back-Bottom-Right
            [x + 0.5, y + 0.5, z - 0.5], // 2: Back-Top-Right
            [x - 0.5, y + 0.5, z - 0.5], // 3: Back-Top-Left
            [x - 0.5, y - 0.5, z + 0.5], // 4: Front-Bottom-Left
            [x + 0.5, y - 0.5, z + 0.5], // 5: Front-Bottom-Right
            [x + 0.5, y + 0.5, z + 0.5], // 6: Front-Top-Right
            [x - 0.5, y + 0.5, z + 0.5], // 7: Front-Top-Left
        ];

        let uv = [
            [u_min, v_min], // 0: Bottom-Left
            [u_max, v_min], // 1: Bottom-Right
            [u_max, v_max], // 2: Top-Right
            [u_min, v_max], // 3: Top-Left
        ];

        let (vertex_indices, uv_indices): ([usize; 4], [usize; 4]) = match params.face_index {
            0 => ([1, 2, 6, 5], [0, 3, 2, 1]), // Right (+X) CCW: bbr, btr, ftr, fbr
            1 => ([4, 7, 3, 0], [0, 3, 2, 1]), // Left (-X) CCW: fbl, ftl, btl, bbl
            2 => ([3, 7, 6, 2], [0, 1, 2, 3]), // Top (+Y) CCW: btl, ftl, ftr, btr
            3 => ([1, 5, 4, 0], [0, 1, 2, 3]), // Bottom (-Y) CCW: bbr, fbr, fbl, bbl
            4 => ([4, 5, 6, 7], [0, 1, 2, 3]), // Front (+Z) CCW: fbl, fbr, ftr, ftl
            5 => ([1, 0, 3, 2], [0, 1, 2, 3]), // Back (-Z) CCW: bbr, bbl, btl, btr
            _ => unreachable!(),
        };

        for i in 0..4 {
            vertices.extend_from_slice(&p[vertex_indices[i]]);
            vertices.extend_from_slice(&uv[uv_indices[i]]);
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
