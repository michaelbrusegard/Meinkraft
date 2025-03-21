pub mod app;
mod gl;
mod input;
mod scheduler;
mod state;
mod window;

pub mod components {
    mod block;
    mod renderable;
    mod transform;

    pub use block::{Block, BlockType};
    pub use renderable::Renderable;
    pub use transform::Transform;
}

pub mod resources {
    mod camera;
    mod config;
    mod input_state;
    mod mesh;
    mod renderer;
    mod shader_program;
    mod world_builder;

    pub use camera::Camera;
    pub use config::{Config, GameAction};
    pub use input_state::InputState;
    pub use mesh::{Mesh, MeshRegistry};
    pub use renderer::Renderer;
    pub use shader_program::ShaderProgram;
    pub use world_builder::WorldBuilder;
}

pub mod systems {
    mod input_system;
    mod render_system;

    pub use input_system::InputSystem;
    pub use render_system::RenderSystem;
}
