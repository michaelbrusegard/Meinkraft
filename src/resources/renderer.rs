use crate::gl;
use std::collections::HashMap;

pub struct Renderer {
    pub gl: gl::Gl,
    pub vaos: HashMap<usize, gl::types::GLuint>,
    pub vbos: HashMap<usize, gl::types::GLuint>,
    pub ebos: HashMap<usize, gl::types::GLuint>,
}

impl Renderer {
    pub fn new(gl: gl::Gl) -> Self {
        unsafe {
            gl.Enable(gl::DEPTH_TEST);
            gl.Enable(gl::BLEND);
            gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl.PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
            gl.Enable(gl::CULL_FACE);
            gl.CullFace(gl::BACK);
            gl.FrontFace(gl::CCW);
        }

        Self {
            gl,
            vaos: HashMap::new(),
            vbos: HashMap::new(),
            ebos: HashMap::new(),
        }
    }

    pub fn upload_mesh_buffers(&mut self, mesh_id: usize, vertices: &[f32], indices: &[u32]) {
        self.cleanup_mesh_buffers(mesh_id);

        unsafe {
            let mut vao = 0;
            self.gl.GenVertexArrays(1, &mut vao);
            self.gl.BindVertexArray(vao);

            let mut vbo = 0;
            self.gl.GenBuffers(1, &mut vbo);
            self.gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
            self.gl.BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            let mut ebo = 0;
            self.gl.GenBuffers(1, &mut ebo);
            self.gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            self.gl.BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * std::mem::size_of::<u32>()) as gl::types::GLsizeiptr,
                indices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            let stride = (5 * std::mem::size_of::<f32>()) as gl::types::GLsizei;

            self.gl
                .VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, std::ptr::null());
            self.gl.EnableVertexAttribArray(0);

            self.gl.VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                stride,
                (3 * std::mem::size_of::<f32>()) as *const _,
            );
            self.gl.EnableVertexAttribArray(1);

            self.vaos.insert(mesh_id, vao);
            self.vbos.insert(mesh_id, vbo);
            self.ebos.insert(mesh_id, ebo);

            self.gl.BindVertexArray(0);
            self.gl.BindBuffer(gl::ARRAY_BUFFER, 0);
            self.gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }
    }

    fn cleanup_mesh_buffers(&mut self, mesh_id: usize) {
        unsafe {
            if let Some(vao) = self.vaos.remove(&mesh_id) {
                self.gl.DeleteVertexArrays(1, &vao);
            }
            if let Some(vbo) = self.vbos.remove(&mesh_id) {
                self.gl.DeleteBuffers(1, &vbo);
            }
            if let Some(ebo) = self.ebos.remove(&mesh_id) {
                self.gl.DeleteBuffers(1, &ebo);
            }
        }
    }

    pub fn resize(&self, width: i32, height: i32) {
        unsafe {
            self.gl.Viewport(0, 0, width, height);
        }
    }

    pub fn clear(&self) {
        unsafe {
            self.gl.ClearColor(0.1, 0.1, 0.1, 1.0);
            self.gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        let mesh_ids: Vec<usize> = self.vaos.keys().copied().collect();
        for mesh_id in mesh_ids {
            self.cleanup_mesh_buffers(mesh_id);
        }
    }
}
