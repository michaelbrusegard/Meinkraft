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

        // --- Opaque Pass ---
        // Ideally, depth writing is enabled here
        // unsafe { renderer.gl.DepthMask(gl::TRUE); } // Assuming default is TRUE

        for (_entity, (transform, renderable, chunk_coord)) in world
            .query::<(&Transform, &Renderable, &ChunkCoord)>()
            .iter()
        {
            let aabb_center = chunk_coord_to_aabb_center(*chunk_coord);
            if !frustum.intersects_aabb(aabb_center, CHUNK_EXTENTS) {
                continue;
            }

            // Render Opaque Mesh if present
            if let Some(opaque_mesh_id) = renderable.opaque_mesh_id {
                if let Some(mesh) = mesh_registry.meshes.get(&opaque_mesh_id) {
                    if let Some(vao) = renderer.vaos.get(&opaque_mesh_id) {
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
        }

        // --- Transparent Pass ---
        // Ideally, depth writing is disabled here, but depth testing is still enabled.
        // Blending should already be enabled from Renderer::new.
        // Sorting transparent objects back-to-front would be ideal but is complex.
        // unsafe { renderer.gl.DepthMask(gl::FALSE); } // Disable depth writing

        for (_entity, (transform, renderable, chunk_coord)) in world
            .query::<(&Transform, &Renderable, &ChunkCoord)>()
            .iter()
        // Iterate again for transparent pass
        {
            let aabb_center = chunk_coord_to_aabb_center(*chunk_coord);
            if !frustum.intersects_aabb(aabb_center, CHUNK_EXTENTS) {
                continue;
            }

            // Render Transparent Mesh if present
            if let Some(transparent_mesh_id) = renderable.transparent_mesh_id {
                if let Some(mesh) = mesh_registry.meshes.get(&transparent_mesh_id) {
                    if let Some(vao) = renderer.vaos.get(&transparent_mesh_id) {
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
        }

        // --- Cleanup ---
        // Re-enable depth writing if it was disabled
        // unsafe { renderer.gl.DepthMask(gl::TRUE); }

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
