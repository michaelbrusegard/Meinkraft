pub mod app;
mod gl;
mod window_manager;

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
    mod mesh;
    mod renderer;
    mod shader_program;

    pub use camera::Camera;
    pub use mesh::{Mesh, MeshRegistry};
    pub use renderer::Renderer;
    pub use shader_program::ShaderProgram;
}

pub mod systems {
    mod init_system;
    mod render_system;

    pub use init_system::InitSystem;
    pub use render_system::RenderSystem;
}
