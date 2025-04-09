use crate::components::{
    world_to_chunk_coords, world_to_local_coords, BlockType, ChunkCoord, ChunkData, ChunkDirty,
};
use fnv::{FnvHashMap, FnvHashSet};
use hecs::{Entity, World};

pub struct BlockPosition {
    x: i32,
    y: i32,
    z: i32,
}

impl BlockPosition {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

pub struct WorldBuilder {}

impl WorldBuilder {
    pub fn new() -> Self {
        Self {}
    }

    pub fn build_initial_world(
        &self,
        world: &mut World,
        chunk_entity_map: &mut FnvHashMap<ChunkCoord, Entity>,
    ) {
        let size = 5;
        let mut affected_coords = FnvHashSet::<ChunkCoord>::default();

        for x in -size..=size {
            for z in -size..=size {
                self.set_block(
                    world,
                    chunk_entity_map,
                    BlockPosition::new(x, 0, z),
                    BlockType::Grass,
                    &mut affected_coords,
                );
                self.set_block(
                    world,
                    chunk_entity_map,
                    BlockPosition::new(x, -1, z),
                    BlockType::Dirt,
                    &mut affected_coords,
                );
                self.set_block(
                    world,
                    chunk_entity_map,
                    BlockPosition::new(x, -2, z),
                    BlockType::Stone,
                    &mut affected_coords,
                );
            }
        }

        self.set_block(
            world,
            chunk_entity_map,
            BlockPosition::new(0, 1, 0),
            BlockType::Stone,
            &mut affected_coords,
        );
        self.set_block(
            world,
            chunk_entity_map,
            BlockPosition::new(0, 2, 0),
            BlockType::Stone,
            &mut affected_coords,
        );
        self.set_block(
            world,
            chunk_entity_map,
            BlockPosition::new(0, 1, 1),
            BlockType::Log,
            &mut affected_coords,
        );
        self.set_block(
            world,
            chunk_entity_map,
            BlockPosition::new(0, 1, 2),
            BlockType::Planks,
            &mut affected_coords,
        );
        self.set_block(
            world,
            chunk_entity_map,
            BlockPosition::new(0, 1, 3),
            BlockType::Glass,
            &mut affected_coords,
        );

        for coord in affected_coords {
            if let Some(entity) = chunk_entity_map.get(&coord) {
                world.insert(*entity, (ChunkDirty,)).unwrap_or_else(|e| {
                    eprintln!("Failed to insert ChunkDirty for {:?}: {}", coord, e)
                });
            }
        }
    }

    fn set_block(
        &self,
        world: &mut World,
        chunk_entity_map: &mut FnvHashMap<ChunkCoord, Entity>,
        position: BlockPosition,
        block_type: BlockType,
        affected_coords: &mut FnvHashSet<ChunkCoord>,
    ) {
        let chunk_coord = world_to_chunk_coords(position.x, position.y, position.z);
        let (lx, ly, lz) = world_to_local_coords(position.x, position.y, position.z);

        let entity = *chunk_entity_map
            .entry(chunk_coord)
            .or_insert_with(|| world.spawn((chunk_coord, ChunkData::new())));

        match world.query_one_mut::<&mut ChunkData>(entity) {
            Ok(chunk_data) => {
                chunk_data.set_block(lx, ly, lz, block_type);
                affected_coords.insert(chunk_coord);
            }
            Err(e) => {
                eprintln!(
                "Error: Failed to get mutable ChunkData via query_one_mut for entity {:?} at coord {:?}: {:?}",
                entity, chunk_coord, e
            );
                chunk_entity_map.remove(&chunk_coord);
            }
        }
    }
}

impl Default for WorldBuilder {
    fn default() -> Self {
        Self::new()
    }
}
