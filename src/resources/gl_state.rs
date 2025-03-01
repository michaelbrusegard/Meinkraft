use crate::gl;
use std::collections::HashMap;

pub struct GlState {
    pub gl: gl::Gl,
    pub vaos: HashMap<usize, gl::types::GLuint>,
    pub ebos: HashMap<usize, gl::types::GLuint>,
}

impl GlState {
    pub fn new(gl: gl::Gl) -> Self {
        Self {
            gl,
            vaos: HashMap::new(),
            ebos: HashMap::new(),
        }
    }

    pub fn setup_mesh_buffers(&mut self, mesh_id: usize, vertices: &[f32], indices: &[u32]) {
        unsafe {
            let mut vao = 0;
            self.gl.GenVertexArrays(1, &mut vao);
            self.gl.BindVertexArray(vao);

            let mut vbo = 0;
            self.gl.GenBuffers(1, &mut vbo);
            self.gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
            self.gl.BufferData(
                gl::ARRAY_BUFFER,
                std::mem::size_of_val(vertices) as gl::types::GLsizeiptr,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            let mut ebo = 0;
            self.gl.GenBuffers(1, &mut ebo);
            self.gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            self.gl.BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                std::mem::size_of_val(indices) as gl::types::GLsizeiptr,
                indices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            self.gl.VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                3 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                std::ptr::null(),
            );
            self.gl.EnableVertexAttribArray(0);

            self.vaos.insert(mesh_id, vao);
            self.ebos.insert(mesh_id, ebo);
        }
    }

    pub fn cleanup_mesh_buffers(&mut self, mesh_id: usize) {
        unsafe {
            if let Some(vao) = self.vaos.remove(&mesh_id) {
                self.gl.DeleteVertexArrays(1, &vao);
            }

            if let Some(ebo) = self.ebos.remove(&mesh_id) {
                self.gl.DeleteBuffers(1, &ebo);
            }
        }
    }
}
