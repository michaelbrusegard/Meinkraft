use fnv::FnvHashMap;

#[derive(Clone)]
pub struct Mesh {
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,
}

#[derive(Clone, Default)]
pub struct ChunkMeshData {
    pub opaque: Option<Mesh>,
    pub transparent: Option<Mesh>,
}

pub struct MeshRegistry {
    pub meshes: FnvHashMap<usize, Mesh>,
    next_mesh_id: usize,
}

impl MeshRegistry {
    pub fn new() -> Self {
        Self {
            meshes: FnvHashMap::default(),
            next_mesh_id: 0,
        }
    }

    pub fn register_mesh(&mut self, vertices: Vec<f32>, indices: Vec<u32>) -> usize {
        let mesh_id = self.next_mesh_id;
        self.next_mesh_id += 1;
        self.meshes.insert(mesh_id, Mesh { vertices, indices });
        mesh_id
    }

    pub fn update_mesh(&mut self, mesh_id: usize, vertices: Vec<f32>, indices: Vec<u32>) -> usize {
        self.meshes
            .entry(mesh_id)
            .and_modify(|mesh| {
                mesh.vertices = vertices.clone();
                mesh.indices = indices.clone();
            })
            .or_insert_with(|| {
                eprintln!(
                    "Warning: Updating mesh for non-existent ID {}, creating new entry.",
                    mesh_id
                );
                Mesh { vertices, indices }
            });
        mesh_id
    }

    pub fn remove_mesh(&mut self, mesh_id: usize) {
        self.meshes.remove(&mesh_id);
    }
}

impl Default for MeshRegistry {
    fn default() -> Self {
        Self::new()
    }
}
