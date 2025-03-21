use crate::resources::{Camera, InputState, MeshRegistry, Renderer, ShaderProgram};
use crate::systems::InitSystem;
use glam::Vec3;
use hecs::World;

pub struct GameState {
    pub world: World,
    pub camera: Camera,
    pub renderer: Renderer,
    pub shader_program: ShaderProgram,
    pub input_state: InputState,
}

impl GameState {
    pub fn new(gl: crate::gl::Gl, width: u32, height: u32) -> Self {
        let mut renderer = Renderer::new(gl);
        let shader_program = ShaderProgram::new(&renderer.gl);

        let camera = Camera::new(
            Vec3::new(0.0, 0.0, 6.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            width as f32 / height as f32,
        );

        let mut world = World::new();
        let mut mesh_registry = MeshRegistry::new();

        let init_system = InitSystem::new();
        init_system.initialize(&mut world, &mut mesh_registry, &mut renderer);

        Self {
            world,
            camera,
            renderer,
            shader_program,
            input_state: InputState::new(),
        }
    }

    pub fn handle_resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width as i32, height as i32);
        self.camera.update_aspect_ratio(width as f32, height as f32);
    }
}
