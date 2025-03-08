pub struct Mesh {
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,
}

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

    pub fn register_cube_mesh(&mut self) -> usize {
        let vertices: Vec<f32> = vec![
            // Front face
            -0.5, -0.5, 0.5, // 0
            0.5, -0.5, 0.5, // 1
            0.5, 0.5, 0.5, // 2
            -0.5, 0.5, 0.5, // 3
            // Back face
            -0.5, -0.5, -0.5, // 4
            0.5, -0.5, -0.5, // 5
            0.5, 0.5, -0.5, // 6
            -0.5, 0.5, -0.5, // 7
        ];

        let indices: Vec<u32> = vec![
            // Front
            0, 1, 2, 2, 3, 0, // Right
            1, 5, 6, 6, 2, 1, // Back
            5, 4, 7, 7, 6, 5, // Left
            4, 0, 3, 3, 7, 4, // Top
            3, 2, 6, 6, 7, 3, // Bottom
            4, 5, 1, 1, 0, 4,
        ];

        self.register_mesh(vertices, indices)
    }
}

impl Default for MeshRegistry {
    fn default() -> Self {
        Self::new()
    }
}
