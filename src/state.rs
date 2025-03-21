use crate::resources::{Camera, Config, InputState, MeshRegistry, Renderer, ShaderProgram};
use glam::Vec3;
use hecs::World;

pub struct GameState {
    pub config: Config,
    pub world: World,
    pub camera: Camera,
    pub renderer: Renderer,
    pub shader_program: ShaderProgram,
    pub mesh_registry: MeshRegistry,
    pub input_state: InputState,
}

impl GameState {
    pub fn new(gl: crate::gl::Gl, width: u32, height: u32) -> Self {
        let renderer = Renderer::new(gl);
        let shader_program = ShaderProgram::new(&renderer.gl);
        let camera = Camera::new(
            Vec3::new(0.0, 0.0, 6.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            width as f32 / height as f32,
        );

        Self {
            config: Config::new(),
            world: World::new(),
            mesh_registry: MeshRegistry::new(),
            input_state: InputState::new(),
            camera,
            renderer,
            shader_program,
        }
    }

    pub fn handle_resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width as i32, height as i32);
        self.camera.update_aspect_ratio(width as f32, height as f32);
    }
}
