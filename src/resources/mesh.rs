use crate::resources::TextureManager;

pub struct Mesh {
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,
}

pub struct MeshRegistry {
    pub meshes: Vec<Mesh>,
    next_mesh_id: usize,
}

impl MeshRegistry {
    pub fn new() -> Self {
        Self {
            meshes: Vec::new(),
            next_mesh_id: 0,
        }
    }

    fn register_mesh(&mut self, vertices: Vec<f32>, indices: Vec<u32>) -> usize {
        let mesh_id = self.next_mesh_id;
        self.next_mesh_id += 1;
        if mesh_id >= self.meshes.len() {
            self.meshes.resize_with(mesh_id + 1, || Mesh {
                vertices: Vec::new(),
                indices: Vec::new(),
            });
        }
        self.meshes[mesh_id] = Mesh { vertices, indices };
        mesh_id
    }

    pub fn register_block_mesh(
        &mut self,
        texture_manager: &TextureManager,
        face_textures: [&str; 6],
    ) -> Result<usize, String> {
        let mut vertices = Vec::with_capacity(6 * 4 * 5);
        let mut uvs = [[0.0f32; 4]; 6];

        // Fetch UVs for all faces first
        for i in 0..6 {
            uvs[i] = texture_manager
                .get_uvs(face_textures[i])
                .ok_or_else(|| format!("UVs not found for texture '{}'", face_textures[i]))?;
        }

        let pos = [
            // Front face (+Z)
            [-0.5, -0.5, 0.5],
            [-0.5, 0.5, 0.5],
            [0.5, 0.5, 0.5],
            [0.5, -0.5, 0.5],
            // Back face (-Z)
            [0.5, -0.5, -0.5],
            [0.5, 0.5, -0.5],
            [-0.5, 0.5, -0.5],
            [-0.5, -0.5, -0.5],
            // Right face (+X)
            [0.5, -0.5, 0.5],
            [0.5, 0.5, 0.5],
            [0.5, 0.5, -0.5],
            [0.5, -0.5, -0.5],
            // Left face (-X)
            [-0.5, -0.5, -0.5],
            [-0.5, 0.5, -0.5],
            [-0.5, 0.5, 0.5],
            [-0.5, -0.5, 0.5],
            // Top face (+Y)
            [-0.5, 0.5, 0.5],
            [-0.5, 0.5, -0.5],
            [0.5, 0.5, -0.5],
            [0.5, 0.5, 0.5],
            // Bottom face (-Y)
            [-0.5, -0.5, -0.5],
            [0.5, -0.5, -0.5],
            [0.5, -0.5, 0.5],
            [-0.5, -0.5, 0.5],
        ];

        let tex = [[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]];

        for face_index in 0..6 {
            let face_uvs = uvs[face_index];
            let uv_width = face_uvs[2] - face_uvs[0];
            let uv_height = face_uvs[3] - face_uvs[1];

            for vert_index in 0..4 {
                let p = pos[face_index * 4 + vert_index];
                let t = tex[vert_index];

                let u = face_uvs[0] + t[0] * uv_width;
                let v = face_uvs[1] + t[1] * uv_height;

                vertices.extend_from_slice(&[p[0], p[1], p[2], u, v]);
            }
        }

        let indices: Vec<u32> = vec![
            0, 3, 2, 2, 1, 0, // Front
            4, 7, 6, 6, 5, 4, // Back
            8, 11, 10, 10, 9, 8, // Right
            12, 15, 14, 14, 13, 12, // Left
            16, 19, 18, 18, 17, 16, // Top
            20, 23, 22, 22, 21, 20, // Bottom
        ];

        Ok(self.register_mesh(vertices, indices))
    }
}

impl Default for MeshRegistry {
    fn default() -> Self {
        Self::new()
    }
}
