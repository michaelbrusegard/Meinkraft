use crate::components::{Block, BlockType, Renderable, Transform};
use crate::resources::{MeshRegistry, Renderer};
use glam::Vec3;
use hecs::World;

pub struct InitSystem;

impl InitSystem {
    pub fn new() -> Self {
        Self
    }

    pub fn initialize(
        &self,
        world: &mut World,
        mesh_registry: &mut MeshRegistry,
        renderer: &mut Renderer,
    ) {
        let voxel_mesh_id = mesh_registry.register_voxel_mesh();
        self.setup_world(world, voxel_mesh_id);
        self.setup_mesh_buffers(mesh_registry, renderer);
    }

    fn setup_world(&self, world: &mut World, voxel_mesh_id: usize) {
        // Create some blocks
        self.spawn_block(
            world,
            Vec3::new(0.0, 0.0, 0.0),
            BlockType::Dirt,
            voxel_mesh_id,
        );
        self.spawn_block(
            world,
            Vec3::new(1.0, 0.0, 0.0),
            BlockType::Stone,
            voxel_mesh_id,
        );
        self.spawn_block(
            world,
            Vec3::new(0.0, 1.0, 0.0),
            BlockType::Grass,
            voxel_mesh_id,
        );
        self.spawn_block(
            world,
            Vec3::new(0.0, 0.0, 1.0),
            BlockType::Dirt,
            voxel_mesh_id,
        );
        self.spawn_block(
            world,
            Vec3::new(-1.0, 0.0, 0.0),
            BlockType::Stone,
            voxel_mesh_id,
        );
        self.spawn_block(
            world,
            Vec3::new(-2.0, 0.0, 0.0),
            BlockType::Stone,
            voxel_mesh_id,
        );
    }

    fn spawn_block(
        &self,
        world: &mut World,
        position: Vec3,
        block_type: BlockType,
        mesh_id: usize,
    ) {
        world.spawn((
            Transform::new(position, Vec3::ZERO, Vec3::ONE),
            Block::new(block_type),
            Renderable::new(mesh_id),
        ));
    }

    fn setup_mesh_buffers(&self, mesh_registry: &MeshRegistry, renderer: &mut Renderer) {
        for (mesh_id, mesh) in mesh_registry.meshes.iter().enumerate() {
            renderer.setup_mesh_buffers(mesh_id, &mesh.vertices, &mesh.indices);
        }
    }
}

impl Default for InitSystem {
    fn default() -> Self {
        Self::new()
    }
}
