use crate::components::{world_to_chunk_coords, ChunkCoord, ChunkData, ChunkDirty, Renderable};
use crate::state::GameState;
use fnv::{FnvHashMap, FnvHashSet};
use hecs::Entity;

pub struct ChunkLoadingSystem {
    last_camera_chunk_coord: Option<ChunkCoord>,
}

impl ChunkLoadingSystem {
    pub fn new() -> Self {
        Self {
            last_camera_chunk_coord: None,
        }
    }

    pub fn update(&mut self, game_state: &mut GameState) {
        let camera_pos = game_state.camera.position;
        let current_cam_coord = world_to_chunk_coords(
            camera_pos.x.floor() as i32,
            camera_pos.y.floor() as i32,
            camera_pos.z.floor() as i32,
        );

        if self.last_camera_chunk_coord == Some(current_cam_coord) {
            return;
        }
        self.last_camera_chunk_coord = Some(current_cam_coord);
        println!("Camera entered chunk: {:?}", current_cam_coord);

        let render_dist = game_state.config.render_distance;

        let mut target_chunks = FnvHashSet::<ChunkCoord>::default();
        for cz in -render_dist..=render_dist {
            for cy in -render_dist..=render_dist {
                for cx in -render_dist..=render_dist {
                    target_chunks.insert(ChunkCoord(
                        current_cam_coord.0 + cx,
                        current_cam_coord.1 + cy,
                        current_cam_coord.2 + cz,
                    ));
                }
            }
        }

        let mut loaded_chunks_map = FnvHashMap::<ChunkCoord, Entity>::default();
        for (entity, &coord) in game_state.world.query::<&ChunkCoord>().iter() {
            loaded_chunks_map.insert(coord, entity);
        }
        let loaded_chunk_coords: FnvHashSet<ChunkCoord> =
            loaded_chunks_map.keys().copied().collect();

        let chunks_to_load = target_chunks
            .difference(&loaded_chunk_coords)
            .copied()
            .collect::<Vec<_>>();
        let chunks_to_unload = loaded_chunk_coords
            .difference(&target_chunks)
            .copied()
            .collect::<Vec<_>>();

        for coord_to_unload in chunks_to_unload {
            if let Some(entity) = loaded_chunks_map.get(&coord_to_unload) {
                if let Ok(renderable) = game_state.world.get::<&Renderable>(*entity) {
                    let mesh_id = renderable.mesh_id;
                    game_state.renderer.cleanup_mesh_buffers(mesh_id);
                    game_state.mesh_registry.remove_mesh(mesh_id);
                }

                if let Err(e) = game_state.world.despawn(*entity) {
                    eprintln!(
                        "Error despawning entity {:?} for chunk {:?}: {}",
                        entity, coord_to_unload, e
                    );
                }

                game_state.chunk_entity_map.remove(&coord_to_unload);
            } else {
                eprintln!(
                    "Warning: Chunk {:?} marked for unload but not found in loaded map.",
                    coord_to_unload
                );
                game_state.chunk_entity_map.remove(&coord_to_unload);
            }
        }

        for coord_to_load in chunks_to_load {
            if game_state.chunk_entity_map.contains_key(&coord_to_load) {
                if let Some(entity) = game_state.chunk_entity_map.get(&coord_to_load) {
                    if game_state.world.contains(*entity) {
                        let _ = game_state.world.insert(*entity, (ChunkDirty,));
                    } else {
                        game_state.chunk_entity_map.remove(&coord_to_load);
                    }
                } else {
                    game_state.chunk_entity_map.remove(&coord_to_load);
                }
                if game_state.chunk_entity_map.contains_key(&coord_to_load) {
                    continue;
                }
            }

            // TODO: Implement actual chunk data generation from noise and stuff
            let chunk_data = ChunkData::new();

            let new_entity = game_state
                .world
                .spawn((coord_to_load, chunk_data, ChunkDirty));

            game_state
                .chunk_entity_map
                .insert(coord_to_load, new_entity);
        }
    }
}

impl Default for ChunkLoadingSystem {
    fn default() -> Self {
        Self::new()
    }
}
