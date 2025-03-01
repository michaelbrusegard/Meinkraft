use hecs::World;
use std::ffi::CString;

use crate::components::{Renderable, Transform};
use crate::gl;
use crate::resources::{Camera, GlState};
use crate::shaders::ShaderProgram;

pub fn render_system(
    world: &World,
    gl_state: &GlState,
    camera: &Camera,
    shader_program: &ShaderProgram,
) {
    unsafe {
        gl_state.gl.ClearColor(0.1, 0.1, 0.1, 0.9);
        gl_state
            .gl
            .Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        gl_state.gl.UseProgram(shader_program.program_id);

        let model_name = CString::new("modelMatrix").unwrap();
        let view_name = CString::new("viewMatrix").unwrap();
        let projection_name = CString::new("projectionMatrix").unwrap();

        let model_loc = gl_state
            .gl
            .GetUniformLocation(shader_program.program_id, model_name.as_ptr());
        let view_loc = gl_state
            .gl
            .GetUniformLocation(shader_program.program_id, view_name.as_ptr());
        let projection_loc = gl_state
            .gl
            .GetUniformLocation(shader_program.program_id, projection_name.as_ptr());

        let view_matrix = camera.view_matrix();
        let projection_matrix = camera.projection_matrix();

        gl_state
            .gl
            .UniformMatrix4fv(view_loc, 1, gl::FALSE, view_matrix.as_ref().as_ptr());
        gl_state.gl.UniformMatrix4fv(
            projection_loc,
            1,
            gl::FALSE,
            projection_matrix.as_ref().as_ptr(),
        );

        for (_, (transform, renderable)) in world.query::<(&Transform, &Renderable)>().iter() {
            let model_matrix = transform.model_matrix();
            gl_state
                .gl
                .UniformMatrix4fv(model_loc, 1, gl::FALSE, model_matrix.as_ref().as_ptr());

            if let Some(vao) = gl_state.vaos.get(&renderable.mesh_id) {
                gl_state.gl.BindVertexArray(*vao);
                gl_state
                    .gl
                    .DrawElements(gl::TRIANGLES, 36, gl::UNSIGNED_INT, std::ptr::null());
            }
        }
    }
}
