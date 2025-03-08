pub mod app;
mod gl;
mod shaders;
mod state;
mod window;

mod components {
    mod block;
    mod renderable;
    mod transform;

    pub use block::{Block, BlockType};
    pub use renderable::Renderable;
    pub use transform::Transform;
}

mod resources {
    mod camera;
    mod gl_state;
    mod mesh;

    pub use camera::Camera;
    pub use gl_state::GlState;
    pub use mesh::MeshRegistry;
}

mod systems {
    mod render;

    pub use render::RenderSystem;
}
