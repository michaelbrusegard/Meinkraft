pub struct MeshRegistry {
    pub meshes: Vec<Mesh>,
}

impl MeshRegistry {
    pub fn new() -> Self {
        Self { meshes: Vec::new() }
    }

    pub fn register_mesh(&mut self, vertices: Vec<f32>, indices: Vec<u32>) -> usize {
        let mesh = Mesh { vertices, indices };
        self.meshes.push(mesh);
        self.meshes.len() - 1
    }
}

impl Default for MeshRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Mesh {
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,
}
