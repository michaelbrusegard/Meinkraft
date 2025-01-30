use crate::gl;
use crate::shaders::ShaderProgram;
use std::ffi::CString;

pub struct Renderer {
    gl: gl::Gl,
    shader_program: ShaderProgram,
    vao: gl::types::GLuint,
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

        let vertices: Vec<f32> = vec![-0.5, -0.5, 0.0, 0.5, -0.5, 0.0, 0.0, 0.5, 0.0];

        let (shader_program, vao) = unsafe {
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

            (shader_program, vao)
        };

        Self {
            gl,
            shader_program,
            vao,
        }
    }

    pub fn draw(&self) {
        unsafe {
            self.gl.ClearColor(0.1, 0.1, 0.1, 0.9);
            self.gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            self.gl.UseProgram(self.shader_program.program_id);
            self.gl.BindVertexArray(self.vao);
            self.gl.DrawArrays(gl::TRIANGLES, 0, 3);
        }
    }

    pub fn resize(&self, width: i32, height: i32) {
        unsafe {
            self.gl.Viewport(0, 0, width, height);
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteVertexArrays(1, &self.vao);
        }
    }
}
