use crate::gl;
use image::RgbaImage;
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use texture_packer::importer::ImageImporter;
use texture_packer::texture::Texture;
use texture_packer::{TexturePacker, TexturePackerConfig};

pub type TextureUVs = [f32; 4];

pub struct TextureManager {
    gl: gl::Gl,
    atlas_texture_id: gl::types::GLuint,
    texture_coords: HashMap<String, TextureUVs>,
}

impl TextureManager {
    pub fn new(gl: gl::Gl) -> Self {
        Self {
            gl,
            atlas_texture_id: 0,
            texture_coords: HashMap::new(),
        }
    }

    pub fn load_textures_and_build_atlas(
        &mut self,
        texture_files: &[(&str, &str)],
    ) -> Result<(), Box<dyn Error>> {
        let config = TexturePackerConfig {
            max_width: 1024,
            max_height: 1024,
            allow_rotation: false,
            texture_outlines: false,
            border_padding: 2,
            ..Default::default()
        };

        let mut packer = TexturePacker::new_skyline(config);
        let mut images = HashMap::new();

        for (name, path) in texture_files {
            let img = ImageImporter::import_from_file(Path::new(path))?.flipv();
            images.insert(name.to_string(), img.clone());
            packer
                .pack_own(name.to_string(), img)
                .map_err(|e| format!("Failed to pack texture {}: {:?}", name, e))?;
        }

        let atlas_width = packer.width();
        let atlas_height = packer.height();
        let mut atlas_image = RgbaImage::new(atlas_width, atlas_height);

        self.texture_coords.clear();

        for (name, frame) in packer.get_frames() {
            let source_image = images
                .get(name)
                .ok_or_else(|| format!("Image not found for frame: {}", name))?;
            let frame_rect = frame.frame;

            image::imageops::overlay(
                &mut atlas_image,
                source_image,
                frame_rect.x as i64,
                frame_rect.y as i64,
            );

            let uvs = [
                frame_rect.x as f32 / atlas_width as f32,
                frame_rect.y as f32 / atlas_height as f32,
                (frame_rect.x + frame_rect.w) as f32 / atlas_width as f32,
                (frame_rect.y + frame_rect.h) as f32 / atlas_height as f32,
            ];
            self.texture_coords.insert(name.clone(), uvs);
        }

        if let Err(e) = atlas_image.save("debug_atlas.png") {
            eprintln!("Failed to save debug atlas: {}", e);
        }

        self.atlas_texture_id =
            self.create_gl_texture(atlas_width, atlas_height, &atlas_image.into_raw());

        Ok(())
    }

    fn create_gl_texture(&self, width: u32, height: u32, data: &[u8]) -> gl::types::GLuint {
        let mut texture_id = 0;
        unsafe {
            self.gl.GenTextures(1, &mut texture_id);
            self.gl.BindTexture(gl::TEXTURE_2D, texture_id);

            self.gl
                .TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            self.gl
                .TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            self.gl
                .TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            self.gl
                .TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            self.gl.TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8 as i32,
                width as i32,
                height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const _,
            );

            self.gl.BindTexture(gl::TEXTURE_2D, 0);
        }
        texture_id
    }

    pub fn get_uvs(&self, name: &str) -> Option<TextureUVs> {
        self.texture_coords.get(name).copied()
    }

    pub fn bind_atlas(&self, texture_unit: gl::types::GLenum) {
        unsafe {
            self.gl.ActiveTexture(texture_unit);
            self.gl.BindTexture(gl::TEXTURE_2D, self.atlas_texture_id);
        }
    }

    fn cleanup_texture(&mut self) {
        if self.atlas_texture_id != 0 {
            unsafe {
                self.gl.DeleteTextures(1, &self.atlas_texture_id);
            }
            self.atlas_texture_id = 0;
        }
        self.texture_coords.clear();
    }
}

impl Drop for TextureManager {
    fn drop(&mut self) {
        self.cleanup_texture();
    }
}
