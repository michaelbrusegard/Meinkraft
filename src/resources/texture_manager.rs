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

    fn pad_image_with_bleed(img: &RgbaImage, padding: u32) -> RgbaImage {
        if padding == 0 {
            return img.clone();
        }
        let (width, height) = img.dimensions();
        let new_width = width + 2 * padding;
        let new_height = height + 2 * padding;
        let mut padded_img = RgbaImage::new(new_width, new_height);
        image::imageops::overlay(&mut padded_img, img, padding as i64, padding as i64);
        for p in 1..=padding {
            let rx1 = width - 1;
            let wx1 = padding + width + p - 1;
            for y in 0..height {
                padded_img.put_pixel(wx1, padding + y, *img.get_pixel(rx1, y));
            }
            let rx2 = 0;
            let wx2 = padding - p;
            for y in 0..height {
                padded_img.put_pixel(wx2, padding + y, *img.get_pixel(rx2, y));
            }
            let ry1 = height - 1;
            let wy1 = padding + height + p - 1;
            for x in 0..width {
                padded_img.put_pixel(padding + x, wy1, *img.get_pixel(x, ry1));
            }
            let ry2 = 0;
            let wy2 = padding - p;
            for x in 0..width {
                padded_img.put_pixel(padding + x, wy2, *img.get_pixel(x, ry2));
            }
        }
        let tl = *img.get_pixel(0, 0);
        let tr = *img.get_pixel(width - 1, 0);
        let bl = *img.get_pixel(0, height - 1);
        let br = *img.get_pixel(width - 1, height - 1);
        for py in 1..=padding {
            for px in 1..=padding {
                padded_img.put_pixel(padding - px, padding - py, tl);
                padded_img.put_pixel(padding + width + px - 1, padding - py, tr);
                padded_img.put_pixel(padding - px, padding + height + py - 1, bl);
                padded_img.put_pixel(padding + width + px - 1, padding + height + py - 1, br);
            }
        }
        padded_img
    }

    pub fn load_textures_and_build_atlas(
        &mut self,
        texture_files: &[(&str, &str)],
    ) -> Result<(), Box<dyn Error>> {
        let image_padding: u32 = 8;

        let config = TexturePackerConfig {
            max_width: 1024,
            max_height: 1024,
            allow_rotation: false,
            texture_outlines: false,
            force_max_dimensions: false,
            trim: false,
            border_padding: 0,
            texture_padding: 0,
            texture_extrusion: 0,
        };

        let mut packer = TexturePacker::new_skyline(config);
        let mut padded_images = HashMap::new();

        for (name, path) in texture_files {
            let dynamic_img = ImageImporter::import_from_file(Path::new(path))?.flipv();
            let original_rgba_img = dynamic_img.to_rgba8();
            let padded_img = Self::pad_image_with_bleed(&original_rgba_img, image_padding);
            padded_images.insert(name.to_string(), padded_img.clone());
            packer
                .pack_own(name.to_string(), padded_img)
                .map_err(|e| format!("Failed to pack texture {}: {:?}", name, e))?;
        }

        let atlas_width = packer.width();
        let atlas_height = packer.height();
        let mut atlas_image = RgbaImage::new(atlas_width, atlas_height);

        self.texture_coords.clear();

        let atlas_width_f = atlas_width as f32;
        let atlas_height_f = atlas_height as f32;

        for (name, frame) in packer.get_frames() {
            let source_padded_image = padded_images
                .get(name)
                .ok_or_else(|| format!("Padded image not found for frame: {}", name))?;
            let frame_rect = frame.frame;

            image::imageops::overlay(
                &mut atlas_image,
                source_padded_image,
                frame_rect.x as i64,
                frame_rect.y as i64,
            );

            let content_u_min = (frame_rect.x + image_padding) as f32 / atlas_width_f;
            let content_v_min = (frame_rect.y + image_padding) as f32 / atlas_height_f;
            let content_u_max =
                (frame_rect.x + frame_rect.w - image_padding) as f32 / atlas_width_f;
            let content_v_max =
                (frame_rect.y + frame_rect.h - image_padding) as f32 / atlas_height_f;

            let uvs = [content_u_min, content_v_min, content_u_max, content_v_max];
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
            self.gl.TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::NEAREST_MIPMAP_LINEAR as i32,
            );
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
            self.gl.GenerateMipmap(gl::TEXTURE_2D);
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
