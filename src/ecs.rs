use glam::Vec3;
use hecs::World;

use crate::components::{Block, BlockType, Renderable, Transform};
use crate::resources::{Camera, MeshRegistry};

pub fn setup_world(world: &mut World, mesh_registry: &mut MeshRegistry) -> usize {
    // Register cube mesh
    let cube_mesh_id = register_cube_mesh(mesh_registry);

    // Create some blocks
    spawn_block(
        world,
        Vec3::new(0.0, 0.0, 0.0),
        BlockType::Dirt,
        cube_mesh_id,
    );
    spawn_block(
        world,
        Vec3::new(1.0, 0.0, 0.0),
        BlockType::Stone,
        cube_mesh_id,
    );
    spawn_block(
        world,
        Vec3::new(0.0, 1.0, 0.0),
        BlockType::Grass,
        cube_mesh_id,
    );
    spawn_block(
        world,
        Vec3::new(0.0, 0.0, 1.0),
        BlockType::Dirt,
        cube_mesh_id,
    );
    spawn_block(
        world,
        Vec3::new(-1.0, 0.0, 0.0),
        BlockType::Stone,
        cube_mesh_id,
    );
    spawn_block(
        world,
        Vec3::new(-2.0, 0.0, 0.0),
        BlockType::Stone,
        cube_mesh_id,
    );

    cube_mesh_id
}

fn spawn_block(world: &mut World, position: Vec3, block_type: BlockType, mesh_id: usize) {
    world.spawn((
        Transform::new(position, Vec3::ZERO, Vec3::ONE),
        Block::new(block_type),
        Renderable::new(mesh_id),
    ));
}

fn register_cube_mesh(mesh_registry: &mut MeshRegistry) -> usize {
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

    mesh_registry.register_mesh(vertices, indices)
}
