use crate::components::{
    world_to_chunk_coords, world_to_local_coords, BlockType, ChunkCoord, ChunkData, ChunkModified,
};
use crate::persistence::{
    ChunkCache, LoadRequest, LoadResult, NeighborData, WorkerChannels, WorkerPool, WorkerResources,
};
use crate::resources::{
    Camera, Config, InputState, Mesh, MeshGenerator, MeshRegistry, Renderer, ShaderProgram,
    TextureManager, WorldGenerator,
};
use crossbeam_channel::{Receiver, Sender};
use fnv::FnvHashMap;
use glam::Vec3;
use hecs::{Entity, World};
use std::sync::Arc;

pub struct GameState {
    pub config: Config,
    pub world: World,
    pub camera: Camera,
    pub renderer: Renderer,
    pub shader_program: ShaderProgram,
    pub input_state: InputState,
    pub texture_manager: Arc<TextureManager>,
    pub mesh_registry: MeshRegistry,
    pub mesh_generator: Arc<MeshGenerator>,
    pub chunk_entity_map: FnvHashMap<ChunkCoord, Entity>,
    pub world_generator: Arc<WorldGenerator>,
    pub chunk_cache: ChunkCache,
    pub gen_request_tx: Sender<LoadRequest>,
    pub gen_result_rx: Receiver<LoadResult>,
    pub mesh_request_tx: Sender<(Entity, ChunkCoord, ChunkData, NeighborData)>,
    pub mesh_result_rx: Receiver<(Entity, ChunkCoord, Option<Mesh>)>,
    gen_request_rx_worker: Option<Receiver<LoadRequest>>,
    gen_result_tx_worker: Option<Sender<LoadResult>>,
    mesh_request_rx_worker: Option<Receiver<(Entity, ChunkCoord, ChunkData, NeighborData)>>,
    mesh_result_tx_worker: Option<Sender<(Entity, ChunkCoord, Option<Mesh>)>>,
    worker_pool: Option<WorkerPool>,
}

impl GameState {
    pub fn new(gl: crate::gl::Gl, width: u32, height: u32) -> Self {
        let config = Config::new();
        let renderer = Renderer::new(gl.clone());
        let shader_program = ShaderProgram::new(&renderer.gl);
        let camera = Camera::new(
            Vec3::new(0.0, 20.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::Y,
            width as f32 / height as f32,
            config.render_distance,
        );

        let mut texture_manager = TextureManager::new(renderer.gl.clone());
        let texture_files = [
            ("dirt", "assets/textures/dirt.png"),
            ("stone", "assets/textures/stone.png"),
            ("grass_side", "assets/textures/grass_side.png"),
            ("grass_top", "assets/textures/grass_top.png"),
            ("sand", "assets/textures/sand.png"),
            ("glass", "assets/textures/glass.png"),
            ("planks", "assets/textures/planks.png"),
            ("log", "assets/textures/log.png"),
            ("log_top", "assets/textures/log_top.png"),
            ("water", "assets/textures/water.png"),
            ("snow", "assets/textures/snow.png"),
            ("ice", "assets/textures/ice.png"),
            ("gravel", "assets/textures/gravel.png"),
            ("andesite", "assets/textures/andesite.png"),
            ("granite", "assets/textures/granite.png"),
            ("diorite", "assets/textures/diorite.png"),
            ("leaves", "assets/textures/leaves.png"),
        ];
        if let Err(e) = texture_manager.load_textures_and_build_atlas(&texture_files) {
            panic!("Failed to load textures or build atlas: {}", e);
        }
        #[allow(clippy::arc_with_non_send_sync)]
        let texture_manager = Arc::new(texture_manager);

        let mesh_registry = MeshRegistry::new();
        let mesh_generator = Arc::new(MeshGenerator::new());
        let world_generator = Arc::new(WorldGenerator::new(&config));
        let world = World::new();
        let chunk_entity_map = FnvHashMap::default();

        let (gen_request_tx, gen_request_rx_worker) = crossbeam_channel::unbounded::<LoadRequest>();
        let (gen_result_tx_worker, gen_result_rx) = crossbeam_channel::unbounded::<LoadResult>();
        let (mesh_request_tx, mesh_request_rx_worker) =
            crossbeam_channel::unbounded::<(Entity, ChunkCoord, ChunkData, NeighborData)>();
        let (mesh_result_tx_worker, mesh_result_rx) =
            crossbeam_channel::unbounded::<(Entity, ChunkCoord, Option<Mesh>)>();

        let chunk_cache =
            ChunkCache::new("default_world").expect("Failed to initialize chunk cache");

        Self {
            config,
            world,
            input_state: InputState::new(),
            camera,
            renderer,
            shader_program,
            texture_manager,
            mesh_registry,
            mesh_generator,
            chunk_entity_map,
            world_generator,
            chunk_cache,
            gen_request_tx,
            gen_result_rx,
            mesh_request_tx,
            mesh_result_rx,
            gen_request_rx_worker: Some(gen_request_rx_worker),
            gen_result_tx_worker: Some(gen_result_tx_worker),
            mesh_request_rx_worker: Some(mesh_request_rx_worker),
            mesh_result_tx_worker: Some(mesh_result_tx_worker),
            worker_pool: None,
        }
    }

    pub fn initialize_workers(&mut self) {
        if self.worker_pool.is_some() {
            println!("Workers already initialized.");
            return;
        }
        println!("Initializing worker pool...");

        let resources = WorkerResources {
            world_generator: Arc::clone(&self.world_generator),
            mesh_generator: Arc::clone(&self.mesh_generator),
            texture_manager_uvs: Arc::new(self.texture_manager.get_all_uvs()),
            chunk_cache: self.chunk_cache.clone(),
        };

        let channels = WorkerChannels {
            gen_request_rx: self
                .gen_request_rx_worker
                .take()
                .expect("WorkerPool Init: Gen Request Rx channel missing"),
            mesh_request_rx: self
                .mesh_request_rx_worker
                .take()
                .expect("WorkerPool Init: Mesh Request Rx channel missing"),
            gen_result_tx: self
                .gen_result_tx_worker
                .take()
                .expect("WorkerPool Init: Gen Result Tx channel missing"),
            mesh_result_tx: self
                .mesh_result_tx_worker
                .take()
                .expect("WorkerPool Init: Mesh Result Tx channel missing"),
        };

        self.worker_pool = Some(WorkerPool::new(resources, channels));
        println!("Worker pool initialized.");
    }

    pub fn shutdown_workers(&mut self) {
        println!("Shutting down worker pool...");
        if let Some(pool) = self.worker_pool.take() {
            pool.shutdown();
        } else {
            println!("Worker pool was not initialized or already shut down.");
        }
        println!("Saving modified chunks on shutdown...");
        self.save_modified_chunks();
        println!("Chunk saving complete.");
    }

    fn save_modified_chunks(&mut self) {
        let modified_entities: Vec<(Entity, ChunkCoord)> = self
            .world
            .query::<(&ChunkCoord, &ChunkModified)>()
            .iter()
            .map(|(e, (c, _))| (e, *c))
            .collect();

        if modified_entities.is_empty() {
            return;
        }

        let mut saved_count = 0;
        let mut failed_count = 0;
        let mut entities_to_remove_tag: Vec<Entity> = Vec::new();

        for (entity, coord) in &modified_entities {
            if !self.world.contains(*entity) {
                continue;
            }

            let chunk_data_clone = match self.world.get::<&ChunkData>(*entity) {
                Ok(chunk_data_ref) => Some((*chunk_data_ref).clone()),
                Err(_) => {
                    eprintln!(
                        "ChunkData missing for modified chunk entity {:?} at {:?} during save",
                        entity, coord
                    );
                    None
                }
            };

            if let Some(data_to_save) = chunk_data_clone {
                match self.chunk_cache.save_chunk(*coord, &data_to_save) {
                    Ok(()) => {
                        entities_to_remove_tag.push(*entity);
                        saved_count += 1;
                    }
                    Err(e) => {
                        eprintln!("Failed to save modified chunk {:?}: {}", coord, e);
                        failed_count += 1;
                    }
                }
            } else {
                failed_count += 1;
            }
        }

        for entity in entities_to_remove_tag {
            if self.world.contains(entity) {
                if let Err(e) = self.world.remove_one::<ChunkModified>(entity) {
                    eprintln!(
                        "Failed to remove ChunkModified tag for {:?} (may already be gone): {}",
                        entity, e
                    );
                }
            }
        }

        if saved_count > 0 || failed_count > 0 {
            println!(
                "Saved {} modified chunks, failed to save {}.",
                saved_count, failed_count
            );
        }
    }

    pub fn handle_resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width as i32, height as i32);
        self.camera.update_aspect_ratio(width as f32, height as f32);
    }

    pub fn get_block_world(&self, world_x: i32, world_y: i32, world_z: i32) -> BlockType {
        let chunk_coord = world_to_chunk_coords(world_x, world_y, world_z);
        if let Some(entity) = self.chunk_entity_map.get(&chunk_coord) {
            if let Ok(data_ref) = self.world.get::<&ChunkData>(*entity) {
                let (lx, ly, lz) = world_to_local_coords(world_x, world_y, world_z);
                return data_ref.get_block(lx, ly, lz);
            }
        }
        BlockType::Air
    }
}
