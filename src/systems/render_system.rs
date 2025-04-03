use crate::components::{Block, BlockType, Renderable, Transform};
use crate::resources::{Camera, Renderer, ShaderProgram};
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
    ) {
        renderer.clear();
        shader_program.use_program();

        let view_matrix = camera.view_matrix();
        let projection_matrix = camera.projection_matrix();

        shader_program.set_uniform_mat4("viewMatrix", &view_matrix);
        shader_program.set_uniform_mat4("projectionMatrix", &projection_matrix);
        shader_program.set_uniform_int("blockTexture", 0);

        unsafe {
            renderer.gl.ActiveTexture(crate::gl::TEXTURE0);
        }

        for (_, (transform, renderable, block)) in
            world.query::<(&Transform, &Renderable, &Block)>().iter()
        {
            let model_matrix = transform.model_matrix();
            shader_program.set_uniform_mat4("modelMatrix", &model_matrix);

            let texture_name = match block.block_type {
                BlockType::Dirt => "dirt",
                BlockType::Stone => "stone",
                BlockType::Grass => "grass_side",
            };

            if let Some(texture_id) = renderer.textures.get(texture_name) {
                unsafe {
                    renderer.gl.BindTexture(crate::gl::TEXTURE_2D, *texture_id);
                }
            }

            if let Some(vao) = renderer.vaos.get(&renderable.mesh_id) {
                unsafe {
                    renderer.gl.BindVertexArray(*vao);
                    renderer.gl.DrawElements(
                        crate::gl::TRIANGLES,
                        36,
                        crate::gl::UNSIGNED_INT,
                        std::ptr::null(),
                    );
                }
            }
        }
    }
}

impl Default for RenderSystem {
    fn default() -> Self {
        Self::new()
    }
}
