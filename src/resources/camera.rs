use crate::components::CHUNK_WIDTH;
use glam::{Mat4, Vec3};

pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub aspect_ratio: f32,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new(
        position: Vec3,
        target: Vec3,
        up: Vec3,
        aspect_ratio: f32,
        distance_chunks: i32,
    ) -> Self {
        let far = (distance_chunks as f32) * (CHUNK_WIDTH as f32) + (CHUNK_WIDTH as f32);
        Self {
            position,
            target,
            up,
            aspect_ratio,
            fov: 90.0f32.to_radians(),
            near: 0.1,
            far,
        }
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, self.up)
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov, self.aspect_ratio, self.near, self.far)
    }

    pub fn update_aspect_ratio(&mut self, width: f32, height: f32) {
        self.aspect_ratio = width / height;
    }
}
