use crate::components::{
    world_to_chunk_coords, world_to_local_coords, BlockType, ChunkCoord, ChunkData,
};
use crate::resources::{
    Camera, Config, InputState, MeshGenerator, MeshRegistry, Renderer, ShaderProgram,
    TextureManager, WorldBuilder,
};
use fnv::FnvHashMap;
use glam::Vec3;
use hecs::{Entity, World};

pub struct GameState {
    pub config: Config,
    pub world: World,
    pub camera: Camera,
    pub renderer: Renderer,
    pub shader_program: ShaderProgram,
    pub input_state: InputState,
    pub texture_manager: TextureManager,
    pub mesh_registry: MeshRegistry,
    pub mesh_generator: MeshGenerator,
    pub chunk_entity_map: FnvHashMap<ChunkCoord, Entity>,
}

impl GameState {
    pub fn new(gl: crate::gl::Gl, width: u32, height: u32) -> Self {
        let renderer = Renderer::new(gl.clone());
        let shader_program = ShaderProgram::new(&renderer.gl);
        let camera = Camera::new(
            Vec3::new(0.0, 5.0, 15.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            width as f32 / height as f32,
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
        ];
        if let Err(e) = texture_manager.load_textures_and_build_atlas(&texture_files) {
            panic!("Failed to load textures or build atlas: {}", e);
        }

        let mesh_registry = MeshRegistry::new();
        let mesh_generator = MeshGenerator::new();
        let world = World::new();
        let chunk_entity_map = FnvHashMap::default();

        let mut state = Self {
            config: Config::new(),
            world,
            input_state: InputState::new(),
            camera,
            renderer,
            shader_program,
            texture_manager,
            mesh_registry,
            mesh_generator,
            chunk_entity_map,
        };

        let world_builder = WorldBuilder::new();
        world_builder.build_initial_world(&mut state.world, &mut state.chunk_entity_map);

        state
    }

    pub fn handle_resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width as i32, height as i32);
        self.camera.update_aspect_ratio(width as f32, height as f32);
    }

    pub fn get_block_world(&self, world_x: i32, world_y: i32, world_z: i32) -> BlockType {
        let chunk_coord = world_to_chunk_coords(world_x, world_y, world_z);
        if let Some(entity) = self.chunk_entity_map.get(&chunk_coord) {
            if let Ok(chunk_data) = self.world.get::<&ChunkData>(*entity) {
                let (lx, ly, lz) = world_to_local_coords(world_x, world_y, world_z);
                return chunk_data.get_block(lx, ly, lz);
            }
        }
        BlockType::Air
    }
}
