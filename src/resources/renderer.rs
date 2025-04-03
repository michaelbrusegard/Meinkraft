use crate::gl;
use crate::resources::MeshRegistry;
use image::GenericImageView;
use std::collections::HashMap;
use std::path::Path;

pub struct Renderer {
    pub gl: gl::Gl,
    pub vaos: HashMap<usize, gl::types::GLuint>,
    pub vbos: HashMap<usize, gl::types::GLuint>,
    pub ebos: HashMap<usize, gl::types::GLuint>,
    pub textures: HashMap<String, gl::types::GLuint>,
}

impl Renderer {
    pub fn new(gl: gl::Gl) -> Self {
        unsafe {
            gl.Enable(gl::DEPTH_BUFFER_BIT);
            gl.Enable(gl::DEPTH_TEST);
            gl.Enable(gl::BLEND);
            gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        Self {
            gl,
            vaos: HashMap::new(),
            vbos: HashMap::new(),
            ebos: HashMap::new(),
            textures: HashMap::new(),
        }
    }

    pub fn load_textures(&mut self) {
        let texture_files = [
            ("dirt", "assets/textures/dirt.png"),
            ("stone", "assets/textures/stone.png"),
            ("grass_side", "assets/textures/grass_side.png"),
            ("grass_top", "assets/textures/grass_top.png"),
        ];

        for (name, path) in texture_files {
            match self.load_texture_from_file(path, name) {
                Ok(_) => println!("Loaded texture: {}", name),
                Err(e) => eprintln!("Failed to load texture {}: {}", name, e),
            }
        }
    }

    fn load_texture_from_file(
        &mut self,
        path: &str,
        name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let img = image::open(Path::new(path))?.flipv();
        let rgba = img.to_rgba8();
        let dimensions = rgba.dimensions();
        let data = rgba.into_raw();

        let mut texture_id = 0;
        unsafe {
            self.gl.GenTextures(1, &mut texture_id);
            self.gl.ActiveTexture(gl::TEXTURE0);
            self.gl.BindTexture(gl::TEXTURE_2D, texture_id);
            self.gl
                .TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            self.gl
                .TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            self.gl
                .TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            self.gl
                .TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            self.gl.TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8 as i32,
                dimensions.0 as i32,
                dimensions.1 as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const _,
            );
            // Generate mipmaps?
            // self.gl.GenerateMipmap(gl::TEXTURE_2D);

            self.gl.BindTexture(gl::TEXTURE_2D, 0);
        }

        self.textures.insert(name.to_string(), texture_id);
        Ok(())
    }

    pub fn initialize_mesh_resources(&mut self, mesh_registry: &MeshRegistry) {
        for (mesh_id, mesh) in mesh_registry.meshes.iter().enumerate() {
            self.setup_mesh_buffers(mesh_id, &mesh.vertices, &mesh.indices);
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

            let stride = 5 * std::mem::size_of::<f32>() as gl::types::GLsizei;

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

            self.gl.BindBuffer(gl::ARRAY_BUFFER, 0);
            self.gl.BindVertexArray(0);
        }
    }

    fn cleanup_textures(&mut self) {
        for texture_id in self.textures.values() {
            unsafe {
                self.gl.DeleteTextures(1, texture_id);
            }
        }
        self.textures.clear();
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
            self.gl.ClearColor(0.1, 0.1, 0.1, 0.9);
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
        self.cleanup_textures();
    }
}
