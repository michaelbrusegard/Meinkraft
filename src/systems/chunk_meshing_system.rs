use crate::components::{
    chunk_coord_to_world_pos, ChunkCoord, ChunkData, ChunkDirty, Renderable, Transform,
};
use crate::state::GameState;
use hecs::Entity;

pub struct ChunkMeshingSystem {}

impl ChunkMeshingSystem {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&self, game_state: &mut GameState) {
        let dirty_chunks_to_process: Vec<(Entity, ChunkCoord)> = game_state
            .world
            .query::<(&ChunkCoord, &ChunkDirty)>()
            .iter()
            .map(|(entity, (coord, _dirty))| (entity, *coord))
            .collect();

        if dirty_chunks_to_process.is_empty() {
            return;
        }

        for (entity, chunk_coord) in dirty_chunks_to_process {
            if !game_state.world.contains(entity) {
                game_state.chunk_entity_map.remove(&chunk_coord);
                continue;
            }

            let maybe_mesh = {
                let chunk_data = match game_state.world.get::<&ChunkData>(entity) {
                    Ok(data) => data,
                    Err(_) => {
                        eprintln!(
                            "Error: ChunkData missing for entity {:?} at {:?} during meshing phase.",
                            entity, chunk_coord
                        );
                        continue;
                    }
                };

                game_state.mesh_generator.generate_chunk_mesh(
                    chunk_coord,
                    &chunk_data,
                    game_state,
                    &game_state.texture_manager,
                )
            };

            let has_render_components = game_state
                .world
                .entity(entity)
                .is_ok_and(|eref| eref.satisfies::<(&Transform, &Renderable)>());
            let existing_mesh_id = game_state
                .world
                .get::<&Renderable>(entity)
                .map(|r| r.mesh_id)
                .ok();

            match maybe_mesh {
                Some(mesh) => {
                    let new_mesh_id = match existing_mesh_id {
                        Some(id) => {
                            game_state
                                .mesh_registry
                                .update_mesh(id, mesh.vertices, mesh.indices)
                        }
                        None => game_state
                            .mesh_registry
                            .register_mesh(mesh.vertices, mesh.indices),
                    };

                    game_state.renderer.upload_mesh_buffers(
                        new_mesh_id,
                        &game_state.mesh_registry.meshes[&new_mesh_id].vertices,
                        &game_state.mesh_registry.meshes[&new_mesh_id].indices,
                    );

                    let world_pos = chunk_coord_to_world_pos(chunk_coord);
                    if let Err(e) = game_state.world.insert(
                        entity,
                        (
                            Transform::new(world_pos, glam::Vec3::ZERO, glam::Vec3::ONE),
                            Renderable::new(new_mesh_id),
                        ),
                    ) {
                        eprintln!("Failed to insert render components for {:?}: {}", entity, e);
                    }
                }
                None => {
                    if has_render_components {
                        match game_state.world.remove::<(Transform, Renderable)>(entity) {
                            Ok(_) => {} // Successfully removed
                            Err(e) => eprintln!(
                                "Failed to remove render components for {:?}: {}",
                                entity, e
                            ),
                        }
                    }
                    if let Some(mesh_id) = existing_mesh_id {
                        game_state.renderer.cleanup_mesh_buffers(mesh_id);
                        game_state.mesh_registry.remove_mesh(mesh_id);
                    }
                }
            }

            match game_state.world.remove_one::<ChunkDirty>(entity) {
                Ok(_) => {}
                Err(e) => eprintln!("Failed to remove ChunkDirty tag for {:?}: {}", entity, e),
            }

            game_state.chunk_entity_map.insert(chunk_coord, entity);
        }
    }
}

impl Default for ChunkMeshingSystem {
    fn default() -> Self {
        Self::new()
    }
}
