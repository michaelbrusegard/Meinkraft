use crate::gl;
use std::ffi::CString;

const VERTEX_SHADER: &str = include_str!("shaders/vertex.glsl");
const FRAGMENT_SHADER: &str = include_str!("shaders/fragment.glsl");

pub struct ShaderProgram {
    gl: gl::Gl,
    pub program_id: gl::types::GLuint,
}

impl ShaderProgram {
    pub fn new(gl: &gl::Gl) -> Self {
        let vertex_shader = Self::compile_shader(gl, gl::VERTEX_SHADER, VERTEX_SHADER);
        let fragment_shader = Self::compile_shader(gl, gl::FRAGMENT_SHADER, FRAGMENT_SHADER);

        let program_id = unsafe {
            let program = gl.CreateProgram();
            gl.AttachShader(program, vertex_shader);
            gl.AttachShader(program, fragment_shader);
            gl.LinkProgram(program);

            let mut success = 0;
            gl.GetProgramiv(program, gl::LINK_STATUS, &mut success);
            if success == 0 {
                let mut len = 0;
                gl.GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
                let mut buffer = vec![0; len as usize - 1];
                gl.GetProgramInfoLog(
                    program,
                    len,
                    std::ptr::null_mut(),
                    buffer.as_mut_ptr() as *mut gl::types::GLchar,
                );
                panic!(
                    "{}",
                    std::str::from_utf8(&buffer).expect("ProgramInfoLog not valid utf8")
                );
            }

            gl.DeleteShader(vertex_shader);
            gl.DeleteShader(fragment_shader);

            program
        };

        Self {
            gl: gl.clone(),
            program_id,
        }
    }

    fn compile_shader(
        gl: &gl::Gl,
        shader_type: gl::types::GLenum,
        source: &str,
    ) -> gl::types::GLuint {
        unsafe {
            let shader = gl.CreateShader(shader_type);
            let shader_source = CString::new(source).unwrap();
            gl.ShaderSource(shader, 1, &shader_source.as_ptr(), std::ptr::null());
            gl.CompileShader(shader);

            let mut success = 0;
            gl.GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
            if success == 0 {
                let mut len = 0;
                gl.GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
                let mut buffer = vec![0; len as usize - 1];
                gl.GetShaderInfoLog(
                    shader,
                    len,
                    std::ptr::null_mut(),
                    buffer.as_mut_ptr() as *mut gl::types::GLchar,
                );
                panic!(
                    "{}",
                    std::str::from_utf8(&buffer).expect("ShaderInfoLog not valid utf8")
                );
            }

            shader
        }
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteProgram(self.program_id);
        }
    }
}
