use crate::gl;
use crate::state::GameState;
use std::ffi::CString;

pub fn render_system(game_state: &GameState) {
    unsafe {
        game_state.gl_state.gl.ClearColor(0.1, 0.1, 0.1, 0.9);
        game_state
            .gl_state
            .gl
            .Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        game_state
            .gl_state
            .gl
            .UseProgram(game_state.shader_program.program_id);

        let model_name = CString::new("modelMatrix").unwrap();
        let view_name = CString::new("viewMatrix").unwrap();
        let projection_name = CString::new("projectionMatrix").unwrap();

        let model_loc = game_state
            .gl_state
            .gl
            .GetUniformLocation(game_state.shader_program.program_id, model_name.as_ptr());
        let view_loc = game_state
            .gl_state
            .gl
            .GetUniformLocation(game_state.shader_program.program_id, view_name.as_ptr());
        let projection_loc = game_state.gl_state.gl.GetUniformLocation(
            game_state.shader_program.program_id,
            projection_name.as_ptr(),
        );

        let view_matrix = game_state.camera.view_matrix();
        let projection_matrix = game_state.camera.projection_matrix();

        game_state.gl_state.gl.UniformMatrix4fv(
            view_loc,
            1,
            gl::FALSE,
            view_matrix.as_ref().as_ptr(),
        );
        game_state.gl_state.gl.UniformMatrix4fv(
            projection_loc,
            1,
            gl::FALSE,
            projection_matrix.as_ref().as_ptr(),
        );

        for (_, (transform, renderable)) in game_state
            .world
            .query::<(
                &crate::components::Transform,
                &crate::components::Renderable,
            )>()
            .iter()
        {
            let model_matrix = transform.model_matrix();
            game_state.gl_state.gl.UniformMatrix4fv(
                model_loc,
                1,
                gl::FALSE,
                model_matrix.as_ref().as_ptr(),
            );

            if let Some(vao) = game_state.gl_state.vaos.get(&renderable.mesh_id) {
                game_state.gl_state.gl.BindVertexArray(*vao);
                game_state.gl_state.gl.DrawElements(
                    gl::TRIANGLES,
                    36,
                    gl::UNSIGNED_INT,
                    std::ptr::null(),
                );
            }
        }
    }
}
