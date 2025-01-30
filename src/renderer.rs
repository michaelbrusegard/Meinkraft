use crate::gl;
use std::ffi::CString;

pub struct Renderer {
    gl: gl::Gl,
    program: gl::types::GLuint,
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

        let (program, vao) = unsafe {
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

            let vertex_shader = gl.CreateShader(gl::VERTEX_SHADER);
            let fragment_shader = gl.CreateShader(gl::FRAGMENT_SHADER);

            let vertex_shader_source = CString::new(
                r#"
                #version 410
                layout (location = 0) in vec3 position;
                void main() {
                    gl_Position = vec4(position, 1.0);
                }
            "#,
            )
            .unwrap();

            let fragment_shader_source = CString::new(
                r#"
                #version 410
                out vec4 FragColor;
                void main() {
                    FragColor = vec4(1.0, 0.5, 0.2, 1.0);
                }
            "#,
            )
            .unwrap();

            gl.ShaderSource(
                vertex_shader,
                1,
                &vertex_shader_source.as_ptr(),
                std::ptr::null(),
            );
            gl.CompileShader(vertex_shader);

            gl.ShaderSource(
                fragment_shader,
                1,
                &fragment_shader_source.as_ptr(),
                std::ptr::null(),
            );
            gl.CompileShader(fragment_shader);

            let program = gl.CreateProgram();
            gl.AttachShader(program, vertex_shader);
            gl.AttachShader(program, fragment_shader);
            gl.LinkProgram(program);

            // Clean up shaders
            gl.DeleteShader(vertex_shader);
            gl.DeleteShader(fragment_shader);

            (program, vao)
        };

        Self { gl, program, vao }
    }

    pub fn draw(&self) {
        unsafe {
            self.gl.ClearColor(0.1, 0.1, 0.1, 0.9);
            self.gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            self.gl.UseProgram(self.program);
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
            self.gl.DeleteProgram(self.program);
            self.gl.DeleteVertexArrays(1, &self.vao);
        }
    }
}
