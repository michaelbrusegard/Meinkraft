pub mod app;
mod gl;
mod input;
mod persistence;
mod scheduler;
mod state;
mod window;

pub mod components {
    mod block;
    mod chunk;
    mod renderable;
    mod transform;

    pub use block::BlockType;
    pub use chunk::{
        chunk_coord_to_world_pos, world_to_chunk_coords, world_to_local_coords, ChunkCoord,
        ChunkData, ChunkDirty, ChunkModified, CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_SIZE, CHUNK_WIDTH,
        MAX_CHUNK_Y, MIN_CHUNK_Y,
    };
    pub use renderable::Renderable;
    pub use transform::Transform;
}

pub mod resources {
    mod camera;
    mod config;
    mod input_state;
    mod mesh;
    mod mesh_generator;
    mod renderer;
    mod shader_program;
    mod texture_manager;
    mod world_generator;

    pub use camera::Camera;
    pub use config::{Config, GameAction};
    pub use input_state::InputState;
    pub use mesh::{Mesh, MeshRegistry};
    pub use mesh_generator::MeshGenerator;
    pub use renderer::Renderer;
    pub use shader_program::ShaderProgram;
    pub use texture_manager::{TextureManager, TextureUVs};
    pub use world_generator::WorldGenerator;
}

pub mod systems {
    mod chunk_loading_system;
    mod chunk_meshing_system;
    mod input_system;
    mod render_system;

    pub use chunk_loading_system::ChunkLoadingSystem;
    pub use chunk_meshing_system::ChunkMeshingSystem;
    pub use input_system::InputSystem;
    pub use render_system::RenderSystem;
}
