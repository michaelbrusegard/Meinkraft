use crate::components::{Renderable, Transform};
use crate::resources::{Camera, MeshRegistry, Renderer, ShaderProgram, TextureManager};
use hecs::World;

pub struct RenderSystem {}

impl RenderSystem {
    pub fn new() -> Self {
        RenderSystem {}
    }

    pub fn render(
        &self,
        world: &World,
        camera: &Camera,
        renderer: &Renderer,
        shader_program: &ShaderProgram,
        texture_manager: &TextureManager,
        mesh_registry: &MeshRegistry,
    ) {
        renderer.clear();
        shader_program.use_program();

        let view_matrix = camera.view_matrix();
        let projection_matrix = camera.projection_matrix();
        shader_program.set_uniform_mat4("viewMatrix", &view_matrix);
        shader_program.set_uniform_mat4("projectionMatrix", &projection_matrix);

        texture_manager.bind_atlas(crate::gl::TEXTURE0);
        shader_program.set_uniform_int("blockTexture", 0);

        for (_entity, (transform, renderable)) in world.query::<(&Transform, &Renderable)>().iter()
        {
            if let Some(mesh) = mesh_registry.meshes.get(&renderable.mesh_id) {
                if let Some(vao) = renderer.vaos.get(&renderable.mesh_id) {
                    let model_matrix = transform.model_matrix();
                    shader_program.set_uniform_mat4("modelMatrix", &model_matrix);

                    unsafe {
                        renderer.gl.BindVertexArray(*vao);
                        let index_count = mesh.indices.len() as i32;
                        if index_count > 0 {
                            renderer.gl.DrawElements(
                                crate::gl::TRIANGLES,
                                index_count,
                                crate::gl::UNSIGNED_INT,
                                std::ptr::null(),
                            );
                        }
                    }
                }
            } else {
                eprintln!(
                    "Error: Mesh data not found in registry for mesh_id: {} (entity: {:?})",
                    renderable.mesh_id, _entity
                );
            }
        }

        unsafe {
            renderer.gl.BindVertexArray(0);
        }
    }
}

impl Default for RenderSystem {
    fn default() -> Self {
        Self::new()
    }
}
