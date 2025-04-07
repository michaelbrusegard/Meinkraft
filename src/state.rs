use crate::components::BlockType;
use crate::resources::{
    Camera, Config, InputState, MeshRegistry, Renderer, ShaderProgram, TextureManager, WorldBuilder,
};
use enum_iterator::all;
use glam::Vec3;
use hecs::World;
use std::collections::HashMap;

pub struct GameState {
    pub config: Config,
    pub world: World,
    pub camera: Camera,
    pub renderer: Renderer,
    pub shader_program: ShaderProgram,
    pub input_state: InputState,
    pub texture_manager: TextureManager,
}

impl GameState {
    pub fn new(gl: crate::gl::Gl, width: u32, height: u32) -> Self {
        let mut renderer = Renderer::new(gl.clone());
        let shader_program = ShaderProgram::new(&renderer.gl);
        let camera = Camera::new(
            Vec3::new(0.0, 10.0, 15.0),
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

        let mut mesh_registry = MeshRegistry::new();
        let mut block_mesh_ids = HashMap::new();

        for block_type in all::<BlockType>() {
            let face_textures = block_type.get_face_textures();
            match mesh_registry.register_block_mesh(&texture_manager, face_textures) {
                Ok(mesh_id) => {
                    let mesh = mesh_registry.meshes.get(mesh_id).unwrap();
                    renderer.upload_mesh_buffers(mesh_id, &mesh.vertices, &mesh.indices);
                    block_mesh_ids.insert(block_type, mesh_id);
                }
                Err(e) => {
                    panic!(
                        "Failed to register mesh for block type {:?}: {}",
                        block_type, e
                    );
                }
            }
        }

        let mut state = Self {
            config: Config::new(),
            world: World::new(),
            input_state: InputState::new(),
            camera,
            renderer,
            shader_program,
            texture_manager,
        };

        let world_builder = WorldBuilder::new(&block_mesh_ids);
        world_builder.build_initial_world(&mut state.world);

        state
    }

    pub fn handle_resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width as i32, height as i32);
        self.camera.update_aspect_ratio(width as f32, height as f32);
    }
}
