pub mod app;
mod gl;
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
    mod mesh;
    mod renderer;
    mod shader;

    pub use camera::Camera;
    pub use mesh::{Mesh, MeshRegistry};
    pub use renderer::Renderer;
    pub use shader::ShaderProgram;
}

pub mod systems {
    mod init;
    mod render;

    pub use init::InitSystem;
    pub use render::RenderSystem;
}
