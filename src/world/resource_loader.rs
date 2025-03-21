use crate::resources::{MeshRegistry, Renderer};

pub struct ResourceLoader;

impl ResourceLoader {
    pub fn new() -> Self {
        Self
    }

    pub fn initialize_resources(mesh_registry: &MeshRegistry, renderer: &mut Renderer) {
        for (mesh_id, mesh) in mesh_registry.meshes.iter().enumerate() {
            renderer.setup_mesh_buffers(mesh_id, &mesh.vertices, &mesh.indices);
        }
    }
}
