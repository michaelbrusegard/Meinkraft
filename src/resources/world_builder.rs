use crate::components::{Block, BlockType, Renderable, Transform};
use glam::Vec3;
use hecs::World;
use std::collections::HashMap;

pub struct WorldBuilder<'a> {
    block_mesh_ids: &'a HashMap<BlockType, usize>,
}

impl<'a> WorldBuilder<'a> {
    pub fn new(block_mesh_ids: &'a HashMap<BlockType, usize>) -> Self {
        Self { block_mesh_ids }
    }

    pub fn build_initial_world(&self, world: &mut World) {
        let size = 5;
        for x in -size..=size {
            for z in -size..=size {
                // Ground layer
                self.spawn_block(world, Vec3::new(x as f32, 0.0, z as f32), BlockType::Grass);
                // Dirt layer below
                self.spawn_block(world, Vec3::new(x as f32, -1.0, z as f32), BlockType::Dirt);
                // Stone layer below dirt
                self.spawn_block(world, Vec3::new(x as f32, -2.0, z as f32), BlockType::Stone);
            }
        }

        self.spawn_block(world, Vec3::new(0.0, 1.0, 0.0), BlockType::Stone);
        self.spawn_block(world, Vec3::new(0.0, 2.0, 0.0), BlockType::Stone);
    }

    fn spawn_block(&self, world: &mut World, position: Vec3, block_type: BlockType) {
        let mesh_id = self
            .block_mesh_ids
            .get(&block_type)
            .copied()
            .unwrap_or_else(|| {
                panic!("Mesh ID not found for block type: {:?}", block_type);
            });

        world.spawn((
            Transform::new(position, Vec3::ZERO, Vec3::ONE),
            Block::new(block_type),
            Renderable::new(mesh_id),
        ));
    }
}
