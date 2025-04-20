#[derive(Default)]
pub struct Renderable {
    pub opaque_mesh_id: Option<usize>,
    pub transparent_mesh_id: Option<usize>,
}

impl Renderable {
    pub fn new(opaque_mesh_id: Option<usize>, transparent_mesh_id: Option<usize>) -> Self {
        Self {
            opaque_mesh_id,
            transparent_mesh_id,
        }
    }
}
