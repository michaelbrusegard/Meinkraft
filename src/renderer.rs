use crate::gl;
use crate::shaders::ShaderProgram;
use glam::{Mat4, Vec3};
use std::ffi::CString;

pub struct Renderer {
    gl: gl::Gl,
    shader_program: ShaderProgram,
    vao: gl::types::GLuint,
    ebo: gl::types::GLuint,
    model_matrix: Mat4,
    view_matrix: Mat4,
    projection_matrix: Mat4,
}

impl Renderer {
    pub fn new<D: glutin::display::GlDisplay>(gl_display: &D) -> Self {
        let gl = unsafe {
            let gl = gl::Gl::load_with(|symbol| {
                let symbol = CString::new(symbol).unwrap();
                gl_display.get_proc_address(symbol.as_c_str()).cast()
            });

            gl.Enable(gl::DEPTH_TEST);
            gl
        };

        let vertices: Vec<f32> = vec![
            // Front face
            -0.5, -0.5, 0.5, // 0
            0.5, -0.5, 0.5, // 1
            0.5, 0.5, 0.5, // 2
            -0.5, 0.5, 0.5, // 3
            // Back face
            -0.5, -0.5, -0.5, // 4
            0.5, -0.5, -0.5, // 5
            0.5, 0.5, -0.5, // 6
            -0.5, 0.5, -0.5, // 7
        ];

        let indices: Vec<u32> = vec![
            // Front
            0, 1, 2, 2, 3, 0, // Right
            1, 5, 6, 6, 2, 1, // Back
            5, 4, 7, 7, 6, 5, // Left
            4, 0, 3, 3, 7, 4, // Top
            3, 2, 6, 6, 7, 3, // Bottom
            4, 5, 1, 1, 0, 4,
        ];

        let (shader_program, vao, ebo) = unsafe {
            let mut vao = 0;
            gl.GenVertexArrays(1, &mut vao);
            gl.BindVertexArray(vao);

            let mut vbo = 0;
            gl.GenBuffers(1, &mut vbo);
            gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            let mut ebo = 0;
            gl.GenBuffers(1, &mut ebo);
            gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl.BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * std::mem::size_of::<u32>()) as gl::types::GLsizeiptr,
                indices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            gl.VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                3 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                std::ptr::null(),
            );
            gl.EnableVertexAttribArray(0);

            let shader_program = ShaderProgram::new(&gl);

            (shader_program, vao, ebo)
        };

        // Set up transformation matrices
        let model_matrix = Mat4::from_rotation_x(0.5);
        let view_matrix = Mat4::look_at_rh(
            Vec3::new(0.0, 0.0, 3.0), // Camera position
            Vec3::new(0.0, 0.0, 0.0), // Look at point
            Vec3::new(0.0, 1.0, 0.0), // Up vector
        );
        let projection_matrix = Mat4::perspective_rh(
            45.0f32.to_radians(), // FOV
            800.0 / 600.0,        // Aspect ratio (update this in resize)
            0.1,                  // Near plane
            100.0,                // Far plane
        );

        Self {
            gl,
            shader_program,
            vao,
            ebo,
            model_matrix,
            view_matrix,
            projection_matrix,
        }
    }

    pub fn draw(&self) {
        unsafe {
            self.gl.ClearColor(0.1, 0.1, 0.1, 0.9);
            self.gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            self.gl.UseProgram(self.shader_program.program_id);

            // Create CStrings and bind them to variables
            let model_name = CString::new("model").unwrap();
            let view_name = CString::new("view").unwrap();
            let projection_name = CString::new("projection").unwrap();

            // Update uniforms using the bound CStrings
            let model_loc = self
                .gl
                .GetUniformLocation(self.shader_program.program_id, model_name.as_ptr());
            let view_loc = self
                .gl
                .GetUniformLocation(self.shader_program.program_id, view_name.as_ptr());
            let projection_loc = self
                .gl
                .GetUniformLocation(self.shader_program.program_id, projection_name.as_ptr());

            self.gl
                .UniformMatrix4fv(model_loc, 1, gl::FALSE, self.model_matrix.as_ref().as_ptr());
            self.gl
                .UniformMatrix4fv(view_loc, 1, gl::FALSE, self.view_matrix.as_ref().as_ptr());
            self.gl.UniformMatrix4fv(
                projection_loc,
                1,
                gl::FALSE,
                self.projection_matrix.as_ref().as_ptr(),
            );

            self.gl.BindVertexArray(self.vao);
            self.gl
                .DrawElements(gl::TRIANGLES, 36, gl::UNSIGNED_INT, std::ptr::null());
        }
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        unsafe {
            self.gl.Viewport(0, 0, width, height);
        }
        // Update projection matrix with new aspect ratio
        self.projection_matrix = Mat4::perspective_rh(
            45.0f32.to_radians(),
            width as f32 / height as f32,
            0.1,
            100.0,
        );
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteVertexArrays(1, &self.vao);
            self.gl.DeleteBuffers(1, &self.ebo);
        }
    }
}
