use crate::components::{
    chunk_coord_to_world_pos, world_to_chunk_coords, ChunkCoord, ChunkData, ChunkDirty, Renderable,
    Transform, LOD,
};
use crate::persistence::NeighborData;
use crate::resources::ChunkMeshData;
use crate::state::GameState;
use fnv::FnvHashSet;
use hecs::Entity;
use std::ops::Deref;

type MeshRequest = (Entity, ChunkCoord, ChunkData, NeighborData, LOD, [LOD; 6]);
type MeshResult = (Entity, ChunkCoord, Option<ChunkMeshData>, LOD);

pub struct ChunkMeshingSystem {
    pending_mesh_requests: FnvHashSet<ChunkCoord>,
    last_camera_chunk_coord_xz: Option<(i32, i32)>,
    load_distance_sq: i32,
    render_distance_sq: i32,
}

impl ChunkMeshingSystem {
    pub fn new() -> Self {
        Self {
            pending_mesh_requests: FnvHashSet::default(),
            last_camera_chunk_coord_xz: None,
            load_distance_sq: 0,
            render_distance_sq: 0,
        }
    }

    pub fn update_lod_parameters(&mut self, game_state: &GameState) {
        let camera_pos = game_state.camera.position;
        let cam_chunk_x = world_to_chunk_coords(camera_pos.x.floor() as i32, 0, 0).0;
        let cam_chunk_z = world_to_chunk_coords(0, 0, camera_pos.z.floor() as i32).2;
        self.last_camera_chunk_coord_xz = Some((cam_chunk_x, cam_chunk_z));

        let load_dist = game_state.config.load_distance;
        let render_dist = game_state.config.render_distance;
        self.load_distance_sq = load_dist * load_dist;
        self.render_distance_sq = render_dist * render_dist;
    }

    pub fn process_mesh_results_and_requests(&mut self, game_state: &mut GameState) {
        self.process_mesh_results(game_state);
        let (requests_to_send, entities_processed) = self.collect_meshing_requests(game_state);

        for (entity, coord, data, neighbors, required_lod, _neighbor_lods) in requests_to_send {
            if game_state
                .mesh_request_tx
                .send((entity, coord, data, neighbors, required_lod))
                .is_ok()
            {
                self.pending_mesh_requests.insert(coord);
            } else {
                eprintln!(
                    "Failed to send mesh request for {:?}, channel closed?",
                    coord
                );
                continue;
            }
        }

        for entity in entities_processed {
            if game_state.world.contains(entity)
                && game_state.world.get::<&ChunkDirty>(entity).is_ok()
            {
                if let Err(e) = game_state.world.remove_one::<ChunkDirty>(entity) {
                    eprintln!("Failed to remove ChunkDirty tag for {:?}: {}", entity, e);
                }
            }
        }
    }

    fn process_mesh_results(&mut self, game_state: &mut GameState) {
        let results_to_process: Vec<MeshResult> = game_state.mesh_result_rx.try_iter().collect();

        for (entity, coord, maybe_chunk_mesh_data, generated_lod) in results_to_process {
            self.pending_mesh_requests.remove(&coord);

            if !game_state.world.contains(entity) {
                continue;
            }

            let (existing_opaque_id, existing_transparent_id) =
                match game_state.world.get::<&Renderable>(entity) {
                    Ok(r) => (r.opaque_mesh_id, r.transparent_mesh_id),
                    Err(_) => (None, None),
                };

            let mut final_opaque_mesh_id: Option<usize> = None;
            let mut final_transparent_mesh_id: Option<usize> = None;
            let mut needs_component_update = false;

            match maybe_chunk_mesh_data {
                Some(chunk_mesh_data) => {
                    match chunk_mesh_data.opaque {
                        Some(mesh) => {
                            let new_id = Self::register_or_update_mesh(
                                game_state,
                                existing_opaque_id,
                                mesh.vertices,
                                mesh.indices,
                            );
                            if Self::upload_mesh_buffers(game_state, new_id) {
                                final_opaque_mesh_id = Some(new_id);
                                needs_component_update = true;
                                if let Some(old_id) = existing_opaque_id {
                                    if old_id != new_id {
                                        Self::cleanup_mesh_resources(game_state, old_id);
                                    }
                                }
                            } else {
                                Self::cleanup_mesh_resources(game_state, new_id);
                                if let Some(old_id) = existing_opaque_id {
                                    Self::cleanup_mesh_resources(game_state, old_id);
                                }
                            }
                        }
                        None => {
                            if let Some(old_id) = existing_opaque_id {
                                Self::cleanup_mesh_resources(game_state, old_id);
                                needs_component_update = true;
                            }
                        }
                    }

                    match chunk_mesh_data.transparent {
                        Some(mesh) => {
                            let new_id = Self::register_or_update_mesh(
                                game_state,
                                existing_transparent_id,
                                mesh.vertices,
                                mesh.indices,
                            );
                            if Self::upload_mesh_buffers(game_state, new_id) {
                                final_transparent_mesh_id = Some(new_id);
                                needs_component_update = true;
                                if let Some(old_id) = existing_transparent_id {
                                    if old_id != new_id {
                                        Self::cleanup_mesh_resources(game_state, old_id);
                                    }
                                }
                            } else {
                                Self::cleanup_mesh_resources(game_state, new_id);
                                if let Some(old_id) = existing_transparent_id {
                                    Self::cleanup_mesh_resources(game_state, old_id);
                                }
                            }
                        }
                        None => {
                            if let Some(old_id) = existing_transparent_id {
                                Self::cleanup_mesh_resources(game_state, old_id);
                                needs_component_update = true;
                            }
                        }
                    }

                    if needs_component_update
                        || final_opaque_mesh_id.is_some()
                        || final_transparent_mesh_id.is_some()
                    {
                        let world_pos = chunk_coord_to_world_pos(coord);
                        let new_renderable =
                            Renderable::new(final_opaque_mesh_id, final_transparent_mesh_id);
                        let components = (
                            Transform::new(world_pos, glam::Vec3::ZERO, glam::Vec3::ONE),
                            new_renderable,
                            generated_lod,
                        );

                        if let Err(e) = game_state.world.insert(entity, components) {
                            eprintln!(
                                "Failed to insert render components for {:?} at {:?}: {}",
                                entity, coord, e
                            );
                            if let Some(id) = final_opaque_mesh_id {
                                Self::cleanup_mesh_resources(game_state, id);
                            }
                            if let Some(id) = final_transparent_mesh_id {
                                Self::cleanup_mesh_resources(game_state, id);
                            }
                        }
                    } else if final_opaque_mesh_id.is_none() && final_transparent_mesh_id.is_none()
                    {
                        if let Err(e) = game_state
                            .world
                            .remove::<(Transform, Renderable, LOD)>(entity)
                        {
                            eprintln!(
                                "Failed to remove components for empty chunk {:?} at {:?}: {}",
                                entity, coord, e
                            );
                        }
                    }
                }
                None => {
                    let mut opaque_id_to_remove: Option<usize> = None;
                    let mut transparent_id_to_remove: Option<usize> = None;

                    if let Ok(renderable_ref) = game_state.world.get::<&Renderable>(entity) {
                        opaque_id_to_remove = renderable_ref.opaque_mesh_id;
                        transparent_id_to_remove = renderable_ref.transparent_mesh_id;
                    }

                    if let Some(id) = opaque_id_to_remove {
                        Self::cleanup_mesh_resources(game_state, id);
                    }
                    if let Some(id) = transparent_id_to_remove {
                        Self::cleanup_mesh_resources(game_state, id);
                    }

                    let _ = game_state
                        .world
                        .remove::<(Transform, Renderable, LOD)>(entity);
                }
            }
        }
    }

    fn register_or_update_mesh(
        game_state: &mut GameState,
        existing_id: Option<usize>,
        vertices: Vec<f32>,
        indices: Vec<u32>,
    ) -> usize {
        match existing_id {
            Some(id) => game_state.mesh_registry.update_mesh(id, vertices, indices),
            None => game_state.mesh_registry.register_mesh(vertices, indices),
        }
    }

    fn upload_mesh_buffers(game_state: &mut GameState, mesh_id: usize) -> bool {
        if let Some(mesh_data) = game_state.mesh_registry.meshes.get(&mesh_id) {
            game_state.renderer.upload_mesh_buffers(
                mesh_id,
                &mesh_data.vertices,
                &mesh_data.indices,
            );
            true
        } else {
            eprintln!(
                "Mesh data missing in registry after register/update for ID {}",
                mesh_id
            );
            false
        }
    }

    fn cleanup_mesh_resources(game_state: &mut GameState, mesh_id: usize) {
        game_state.renderer.cleanup_mesh_buffers(mesh_id);
        game_state.mesh_registry.remove_mesh(mesh_id);
    }

    fn collect_meshing_requests(&self, game_state: &GameState) -> (Vec<MeshRequest>, Vec<Entity>) {
        let mut requests_to_send: Vec<MeshRequest> = Vec::new();
        let mut entities_to_undirty = Vec::new();

        let (cam_cx, cam_cz) = match self.last_camera_chunk_coord_xz {
            Some(coords) => coords,
            None => {
                return (requests_to_send, entities_to_undirty);
            }
        };

        let query_candidates = game_state
            .world
            .query::<(&ChunkCoord, &ChunkData, Option<&ChunkDirty>, Option<&LOD>)>()
            .iter()
            .filter(|(_entity, (coord, _data, _dirty, _lod))| {
                !self.pending_mesh_requests.contains(coord)
            })
            .map(|(entity, (coord, data, dirty_opt, current_lod_opt))| {
                (
                    entity,
                    *coord,
                    data.clone(),
                    dirty_opt.is_some(),
                    current_lod_opt.copied(),
                )
            })
            .collect::<Vec<_>>();

        for (entity, chunk_coord, chunk_data, is_dirty, current_lod) in query_candidates {
            let dx = chunk_coord.0 - cam_cx;
            let dz = chunk_coord.2 - cam_cz;
            let dist_sq = dx * dx + dz * dz;

            if dist_sq > self.render_distance_sq {
                continue;
            }

            let required_lod = if dist_sq <= self.load_distance_sq {
                LOD::High
            } else {
                LOD::Low
            };

            let required_neighbor_lods =
                self.get_neighbor_lods(chunk_coord, cam_cx, cam_cz, game_state);

            let mut needs_remesh = false;

            if is_dirty {
                needs_remesh = true;
            }

            if !needs_remesh && current_lod.is_none_or(|n_curr| n_curr != required_lod) {
                needs_remesh = true;
            }

            if !needs_remesh {
                let neighbor_offsets = [
                    (1, 0, 0),
                    (-1, 0, 0),
                    (0, 1, 0),
                    (0, -1, 0),
                    (0, 0, 1),
                    (0, 0, -1),
                ];
                for i in 0..6 {
                    let offset = neighbor_offsets[i];
                    let neighbor_coord = ChunkCoord(
                        chunk_coord.0 + offset.0,
                        chunk_coord.1 + offset.1,
                        chunk_coord.2 + offset.2,
                    );

                    let neighbor_current_lod: Option<LOD> = game_state
                        .chunk_entity_map
                        .get(&neighbor_coord)
                        .and_then(|&neighbor_entity| {
                            game_state
                                .world
                                .get::<&LOD>(neighbor_entity)
                                .ok()
                                .map(|lod_ref| *lod_ref)
                        });

                    if neighbor_current_lod.is_none_or(|n_curr| n_curr != required_neighbor_lods[i])
                    {
                        needs_remesh = true;
                        break;
                    }
                }
            }

            if needs_remesh {
                match self.get_neighbor_data(chunk_coord, game_state) {
                    Some(neighbor_data) => {
                        requests_to_send.push((
                            entity,
                            chunk_coord,
                            chunk_data,
                            Box::new(neighbor_data),
                            required_lod,
                            required_neighbor_lods,
                        ));
                        entities_to_undirty.push(entity);
                    }
                    None => {
                        continue;
                    }
                }
            } else if is_dirty {
                entities_to_undirty.push(entity);
            }
        }
        (requests_to_send, entities_to_undirty)
    }

    fn get_neighbor_data(
        &self,
        coord: ChunkCoord,
        game_state: &GameState,
    ) -> Option<[Option<ChunkData>; 6]> {
        use crate::components::{MAX_CHUNK_Y, MIN_CHUNK_Y};

        let neighbor_offsets = [
            (1, 0, 0),
            (-1, 0, 0),
            (0, 1, 0),
            (0, -1, 0),
            (0, 0, 1),
            (0, 0, -1),
        ];
        let mut neighbor_data: [Option<ChunkData>; 6] = Default::default();
        let mut all_required_neighbors_present = true;

        for (i, offset) in neighbor_offsets.iter().enumerate() {
            let neighbor_coord =
                ChunkCoord(coord.0 + offset.0, coord.1 + offset.1, coord.2 + offset.2);

            if neighbor_coord.1 >= MIN_CHUNK_Y && neighbor_coord.1 <= MAX_CHUNK_Y {
                if let Some(neighbor_entity) = game_state.chunk_entity_map.get(&neighbor_coord) {
                    match game_state.world.get::<&ChunkData>(*neighbor_entity) {
                        Ok(data_ref) => {
                            neighbor_data[i] = Some(data_ref.deref().clone());
                        }
                        Err(_) => {
                            all_required_neighbors_present = false;
                            break;
                        }
                    }
                } else {
                    all_required_neighbors_present = false;
                    break;
                }
            } else {
                neighbor_data[i] = None;
            }
        }

        if all_required_neighbors_present {
            Some(neighbor_data)
        } else {
            None
        }
    }

    fn get_neighbor_lods(
        &self,
        coord: ChunkCoord,
        cam_cx: i32,
        cam_cz: i32,
        _game_state: &GameState,
    ) -> [LOD; 6] {
        let neighbor_offsets = [
            (1, 0, 0),
            (-1, 0, 0),
            (0, 1, 0),
            (0, -1, 0),
            (0, 0, 1),
            (0, 0, -1),
        ];
        let mut neighbor_lods: [LOD; 6] = [LOD::Low; 6];

        for (i, offset) in neighbor_offsets.iter().enumerate() {
            if offset.1 != 0 {
                let dx = coord.0 - cam_cx;
                let dz = coord.2 - cam_cz;
                let dist_sq = dx * dx + dz * dz;
                neighbor_lods[i] = if dist_sq <= self.load_distance_sq {
                    LOD::High
                } else {
                    LOD::Low
                };
                continue;
            }

            let neighbor_coord =
                ChunkCoord(coord.0 + offset.0, coord.1 + offset.1, coord.2 + offset.2);

            let dx = neighbor_coord.0 - cam_cx;
            let dz = neighbor_coord.2 - cam_cz;
            let dist_sq = dx * dx + dz * dz;

            neighbor_lods[i] = if dist_sq <= self.load_distance_sq {
                LOD::High
            } else {
                LOD::Low
            };
        }
        neighbor_lods
    }
}

impl Default for ChunkMeshingSystem {
    fn default() -> Self {
        Self::new()
    }
}
