use crate::gl;
use fnv::FnvHashMap;
use glam::Vec3;
use rand::Rng;

pub struct Renderer {
    pub gl: gl::Gl,
    pub vaos: FnvHashMap<usize, gl::types::GLuint>,
    pub vbos: FnvHashMap<usize, gl::types::GLuint>,
    pub ebos: FnvHashMap<usize, gl::types::GLuint>,
    celestial_vao: gl::types::GLuint,
    celestial_vbo: gl::types::GLuint,
    celestial_ebo: gl::types::GLuint,
    star_vao: gl::types::GLuint,
    star_vbo: gl::types::GLuint,
    pub num_stars: usize,
}

impl Renderer {
    pub fn new(gl: gl::Gl) -> Self {
        unsafe {
            gl.ClearColor(0.1, 0.1, 0.1, 1.0);
            gl.Enable(gl::DEPTH_TEST);
            gl.DepthFunc(gl::LESS);
            gl.Enable(gl::BLEND);
            gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl.PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
            gl.Enable(gl::CULL_FACE);
            gl.CullFace(gl::BACK);
            gl.FrontFace(gl::CCW);
        }
        let mut renderer = Self {
            gl,
            vaos: FnvHashMap::default(),
            vbos: FnvHashMap::default(),
            ebos: FnvHashMap::default(),
            celestial_vao: 0,
            celestial_vbo: 0,
            celestial_ebo: 0,
            star_vao: 0,
            star_vbo: 0,
            num_stars: 0,
        };
        renderer.create_celestial_buffers();
        renderer.create_star_buffers();
        renderer
    }

    fn create_celestial_buffers(&mut self) {
        let vertices: [f32; 32] = [
            -0.5, -0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, // Bottom-left
            0.5, -0.5, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, // Bottom-right
            0.5, 0.5, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, // Top-right
            -0.5, 0.5, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, // Top-left
        ];
        let indices: [u32; 6] = [0, 1, 2, 2, 3, 0];

        unsafe {
            self.gl.GenVertexArrays(1, &mut self.celestial_vao);
            self.gl.BindVertexArray(self.celestial_vao);

            self.gl.GenBuffers(1, &mut self.celestial_vbo);
            self.gl.BindBuffer(gl::ARRAY_BUFFER, self.celestial_vbo);
            self.gl.BufferData(
                gl::ARRAY_BUFFER,
                std::mem::size_of_val(&vertices) as gl::types::GLsizeiptr,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            self.gl.GenBuffers(1, &mut self.celestial_ebo);
            self.gl
                .BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.celestial_ebo);
            self.gl.BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                std::mem::size_of_val(&indices) as gl::types::GLsizeiptr,
                indices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            let stride = (8 * std::mem::size_of::<f32>()) as gl::types::GLsizei;

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

            self.gl.VertexAttribPointer(
                3,
                3,
                gl::FLOAT,
                gl::FALSE,
                stride,
                (5 * std::mem::size_of::<f32>()) as *const _,
            );
            self.gl.EnableVertexAttribArray(3);

            self.gl.BindVertexArray(0);
            self.gl.BindBuffer(gl::ARRAY_BUFFER, 0);
            self.gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }
    }

    fn create_star_buffers(&mut self) {
        const NUM_STARS: usize = 3000;
        let mut star_directions: Vec<f32> = Vec::with_capacity(NUM_STARS * 3);
        let mut rng = rand::rng();

        for _ in 0..NUM_STARS {
            loop {
                let x: f32 = rng.random_range(-1.0..1.0);
                let y: f32 = rng.random_range(-1.0..1.0);
                let z: f32 = rng.random_range(-1.0..1.0);
                let len_sq = x * x + y * y + z * z;
                if len_sq > 0.0f32 && len_sq < 1.0f32 {
                    let len = len_sq.sqrt();
                    star_directions.push(x / len);
                    star_directions.push(y / len);
                    star_directions.push(z / len);
                    break;
                }
            }
        }
        self.num_stars = NUM_STARS;

        unsafe {
            self.gl.GenVertexArrays(1, &mut self.star_vao);
            self.gl.BindVertexArray(self.star_vao);

            self.gl.GenBuffers(1, &mut self.star_vbo);
            self.gl.BindBuffer(gl::ARRAY_BUFFER, self.star_vbo);
            self.gl.BufferData(
                gl::ARRAY_BUFFER,
                std::mem::size_of_val(star_directions.as_slice()) as gl::types::GLsizeiptr,
                star_directions.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            let stride = (3 * std::mem::size_of::<f32>()) as gl::types::GLsizei;
            self.gl
                .VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, std::ptr::null());
            self.gl.EnableVertexAttribArray(0);

            self.gl.BindVertexArray(0);
            self.gl.BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }

    pub fn bind_celestial_vao(&self) {
        unsafe {
            self.gl.BindVertexArray(self.celestial_vao);
        }
    }

    pub fn bind_star_vao(&self) {
        unsafe {
            self.gl.BindVertexArray(self.star_vao);
        }
    }

    pub fn upload_mesh_buffers(&mut self, mesh_id: usize, vertices: &[f32], indices: &[u32]) {
        self.cleanup_mesh_buffers(mesh_id);

        if vertices.is_empty() || indices.is_empty() {
            return;
        }

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

            let stride = (9 * std::mem::size_of::<f32>()) as gl::types::GLsizei;

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

            self.gl.VertexAttribPointer(
                2,
                1,
                gl::FLOAT,
                gl::FALSE,
                stride,
                (5 * std::mem::size_of::<f32>()) as *const _,
            );
            self.gl.EnableVertexAttribArray(2);

            self.gl.VertexAttribPointer(
                3,
                3,
                gl::FLOAT,
                gl::FALSE,
                stride,
                (6 * std::mem::size_of::<f32>()) as *const _,
            );
            self.gl.EnableVertexAttribArray(3);

            self.vaos.insert(mesh_id, vao);
            self.vbos.insert(mesh_id, vbo);
            self.ebos.insert(mesh_id, ebo);

            self.gl.BindVertexArray(0);
            self.gl.BindBuffer(gl::ARRAY_BUFFER, 0);
            self.gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }
    }

    pub fn cleanup_mesh_buffers(&mut self, mesh_id: usize) {
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

    pub fn clear(&self, sky_color: Vec3) {
        unsafe {
            self.gl
                .ClearColor(sky_color.x, sky_color.y, sky_color.z, 1.0);
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
        unsafe {
            if self.celestial_vao != 0 {
                self.gl.DeleteVertexArrays(1, &self.celestial_vao);
            }
            if self.celestial_vbo != 0 {
                self.gl.DeleteBuffers(1, &self.celestial_vbo);
            }
            if self.celestial_ebo != 0 {
                self.gl.DeleteBuffers(1, &self.celestial_ebo);
            }
            if self.star_vao != 0 {
                self.gl.DeleteVertexArrays(1, &self.star_vao);
            }
            if self.star_vbo != 0 {
                self.gl.DeleteBuffers(1, &self.star_vbo);
            }
        }
    }
}
