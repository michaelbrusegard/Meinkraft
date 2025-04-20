use crate::components::{
    chunk_coord_to_aabb_center, ChunkCoord, Renderable, Transform, CHUNK_EXTENTS,
};
use crate::resources::{Camera, MeshRegistry, Renderer, ShaderProgram, TextureManager};
use hecs::World;
use std::sync::Arc;

pub struct RenderSystem {}

impl RenderSystem {
    pub fn new() -> Self {
        RenderSystem {}
    }

    pub fn render(
        &self,
        world: &World,
        camera: &mut Camera,
        renderer: &Renderer,
        shader_program: &ShaderProgram,
        texture_manager: &Arc<TextureManager>,
        mesh_registry: &MeshRegistry,
    ) {
        renderer.clear();
        shader_program.use_program();

        let view_matrix = camera.view_matrix();
        let projection_matrix = camera.projection_matrix();
        let frustum = camera.frustum();

        shader_program.set_uniform_mat4("viewMatrix", &view_matrix);
        shader_program.set_uniform_mat4("projectionMatrix", &projection_matrix);

        texture_manager.bind_texture_array(crate::gl::TEXTURE0);
        shader_program.set_uniform_int("blockTexture", 0);

        for (_entity, (transform, renderable, chunk_coord)) in world
            .query::<(&Transform, &Renderable, &ChunkCoord)>()
            .iter()
        {
            let aabb_center = chunk_coord_to_aabb_center(*chunk_coord);
            if !frustum.intersects_aabb(aabb_center, CHUNK_EXTENTS) {
                continue;
            }

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
