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

    fn register_mesh(&mut self, vertices: Vec<f32>, indices: Vec<u32>) -> usize {
        let mesh = Mesh { vertices, indices };
        self.meshes.push(mesh);
        self.meshes.len() - 1
    }

    pub fn register_block_mesh(&mut self) -> usize {
        let vertices: Vec<f32> = vec![
            // Front face (+Z)
            -0.5, -0.5, 0.5, 0.0, 0.0, // Bottom-left
            0.5, -0.5, 0.5, 1.0, 0.0, // Bottom-right
            0.5, 0.5, 0.5, 1.0, 1.0, // Top-right
            -0.5, 0.5, 0.5, 0.0, 1.0, // Top-left
            // Back face (-Z)
            -0.5, -0.5, -0.5, 1.0, 0.0, // Bottom-right (flipped UV X)
            0.5, -0.5, -0.5, 0.0, 0.0, // Bottom-left (flipped UV X)
            0.5, 0.5, -0.5, 0.0, 1.0, // Top-left (flipped UV X)
            -0.5, 0.5, -0.5, 1.0, 1.0, // Top-right (flipped UV X)
            // Right face (+X)
            0.5, -0.5, 0.5, 0.0, 0.0, // Bottom-back
            0.5, -0.5, -0.5, 1.0, 0.0, // Bottom-front
            0.5, 0.5, -0.5, 1.0, 1.0, // Top-front
            0.5, 0.5, 0.5, 0.0, 1.0, // Top-back
            // Left face (-X)
            -0.5, -0.5, -0.5, 0.0, 0.0, // Bottom-front
            -0.5, -0.5, 0.5, 1.0, 0.0, // Bottom-back
            -0.5, 0.5, 0.5, 1.0, 1.0, // Top-back
            -0.5, 0.5, -0.5, 0.0, 1.0, // Top-front
            // Top face (+Y)
            -0.5, 0.5, 0.5, 0.0, 0.0, // Back-left
            0.5, 0.5, 0.5, 1.0, 0.0, // Back-right
            0.5, 0.5, -0.5, 1.0, 1.0, // Front-right
            -0.5, 0.5, -0.5, 0.0, 1.0, // Front-left
            // Bottom face (-Y)
            -0.5, -0.5, -0.5, 0.0, 0.0, // Front-left
            0.5, -0.5, -0.5, 1.0, 0.0, // Front-right
            0.5, -0.5, 0.5, 1.0, 1.0, // Back-right
            -0.5, -0.5, 0.5, 0.0, 1.0, // Back-left
        ];

        let indices: Vec<u32> = vec![
            0, 1, 2, 2, 3, 0, // Front
            4, 5, 6, 6, 7, 4, // Back
            8, 9, 10, 10, 11, 8, // Right
            12, 13, 14, 14, 15, 12, // Left
            16, 17, 18, 18, 19, 16, // Top
            20, 21, 22, 22, 23, 20, // Bottom
        ];

        self.register_mesh(vertices, indices)
    }
}

impl Default for MeshRegistry {
    fn default() -> Self {
        Self::new()
    }
}
