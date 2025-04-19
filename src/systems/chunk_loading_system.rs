use crate::components::{
    world_to_chunk_coords, ChunkCoord, ChunkDirty, Renderable, MAX_CHUNK_Y, MIN_CHUNK_Y,
};
use crate::state::GameState;
use fnv::FnvHashSet;

pub struct ChunkLoadingSystem {
    last_camera_chunk_coord_xz: Option<(i32, i32)>,
    pending_chunks: FnvHashSet<ChunkCoord>,
}

impl ChunkLoadingSystem {
    pub fn new() -> Self {
        Self {
            last_camera_chunk_coord_xz: None,
            pending_chunks: FnvHashSet::default(),
        }
    }

    pub fn update(&mut self, game_state: &mut GameState) {
        while let Ok((coord, chunk_data)) = game_state.gen_result_rx.try_recv() {
            self.pending_chunks.remove(&coord);
            if self.is_chunk_in_range(coord, game_state) {
                let new_entity = game_state.world.spawn((coord, chunk_data, ChunkDirty));
                game_state.chunk_entity_map.insert(coord, new_entity);
                self.mark_neighbors_dirty(coord, game_state);
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

        let render_dist = game_state.config.render_distance;
        let render_dist_sq = render_dist * render_dist;

        let mut target_chunks = FnvHashSet::<ChunkCoord>::default();
        let (cam_cx, cam_cz) = current_cam_chunk_xz;

        for dz in -render_dist..=render_dist {
            for dx in -render_dist..=render_dist {
                let dist_sq_xz = dx * dx + dz * dz;

                if dist_sq_xz <= render_dist_sq {
                    let target_cx = cam_cx + dx;
                    let target_cz = cam_cz + dz;

                    for target_cy in MIN_CHUNK_Y..=MAX_CHUNK_Y {
                        target_chunks.insert(ChunkCoord(target_cx, target_cy, target_cz));
                    }
                }
            }
        }

        let loaded_chunk_coords: FnvHashSet<ChunkCoord> =
            game_state.chunk_entity_map.keys().copied().collect();

        let chunks_to_load: Vec<ChunkCoord> = target_chunks
            .difference(&loaded_chunk_coords)
            .filter(|coord| !self.pending_chunks.contains(coord))
            .copied()
            .collect();

        let chunks_to_unload: Vec<ChunkCoord> = loaded_chunk_coords
            .iter()
            .filter(|loaded_coord| {
                let dx = loaded_coord.0 - cam_cx;
                let dz = loaded_coord.2 - cam_cz;
                let dist_sq_xz = dx * dx + dz * dz;
                dist_sq_xz > render_dist_sq
            })
            .copied()
            .collect();

        for coord_to_unload in chunks_to_unload {
            if let Some(entity) = game_state.chunk_entity_map.remove(&coord_to_unload) {
                if game_state.world.contains(entity) {
                    if let Ok(renderable) = game_state.world.get::<&Renderable>(entity) {
                        let mesh_id = renderable.mesh_id;
                        game_state.renderer.cleanup_mesh_buffers(mesh_id);
                        game_state.mesh_registry.remove_mesh(mesh_id);
                    }
                    if let Err(e) = game_state.world.despawn(entity) {
                        eprintln!(
                            "Error despawning entity {:?} for chunk {:?}: {}",
                            entity, coord_to_unload, e
                        );
                    }
                }
            }
            self.pending_chunks.remove(&coord_to_unload);
        }

        for coord_to_load in chunks_to_load {
            if game_state.chunk_entity_map.contains_key(&coord_to_load)
                || self.pending_chunks.contains(&coord_to_load)
            {
                continue;
            }

            if game_state.gen_request_tx.send(coord_to_load).is_ok() {
                self.pending_chunks.insert(coord_to_load);
            } else {
                eprintln!(
                    "Failed to send chunk generation request for {:?}",
                    coord_to_load
                );
            }
        }
    }

    fn is_chunk_in_range(&self, coord: ChunkCoord, game_state: &GameState) -> bool {
        if let Some((cam_cx, cam_cz)) = self.last_camera_chunk_coord_xz {
            let render_dist = game_state.config.render_distance;
            let render_dist_sq = render_dist * render_dist;

            let dx = coord.0 - cam_cx;
            let dz = coord.2 - cam_cz;

            let dist_sq_xz = dx * dx + dz * dz;

            dist_sq_xz <= render_dist_sq
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
                        let _ = game_state.world.insert_one(*neighbor_entity, ChunkDirty);
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
