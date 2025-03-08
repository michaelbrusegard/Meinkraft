use crate::gl;
use std::collections::HashMap;
use std::ffi::CString;

const VERTEX_SHADER: &str = include_str!("shaders/vertex.glsl");
const FRAGMENT_SHADER: &str = include_str!("shaders/fragment.glsl");

pub struct ShaderProgram {
    gl: gl::Gl,
    pub program_id: gl::types::GLuint,
    uniform_locations: HashMap<String, gl::types::GLint>,
}

impl ShaderProgram {
    pub fn new(gl: &gl::Gl) -> Self {
        let vertex_shader = Self::compile_shader(gl, gl::VERTEX_SHADER, VERTEX_SHADER)
            .expect("Vertex shader compilation failed");

        let fragment_shader = Self::compile_shader(gl, gl::FRAGMENT_SHADER, FRAGMENT_SHADER)
            .expect("Fragment shader compilation failed");

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
            uniform_locations: HashMap::new(),
        }
    }

    pub fn get_uniform_location(&mut self, name: &str) -> gl::types::GLint {
        if !self.uniform_locations.contains_key(name) {
            let c_name = CString::new(name).unwrap();
            let location = unsafe { self.gl.GetUniformLocation(self.program_id, c_name.as_ptr()) };
            self.uniform_locations.insert(name.to_string(), location);
        }
        *self.uniform_locations.get(name).unwrap()
    }

    fn compile_shader(
        gl: &gl::Gl,
        shader_type: gl::types::GLenum,
        source: &str,
    ) -> Result<gl::types::GLuint, String> {
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

                let error_message =
                    std::str::from_utf8(&buffer).expect("ShaderInfoLog not valid utf8");

                gl.DeleteShader(shader);
                return Err(error_message.to_string());
            }

            Ok(shader)
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
