use glam::{Mat4, Vec3};

pub struct Transform {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
}

impl Transform {
    pub fn new(position: Vec3, rotation: Vec3, scale: Vec3) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    pub fn model_matrix(&self) -> Mat4 {
        let translation = Mat4::from_translation(self.position);
        let rotation_x = Mat4::from_rotation_x(self.rotation.x);
        let rotation_y = Mat4::from_rotation_y(self.rotation.y);
        let rotation_z = Mat4::from_rotation_z(self.rotation.z);
        let rotation = rotation_z * rotation_y * rotation_x;
        let scale = Mat4::from_scale(self.scale);

        translation * rotation * scale
    }
}
