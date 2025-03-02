pub mod app;
pub mod gl;
pub mod shaders;
pub mod state;
pub mod window;

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
    mod gl_state;
    mod mesh;

    pub use camera::Camera;
    pub use gl_state::GlState;
    pub use mesh::{Mesh, MeshRegistry};
}

pub mod systems {
    mod render;

    pub use render::render_system;
}
