use crate::gl;
use glam::Mat4;
use std::collections::HashMap;
use std::ffi::CString;

pub struct ShaderProgram {
    gl: gl::Gl,
    pub program_id: gl::types::GLuint,
    uniform_locations: HashMap<String, gl::types::GLint>,
}

impl ShaderProgram {
    pub fn from_sources(gl: &gl::Gl, vertex_src: &str, fragment_src: &str) -> Result<Self, String> {
        let vertex_shader = Self::compile_shader(gl, gl::VERTEX_SHADER, vertex_src)?;
        let fragment_shader = Self::compile_shader(gl, gl::FRAGMENT_SHADER, fragment_src)?;

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
                let mut buffer = vec![0; len as usize];
                gl.GetProgramInfoLog(
                    program,
                    len,
                    std::ptr::null_mut(),
                    buffer.as_mut_ptr() as *mut gl::types::GLchar,
                );

                let error_message = std::str::from_utf8(&buffer)
                    .unwrap_or("ProgramInfoLog not valid utf8")
                    .trim_end_matches('\0');

                gl.DeleteShader(vertex_shader);
                gl.DeleteShader(fragment_shader);
                gl.DeleteProgram(program);

                return Err(format!("Shader linking failed: {}", error_message));
            }

            gl.DeleteShader(vertex_shader);
            gl.DeleteShader(fragment_shader);

            program
        };

        Ok(Self {
            gl: gl.clone(),
            program_id,
            uniform_locations: HashMap::new(),
        })
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
                let mut buffer = vec![0; len as usize];
                gl.GetShaderInfoLog(
                    shader,
                    len,
                    std::ptr::null_mut(),
                    buffer.as_mut_ptr() as *mut gl::types::GLchar,
                );

                let error_message = std::str::from_utf8(&buffer)
                    .unwrap_or("ShaderInfoLog not valid utf8")
                    .trim_end_matches('\0');

                gl.DeleteShader(shader);
                return Err(error_message.to_string());
            }

            Ok(shader)
        }
    }

    pub fn use_program(&self) {
        unsafe {
            self.gl.UseProgram(self.program_id);
        }
    }

    pub fn register_uniform(&mut self, name: &str) {
        unsafe {
            let c_name = CString::new(name).unwrap();
            let location = self.gl.GetUniformLocation(self.program_id, c_name.as_ptr());
            if location == -1 {}
            self.uniform_locations.insert(name.to_string(), location);
        }
    }

    pub fn set_uniform_mat4(&self, name: &str, value: &Mat4) {
        if let Some(&location) = self.uniform_locations.get(name) {
            unsafe {
                self.gl
                    .UniformMatrix4fv(location, 1, gl::FALSE, value.as_ref().as_ptr());
            }
        }
    }

    pub fn set_uniform_int(&self, name: &str, value: i32) {
        if let Some(&location) = self.uniform_locations.get(name) {
            unsafe {
                self.gl.Uniform1i(location, value);
            }
        }
    }

    pub fn set_uniform_float(&self, name: &str, value: f32) {
        if let Some(&location) = self.uniform_locations.get(name) {
            unsafe {
                self.gl.Uniform1f(location, value);
            }
        }
    }

    pub fn set_uniform_bool(&self, name: &str, value: bool) {
        if let Some(&location) = self.uniform_locations.get(name) {
            unsafe {
                self.gl.Uniform1i(location, if value { 1 } else { 0 });
            }
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
