use crate::components::{
    chunk_coord_to_world_pos, ChunkCoord, ChunkData, ChunkDirty, Renderable, Transform,
};
use crate::persistence::NeighborData;
use crate::state::GameState;
use fnv::FnvHashSet;
use hecs::Entity;
use std::ops::Deref;

type MeshRequest = (Entity, ChunkCoord, ChunkData, NeighborData);

pub struct ChunkMeshingSystem {
    pending_mesh_requests: FnvHashSet<ChunkCoord>,
}

impl ChunkMeshingSystem {
    pub fn new() -> Self {
        Self {
            pending_mesh_requests: FnvHashSet::default(),
        }
    }

    pub fn process_mesh_results_and_requests(&mut self, game_state: &mut GameState) {
        self.process_mesh_results(game_state);

        let (requests_to_send, entities_to_undirty) = self.collect_meshing_requests(game_state);

        for (entity, coord, data, neighbors) in requests_to_send {
            if game_state
                .mesh_request_tx
                .send((entity, coord, data, neighbors))
                .is_ok()
            {
                self.pending_mesh_requests.insert(coord);
            } else {
                eprintln!(
                    "Failed to send mesh request for {:?}, channel closed?",
                    coord
                );
            }
        }

        for entity in entities_to_undirty {
            let coord = match game_state.world.get::<&ChunkCoord>(entity) {
                Ok(c) => *c,
                Err(_) => continue,
            };
            if self.pending_mesh_requests.contains(&coord) && game_state.world.contains(entity) {
                if let Err(e) = game_state.world.remove_one::<ChunkDirty>(entity) {
                    eprintln!("Failed to remove ChunkDirty tag for {:?}: {}", entity, e);
                }
            }
        }
    }

    fn process_mesh_results(&mut self, game_state: &mut GameState) {
        let mut results_to_process = Vec::new();
        while let Ok(result) = game_state.mesh_result_rx.try_recv() {
            results_to_process.push(result);
        }

        for (entity, coord, maybe_mesh) in results_to_process {
            self.pending_mesh_requests.remove(&coord);

            if !game_state.world.contains(entity) {
                continue;
            }

            match maybe_mesh {
                Some(mesh) => {
                    let existing_mesh_id = game_state
                        .world
                        .get::<&Renderable>(entity)
                        .map(|r| r.mesh_id)
                        .ok();

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

                    if let Some(mesh_data) = game_state.mesh_registry.meshes.get(&new_mesh_id) {
                        game_state.renderer.upload_mesh_buffers(
                            new_mesh_id,
                            &mesh_data.vertices,
                            &mesh_data.indices,
                        );

                        let world_pos = chunk_coord_to_world_pos(coord);
                        if let Err(e) = game_state.world.insert(
                            entity,
                            (
                                Transform::new(world_pos, glam::Vec3::ZERO, glam::Vec3::ONE),
                                Renderable::new(new_mesh_id),
                            ),
                        ) {
                            eprintln!(
                                "Failed to insert render components for {:?} at {:?}: {}",
                                entity, coord, e
                            );
                            game_state.renderer.cleanup_mesh_buffers(new_mesh_id);
                            game_state.mesh_registry.remove_mesh(new_mesh_id);
                        }
                    } else {
                        eprintln!(
                            "Mesh data missing in registry after register/update for ID {}",
                            new_mesh_id
                        );
                    }
                }
                None => {
                    let mesh_id_to_remove = game_state
                        .world
                        .get::<&Renderable>(entity)
                        .map(|r| r.mesh_id)
                        .ok();

                    if let Some(id) = mesh_id_to_remove {
                        if let Err(e) = game_state.world.remove::<(Transform, Renderable)>(entity) {
                            eprintln!(
                                "Failed to remove render components for {:?} at {:?}: {}",
                                entity, coord, e
                            );
                        }
                        game_state.renderer.cleanup_mesh_buffers(id);
                        game_state.mesh_registry.remove_mesh(id);
                    }
                }
            }
        }
    }

    fn collect_meshing_requests(
        &mut self,
        game_state: &GameState,
    ) -> (Vec<MeshRequest>, Vec<Entity>) {
        let mut requests_to_send = Vec::new();
        let mut entities_to_undirty = Vec::new();

        let query = game_state
            .world
            .query::<(&ChunkCoord, &ChunkData, &ChunkDirty)>()
            .iter()
            .filter(|(_entity, (coord, _data, _dirty))| !self.pending_mesh_requests.contains(coord))
            .map(|(entity, (coord, data, _dirty))| (entity, *coord, data.clone()))
            .collect::<Vec<_>>();

        for (entity, chunk_coord, chunk_data) in query {
            match self.get_neighbor_data(chunk_coord, game_state) {
                Some(neighbor_data) => {
                    requests_to_send.push((
                        entity,
                        chunk_coord,
                        chunk_data,
                        Box::new(neighbor_data),
                    ));
                    entities_to_undirty.push(entity);
                }
                None => continue,
            }
        }
        (requests_to_send, entities_to_undirty)
    }

    fn get_neighbor_data(
        &self,
        coord: ChunkCoord,
        game_state: &GameState,
    ) -> Option<[Option<ChunkData>; 6]> {
        let neighbor_offsets = [
            (1, 0, 0),
            (-1, 0, 0),
            (0, 1, 0),
            (0, -1, 0),
            (0, 0, 1),
            (0, 0, -1),
        ];
        let mut neighbor_data: [Option<ChunkData>; 6] = Default::default();

        for (i, offset) in neighbor_offsets.iter().enumerate() {
            let neighbor_coord =
                ChunkCoord(coord.0 + offset.0, coord.1 + offset.1, coord.2 + offset.2);

            if let Some(neighbor_entity) = game_state.chunk_entity_map.get(&neighbor_coord) {
                match game_state.world.get::<&ChunkData>(*neighbor_entity) {
                    Ok(data_ref) => {
                        neighbor_data[i] = Some(data_ref.deref().clone());
                    }
                    Err(_) => return None,
                }
            } else {
                return None;
            }
        }
        Some(neighbor_data)
    }
}

impl Default for ChunkMeshingSystem {
    fn default() -> Self {
        Self::new()
    }
}
