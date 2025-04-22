use crate::components::{
    world_to_chunk_coords, world_to_local_coords, BlockType, ChunkCoord, ChunkData, ChunkModified,
    LOD,
};
use crate::persistence::{
    ChunkCache, LoadRequest, LoadResult, NeighborData, WorkerChannels, WorkerPool, WorkerResources,
};
use crate::resources::{
    Camera, ChunkMeshData, Config, InputState, MeshGenerator, MeshRegistry, Renderer,
    ShaderProgram, TextureManager, WorldGenerator,
};
use crossbeam_channel::{Receiver, Sender};
use fnv::FnvHashMap;
use glam::Vec3;
use hecs::{Entity, World};
use std::sync::Arc;

pub type MeshRequestData = (Entity, ChunkCoord, ChunkData, NeighborData, LOD);
pub type MeshResultData = (Entity, ChunkCoord, Option<ChunkMeshData>, LOD);

pub struct GameState {
    pub config: Config,
    pub world: World,
    pub camera: Camera,
    pub renderer: Renderer,
    pub shader_program: ShaderProgram,
    pub star_shader_program: ShaderProgram,
    pub shadow_shader_program: ShaderProgram,
    pub input_state: InputState,
    pub texture_manager: Arc<TextureManager>,
    pub mesh_registry: MeshRegistry,
    pub mesh_generator: Arc<MeshGenerator>,
    pub chunk_entity_map: FnvHashMap<ChunkCoord, Entity>,
    pub world_generator: Arc<WorldGenerator>,
    pub chunk_cache: ChunkCache,
    pub gen_request_tx: Sender<LoadRequest>,
    pub gen_result_rx: Receiver<LoadResult>,
    pub mesh_request_tx: Sender<MeshRequestData>,
    pub mesh_result_rx: Receiver<MeshResultData>,
    gen_request_rx_worker: Option<Receiver<LoadRequest>>,
    gen_result_tx_worker: Option<Sender<LoadResult>>,
    mesh_request_rx_worker: Option<Receiver<MeshRequestData>>,
    mesh_result_tx_worker: Option<Sender<MeshResultData>>,
    worker_pool: Option<WorkerPool>,
    pub time_of_day: f32,
    pub total_time: f32,
    pub light_space_matrix: glam::Mat4,
}

impl GameState {
    pub fn new(gl: crate::gl::Gl, width: u32, height: u32) -> Self {
        let config = Config::new();
        let renderer = Renderer::new(gl.clone(), &config);
        let mut shader_program = ShaderProgram::from_sources(
            &renderer.gl,
            include_str!("./shaders/vertex.glsl"),
            include_str!("./shaders/fragment.glsl"),
        )
        .expect("Failed to create main shader program");

        shader_program.register_uniform("modelMatrix");
        shader_program.register_uniform("viewMatrix");
        shader_program.register_uniform("projectionMatrix");
        shader_program.register_uniform("blockTexture");
        shader_program.register_uniform("lightDirection");
        shader_program.register_uniform("ambientColor");
        shader_program.register_uniform("lightColor");
        shader_program.register_uniform("minAmbientContribution");
        shader_program.register_uniform("isCelestial");
        shader_program.register_uniform("celestialLayerIndex");
        shader_program.register_uniform("cameraPosition");
        shader_program.register_uniform("shininess");
        shader_program.register_uniform("lightSpaceMatrix");
        shader_program.register_uniform("shadowMap");

        let mut star_shader_program = ShaderProgram::from_sources(
            &renderer.gl,
            include_str!("./shaders/stars_vertex.glsl"),
            include_str!("./shaders/stars_fragment.glsl"),
        )
        .expect("Failed to create star shader program");

        star_shader_program.register_uniform("viewMatrix");
        star_shader_program.register_uniform("projectionMatrix");
        star_shader_program.register_uniform("starDistance");
        star_shader_program.register_uniform("time");
        star_shader_program.register_uniform("nightFactor");

        let mut shadow_shader_program = ShaderProgram::from_sources(
            &renderer.gl,
            include_str!("./shaders/shadow_vertex.glsl"),
            include_str!("./shaders/shadow_fragment.glsl"),
        )
        .expect("Failed to create shadow shader program");

        shadow_shader_program.register_uniform("lightSpaceMatrix");
        shadow_shader_program.register_uniform("modelMatrix");

        let camera = Camera::new(
            Vec3::new(0.0, 20.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::Y,
            width as f32 / height as f32,
            &config,
        );

        let mut texture_manager = TextureManager::new(renderer.gl.clone());
        let texture_files = [
            ("dirt", "assets/textures/dirt.png"),
            ("grassy_dirt_side", "assets/textures/grassy_dirt_side.png"),
            ("grassy_dirt_top", "assets/textures/grassy_dirt_top.png"),
            ("stone", "assets/textures/stone.png"),
            ("sand", "assets/textures/sand.png"),
            ("glass", "assets/textures/glass.png"),
            ("planks", "assets/textures/planks.png"),
            ("log", "assets/textures/log.png"),
            ("log_top", "assets/textures/log_top.png"),
            ("leaves", "assets/textures/leaves.png"),
            ("water", "assets/textures/water.png"),
            ("snow", "assets/textures/snow.png"),
            ("snowy_dirt_side", "assets/textures/snowy_dirt_side.png"),
            ("ice", "assets/textures/ice.png"),
            ("gravel", "assets/textures/gravel.png"),
            ("andesite", "assets/textures/andesite.png"),
            ("granite", "assets/textures/granite.png"),
            ("diorite", "assets/textures/diorite.png"),
            ("cobblestone", "assets/textures/cobblestone.png"),
            ("sun", "assets/textures/sun.png"),
            ("moon", "assets/textures/moon.png"),
        ];
        if let Err(e) = texture_manager.load_textures_as_array(&texture_files) {
            panic!("Failed to load textures into array: {}", e);
        }
        #[allow(clippy::arc_with_non_send_sync)]
        let texture_manager = Arc::new(texture_manager);

        let mesh_registry = MeshRegistry::new();
        let mesh_generator = Arc::new(MeshGenerator::new());
        let world_generator = Arc::new(WorldGenerator::new(config.clone()));
        let world = World::new();
        let chunk_entity_map = FnvHashMap::default();

        let (gen_request_tx, gen_request_rx_worker) = crossbeam_channel::unbounded::<LoadRequest>();
        let (gen_result_tx_worker, gen_result_rx) = crossbeam_channel::unbounded::<LoadResult>();
        let (mesh_request_tx, mesh_request_rx_worker) =
            crossbeam_channel::unbounded::<MeshRequestData>();
        let (mesh_result_tx_worker, mesh_result_rx) =
            crossbeam_channel::unbounded::<MeshResultData>();

        let chunk_cache = ChunkCache::new("world").expect("Failed to initialize chunk cache");

        Self {
            config,
            world,
            input_state: InputState::new(),
            camera,
            renderer,
            shader_program,
            star_shader_program,
            shadow_shader_program,
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
            time_of_day: 0.5,
            total_time: 0.0,
            light_space_matrix: glam::Mat4::IDENTITY,
        }
    }

    pub fn initialize_workers(&mut self) {
        if self.worker_pool.is_some() {
            println!("Workers already initialized.");
            return;
        }
        let resources = WorkerResources {
            world_generator: Arc::clone(&self.world_generator),
            mesh_generator: Arc::clone(&self.mesh_generator),
            texture_manager_layers: Arc::new(self.texture_manager.get_all_layers()),
            chunk_cache: self.chunk_cache.clone(),
            config: self.config.clone(),
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
    }

    pub fn shutdown_workers(&mut self) {
        if let Some(pool) = self.worker_pool.take() {
            pool.shutdown();
        } else {
            println!("Worker pool was not initialized or already shut down.");
        }
        self.save_modified_chunks();
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
        let chunk_coord = world_to_chunk_coords(&self.config, world_x, world_y, world_z);
        if let Some(entity) = self.chunk_entity_map.get(&chunk_coord) {
            if let Ok(data_ref) = self.world.get::<&ChunkData>(*entity) {
                let (lx, ly, lz) = world_to_local_coords(&self.config, world_x, world_y, world_z);
                return data_ref.get_block(&self.config, lx, ly, lz);
            }
        }
        BlockType::Air
    }
}
