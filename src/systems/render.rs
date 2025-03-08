use crate::gl;
use crate::state::GameState;
use std::ffi::CString;

pub struct RenderSystem {}

impl RenderSystem {
    pub fn new() -> Self {
        RenderSystem {}
    }

    pub fn render(&self, state: &GameState) {
        unsafe {
            state.gl_state.gl.ClearColor(0.1, 0.1, 0.1, 0.9);
            state
                .gl_state
                .gl
                .Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            state
                .gl_state
                .gl
                .UseProgram(state.shader_program.program_id);

            let model_name = CString::new("modelMatrix").unwrap();
            let view_name = CString::new("viewMatrix").unwrap();
            let projection_name = CString::new("projectionMatrix").unwrap();

            let model_loc = state
                .gl_state
                .gl
                .GetUniformLocation(state.shader_program.program_id, model_name.as_ptr());
            let view_loc = state
                .gl_state
                .gl
                .GetUniformLocation(state.shader_program.program_id, view_name.as_ptr());
            let projection_loc = state
                .gl_state
                .gl
                .GetUniformLocation(state.shader_program.program_id, projection_name.as_ptr());

            let view_matrix = state.camera.view_matrix();
            let projection_matrix = state.camera.projection_matrix();

            state.gl_state.gl.UniformMatrix4fv(
                view_loc,
                1,
                gl::FALSE,
                view_matrix.as_ref().as_ptr(),
            );
            state.gl_state.gl.UniformMatrix4fv(
                projection_loc,
                1,
                gl::FALSE,
                projection_matrix.as_ref().as_ptr(),
            );

            for (_, (transform, renderable)) in state
                .world
                .query::<(
                    &crate::components::Transform,
                    &crate::components::Renderable,
                )>()
                .iter()
            {
                let model_matrix = transform.model_matrix();
                state.gl_state.gl.UniformMatrix4fv(
                    model_loc,
                    1,
                    gl::FALSE,
                    model_matrix.as_ref().as_ptr(),
                );

                if let Some(vao) = state.gl_state.vaos.get(&renderable.mesh_id) {
                    state.gl_state.gl.BindVertexArray(*vao);
                    state.gl_state.gl.DrawElements(
                        gl::TRIANGLES,
                        36,
                        gl::UNSIGNED_INT,
                        std::ptr::null(),
                    );
                }
            }
        }
    }
}
