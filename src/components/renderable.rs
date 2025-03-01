pub struct Renderable {
    pub mesh_id: usize,
}

impl Renderable {
    pub fn new(mesh_id: usize) -> Self {
        Self { mesh_id }
    }
}
