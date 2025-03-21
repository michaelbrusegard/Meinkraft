use crate::components::{Block, BlockType, Renderable, Transform};
use glam::Vec3;
use hecs::World;

pub struct WorldBuilder {
    mesh_id: usize,
}

impl WorldBuilder {
    pub fn new(mesh_id: usize) -> Self {
        Self { mesh_id }
    }

    pub fn build_initial_world(&self, world: &mut World) {
        self.spawn_block(world, Vec3::new(0.0, 0.0, 0.0), BlockType::Dirt);
        self.spawn_block(world, Vec3::new(1.0, 0.0, 0.0), BlockType::Stone);
        self.spawn_block(world, Vec3::new(0.0, 1.0, 0.0), BlockType::Grass);
        self.spawn_block(world, Vec3::new(0.0, 0.0, 1.0), BlockType::Dirt);
        self.spawn_block(world, Vec3::new(-1.0, 0.0, 0.0), BlockType::Stone);
        self.spawn_block(world, Vec3::new(-2.0, 0.0, 0.0), BlockType::Stone);
    }

    fn spawn_block(&self, world: &mut World, position: Vec3, block_type: BlockType) {
        world.spawn((
            Transform::new(position, Vec3::ZERO, Vec3::ONE),
            Block::new(block_type),
            Renderable::new(self.mesh_id),
        ));
    }
}
