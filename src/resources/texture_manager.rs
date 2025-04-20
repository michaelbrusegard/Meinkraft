use crate::gl;
use image::RgbaImage;
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

pub struct TextureManager {
    gl: gl::Gl,
    array_texture_id: gl::types::GLuint,
    texture_layers: HashMap<String, f32>,
    layer_count: u32,
    texture_width: u32,
    texture_height: u32,
}

impl TextureManager {
    pub fn new(gl: gl::Gl) -> Self {
        Self {
            gl,
            array_texture_id: 0,
            texture_layers: HashMap::new(),
            layer_count: 0,
            texture_width: 0,
            texture_height: 0,
        }
    }

    pub fn load_textures_as_array(
        &mut self,
        texture_files: &[(&str, &str)],
    ) -> Result<(), Box<dyn Error>> {
        self.cleanup_texture();

        if texture_files.is_empty() {
            return Ok(());
        }

        let mut images = Vec::new();
        let mut max_width = 0;
        let mut max_height = 0;

        for (i, (name, path)) in texture_files.iter().enumerate() {
            let img = image::open(Path::new(path))
                .map_err(|e| format!("Failed to load texture {}: {}", name, e))?
                .flipv();
            let rgba_img = img.to_rgba8();
            let (width, height) = rgba_img.dimensions();

            max_width = max_width.max(width);
            max_height = max_height.max(height);

            images.push((name.to_string(), rgba_img));
            self.texture_layers.insert(name.to_string(), i as f32);
        }

        self.layer_count = images.len() as u32;
        self.texture_width = max_width;
        self.texture_height = max_height;

        unsafe {
            self.gl.GenTextures(1, &mut self.array_texture_id);
            self.gl
                .BindTexture(gl::TEXTURE_2D_ARRAY, self.array_texture_id);

            self.gl.TexImage3D(
                gl::TEXTURE_2D_ARRAY,
                0,
                gl::RGBA8 as i32,
                max_width as i32,
                max_height as i32,
                self.layer_count as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                std::ptr::null(),
            );

            for (i, (_name, image_data)) in images.iter().enumerate() {
                let (width, height) = image_data.dimensions();
                let mut aligned_image = RgbaImage::new(max_width, max_height);
                image::imageops::overlay(&mut aligned_image, image_data, 0, 0);

                self.gl.TexSubImage3D(
                    gl::TEXTURE_2D_ARRAY,
                    0,
                    0,
                    0,
                    i as i32,
                    width as i32,
                    height as i32,
                    1,
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    aligned_image.as_raw().as_ptr() as *const _,
                );
            }

            self.gl
                .TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            self.gl
                .TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            self.gl.TexParameteri(
                gl::TEXTURE_2D_ARRAY,
                gl::TEXTURE_MIN_FILTER,
                gl::NEAREST_MIPMAP_LINEAR as i32,
            );
            self.gl.TexParameteri(
                gl::TEXTURE_2D_ARRAY,
                gl::TEXTURE_MAG_FILTER,
                gl::NEAREST as i32,
            );

            self.gl.GenerateMipmap(gl::TEXTURE_2D_ARRAY);

            self.gl.BindTexture(gl::TEXTURE_2D_ARRAY, 0);
        }

        Ok(())
    }

    pub fn get_layer_index(&self, name: &str) -> Option<f32> {
        self.texture_layers.get(name).copied()
    }

    pub fn get_all_layers(&self) -> HashMap<String, f32> {
        self.texture_layers.clone()
    }

    pub fn bind_texture_array(&self, texture_unit: gl::types::GLenum) {
        unsafe {
            self.gl.ActiveTexture(texture_unit);
            self.gl
                .BindTexture(gl::TEXTURE_2D_ARRAY, self.array_texture_id);
        }
    }

    fn cleanup_texture(&mut self) {
        if self.array_texture_id != 0 {
            unsafe {
                self.gl.DeleteTextures(1, &self.array_texture_id);
            }
            self.array_texture_id = 0;
        }
        self.texture_layers.clear();
        self.layer_count = 0;
        self.texture_width = 0;
        self.texture_height = 0;
    }
}

impl Drop for TextureManager {
    fn drop(&mut self) {
        self.cleanup_texture();
    }
}
