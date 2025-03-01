use glam::Vec3;
use hecs::World;

use crate::components::{Block, BlockType, Renderable, Transform};
use crate::resources::{Camera, GlState, MeshRegistry};
use crate::shaders::ShaderProgram;

pub struct GameState {
    pub world: World,
    pub camera: Camera,
    pub mesh_registry: MeshRegistry,
    pub gl_state: GlState,
    pub shader_program: ShaderProgram,
}

impl GameState {
    pub fn new(gl_state: GlState) -> Self {
        let shader_program = ShaderProgram::new(&gl_state.gl);
        let mesh_registry = MeshRegistry::new();

        let camera = Camera::new(
            Vec3::new(0.0, 0.0, 6.0), // Camera position
            Vec3::new(0.0, 0.0, 0.0), // Look at point
            Vec3::new(0.0, 1.0, 0.0), // Up vector
            800.0 / 600.0,            // Aspect ratio
        );

        let world = World::new();

        Self {
            gl_state,
            shader_program,
            world,
            camera,
            mesh_registry,
        }
    }

    pub fn initialize(&mut self) {
        let cube_mesh_id = self.register_cube_mesh();
        self.setup_world(cube_mesh_id);
        self.setup_mesh_buffers();
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        unsafe {
            self.gl_state.gl.Viewport(0, 0, width, height);
        }

        self.camera.update_aspect_ratio(width as f32, height as f32);
    }

    fn setup_world(&mut self, cube_mesh_id: usize) {
        // Create some blocks
        self.spawn_block(Vec3::new(0.0, 0.0, 0.0), BlockType::Dirt, cube_mesh_id);
        self.spawn_block(Vec3::new(1.0, 0.0, 0.0), BlockType::Stone, cube_mesh_id);
        self.spawn_block(Vec3::new(0.0, 1.0, 0.0), BlockType::Grass, cube_mesh_id);
        self.spawn_block(Vec3::new(0.0, 0.0, 1.0), BlockType::Dirt, cube_mesh_id);
        self.spawn_block(Vec3::new(-1.0, 0.0, 0.0), BlockType::Stone, cube_mesh_id);
        self.spawn_block(Vec3::new(-2.0, 0.0, 0.0), BlockType::Stone, cube_mesh_id);
    }

    fn spawn_block(&mut self, position: Vec3, block_type: BlockType, mesh_id: usize) {
        self.world.spawn((
            Transform::new(position, Vec3::ZERO, Vec3::ONE),
            Block::new(block_type),
            Renderable::new(mesh_id),
        ));
    }

    fn register_cube_mesh(&mut self) -> usize {
        let vertices: Vec<f32> = vec![
            // Front face
            -0.5, -0.5, 0.5, // 0
            0.5, -0.5, 0.5, // 1
            0.5, 0.5, 0.5, // 2
            -0.5, 0.5, 0.5, // 3
            // Back face
            -0.5, -0.5, -0.5, // 4
            0.5, -0.5, -0.5, // 5
            0.5, 0.5, -0.5, // 6
            -0.5, 0.5, -0.5, // 7
        ];

        let indices: Vec<u32> = vec![
            // Front
            0, 1, 2, 2, 3, 0, // Right
            1, 5, 6, 6, 2, 1, // Back
            5, 4, 7, 7, 6, 5, // Left
            4, 0, 3, 3, 7, 4, // Top
            3, 2, 6, 6, 7, 3, // Bottom
            4, 5, 1, 1, 0, 4,
        ];

        self.mesh_registry.register_mesh(vertices, indices)
    }

    fn setup_mesh_buffers(&mut self) {
        for (mesh_id, mesh) in self.mesh_registry.meshes.iter().enumerate() {
            self.gl_state
                .setup_mesh_buffers(mesh_id, &mesh.vertices, &mesh.indices);
        }
    }
}

impl Drop for GameState {
    fn drop(&mut self) {
        let mesh_ids: Vec<usize> = self.gl_state.vaos.keys().copied().collect();
        for mesh_id in mesh_ids {
            self.gl_state.cleanup_mesh_buffers(mesh_id);
        }
    }
}
