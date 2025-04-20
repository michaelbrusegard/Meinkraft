use crate::components::{
    world_to_chunk_coords, ChunkCoord, ChunkData, ChunkDirty, ChunkModified, Renderable,
    MAX_CHUNK_Y, MIN_CHUNK_Y,
};
use crate::persistence::LoadRequest;
use crate::state::GameState;
use fnv::FnvHashSet;
use hecs::Entity;

pub struct ChunkLoadingSystem {
    last_camera_chunk_coord_xz: Option<(i32, i32)>,
    pending_requests: FnvHashSet<ChunkCoord>,
}

impl ChunkLoadingSystem {
    pub fn new() -> Self {
        Self {
            last_camera_chunk_coord_xz: None,
            pending_requests: FnvHashSet::default(),
        }
    }

    pub fn update(&mut self, game_state: &mut GameState) {
        while let Ok((coord, opt_chunk_data)) = game_state.gen_result_rx.try_recv() {
            self.pending_requests.remove(&coord);
            if let Some(chunk_data) = opt_chunk_data {
                if self.is_chunk_within_render_distance(coord, game_state)
                    && !game_state.chunk_entity_map.contains_key(&coord)
                {
                    let new_entity = game_state.world.spawn((coord, chunk_data, ChunkDirty)); // Mut borrow world
                    game_state.chunk_entity_map.insert(coord, new_entity);
                    self.mark_neighbors_dirty(coord, game_state);
                }
            }
        }

        let camera_pos = game_state.camera.position;
        let current_cam_chunk_xz = (
            world_to_chunk_coords(camera_pos.x.floor() as i32, 0, 0).0,
            world_to_chunk_coords(0, 0, camera_pos.z.floor() as i32).2,
        );

        let needs_recalc = match self.last_camera_chunk_coord_xz {
            Some(last_xz) => last_xz != current_cam_chunk_xz,
            None => true,
        };

        if !needs_recalc {
            return;
        }
        self.last_camera_chunk_coord_xz = Some(current_cam_chunk_xz);

        let (cam_cx, cam_cz) = current_cam_chunk_xz;
        let load_dist = game_state.config.load_distance;
        let render_dist = game_state.config.render_distance;
        let load_dist_sq = load_dist * load_dist;
        let render_dist_sq = render_dist * render_dist;

        let mut target_render_chunks = FnvHashSet::<ChunkCoord>::default();

        for dz in -render_dist..=render_dist {
            for dx in -render_dist..=render_dist {
                let dist_sq = dx * dx + dz * dz;
                if dist_sq <= render_dist_sq {
                    let target_cx = cam_cx + dx;
                    let target_cz = cam_cz + dz;
                    for target_cy in MIN_CHUNK_Y..=MAX_CHUNK_Y {
                        target_render_chunks.insert(ChunkCoord(target_cx, target_cy, target_cz));
                    }
                }
            }
        }

        let currently_loaded_coords: FnvHashSet<ChunkCoord> =
            game_state.chunk_entity_map.keys().copied().collect();

        for coord_to_load in target_render_chunks.iter() {
            if !currently_loaded_coords.contains(coord_to_load)
                && !self.pending_requests.contains(coord_to_load)
            {
                let dx = coord_to_load.0 - cam_cx;
                let dz = coord_to_load.2 - cam_cz;
                let request_type = if dx * dx + dz * dz <= load_dist_sq {
                    LoadRequest::LoadOrGenerate(*coord_to_load)
                } else {
                    LoadRequest::LoadFromCache(*coord_to_load)
                };

                if game_state.gen_request_tx.send(request_type).is_ok() {
                    self.pending_requests.insert(*coord_to_load);
                } else {
                    eprintln!("Failed to send chunk load request for {:?}", coord_to_load);
                }
            }
        }

        let mut coords_to_unload = Vec::new();
        for loaded_coord in currently_loaded_coords.iter() {
            let dx = loaded_coord.0 - cam_cx;
            let dz = loaded_coord.2 - cam_cz;
            if dx * dx + dz * dz > render_dist_sq {
                coords_to_unload.push(*loaded_coord);
            }
        }

        struct UnloadInfo {
            entity: Entity,
            coord: ChunkCoord,
            data_to_save: Option<ChunkData>,
            opaque_mesh_id_to_remove: Option<usize>,
            transparent_mesh_id_to_remove: Option<usize>,
        }
        let mut unload_infos = Vec::new();

        for coord in coords_to_unload {
            if let Some(entity) = game_state.chunk_entity_map.get(&coord).copied() {
                if game_state.world.contains(entity) {
                    let mut data_to_save: Option<ChunkData> = None;
                    if game_state.world.get::<&ChunkModified>(entity).is_ok() {
                        if let Ok(data_ref) = game_state.world.get::<&ChunkData>(entity) {
                            data_to_save = Some((*data_ref).clone());
                        } else {
                            eprintln!(
                                "ChunkData missing for modified chunk {:?} during unload check",
                                coord
                            );
                        }
                    }

                    // Get both opaque and transparent mesh IDs for cleanup
                    let (opaque_id, transparent_id) =
                        match game_state.world.get::<&Renderable>(entity) {
                            Ok(r) => (r.opaque_mesh_id, r.transparent_mesh_id),
                            Err(_) => (None, None),
                        };

                    unload_infos.push(UnloadInfo {
                        entity,
                        coord,
                        data_to_save,
                        opaque_mesh_id_to_remove: opaque_id,
                        transparent_mesh_id_to_remove: transparent_id,
                    });
                } else {
                    game_state.chunk_entity_map.remove(&coord);
                }
            }
            self.pending_requests.remove(&coord);
        }

        for info in unload_infos {
            if let Some(data) = info.data_to_save {
                if let Err(e) = game_state.chunk_cache.save_chunk(info.coord, &data) {
                    eprintln!("Failed to save chunk {:?} during unload: {}", info.coord, e);
                }
                if game_state.world.contains(info.entity) {
                    let _ = game_state.world.remove_one::<ChunkModified>(info.entity);
                }
            }

            // Cleanup both opaque and transparent meshes
            if let Some(mesh_id) = info.opaque_mesh_id_to_remove {
                game_state.renderer.cleanup_mesh_buffers(mesh_id);
                game_state.mesh_registry.remove_mesh(mesh_id);
            }
            if let Some(mesh_id) = info.transparent_mesh_id_to_remove {
                game_state.renderer.cleanup_mesh_buffers(mesh_id);
                game_state.mesh_registry.remove_mesh(mesh_id);
            }

            game_state.chunk_entity_map.remove(&info.coord);

            if game_state.world.contains(info.entity) {
                if let Err(e) = game_state.world.despawn(info.entity) {
                    eprintln!(
                        "Error despawning entity {:?} for chunk {:?}: {}",
                        info.entity, info.coord, e
                    );
                }
            }
        }
    }

    fn is_chunk_within_render_distance(&self, coord: ChunkCoord, game_state: &GameState) -> bool {
        if let Some((cam_cx, cam_cz)) = self.last_camera_chunk_coord_xz {
            let render_dist = game_state.config.render_distance;
            let render_dist_sq = render_dist * render_dist;
            let dx = coord.0 - cam_cx;
            let dz = coord.2 - cam_cz;
            dx * dx + dz * dz <= render_dist_sq
        } else {
            false
        }
    }

    fn mark_neighbors_dirty(&self, coord: ChunkCoord, game_state: &mut GameState) {
        let neighbor_offsets = [
            (1, 0, 0),
            (-1, 0, 0),
            (0, 1, 0),
            (0, -1, 0),
            (0, 0, 1),
            (0, 0, -1),
        ];
        for offset in neighbor_offsets {
            let neighbor_coord =
                ChunkCoord(coord.0 + offset.0, coord.1 + offset.1, coord.2 + offset.2);
            if neighbor_coord.1 >= MIN_CHUNK_Y && neighbor_coord.1 <= MAX_CHUNK_Y {
                if let Some(neighbor_entity) = game_state.chunk_entity_map.get(&neighbor_coord) {
                    if game_state.world.contains(*neighbor_entity)
                        && game_state
                            .world
                            .get::<&ChunkDirty>(*neighbor_entity)
                            .is_err()
                    {
                        if let Err(e) = game_state.world.insert_one(*neighbor_entity, ChunkDirty) {
                            eprintln!(
                                "Failed to insert ChunkDirty for neighbor {:?} of {:?}: {}",
                                neighbor_coord, coord, e
                            );
                        }
                    }
                }
            }
        }
    }
}

impl Default for ChunkLoadingSystem {
    fn default() -> Self {
        Self::new()
    }
}
