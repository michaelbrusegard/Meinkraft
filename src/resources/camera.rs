use crate::components::CHUNK_WIDTH;
use glam::{Mat4, Vec3};

#[derive(Clone, Copy, Debug)]
struct Plane {
    normal: Vec3,
    distance: f32,
}

impl Plane {
    fn normalize(&mut self) {
        let length = self.normal.length();
        if length > f32::EPSILON {
            self.normal /= length;
            self.distance /= length;
        } else {
            self.normal = Vec3::ZERO;
            self.distance = 0.0;
        }
    }

    fn distance_to_point(&self, point: Vec3) -> f32 {
        self.normal.dot(point) + self.distance
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Frustum {
    planes: [Plane; 6],
}

impl Frustum {
    pub fn intersects_aabb(&self, center: Vec3, extents: Vec3) -> bool {
        for plane in &self.planes {
            let r = extents.x * plane.normal.x.abs()
                + extents.y * plane.normal.y.abs()
                + extents.z * plane.normal.z.abs();

            let s = plane.distance_to_point(center);

            if s + r < -f32::EPSILON {
                return false;
            }
        }
        true
    }
}

pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    aspect_ratio: f32,
    fov_y_radians: f32,
    z_near: f32,
    z_far: f32,
    view_matrix: Mat4,
    projection_matrix: Mat4,
    view_projection_matrix: Mat4,
    frustum: Frustum,
    dirty: bool,
}

impl Camera {
    pub fn new(
        position: Vec3,
        target: Vec3,
        up: Vec3,
        aspect_ratio: f32,
        far_distance_chunks: i32,
    ) -> Self {
        let fov_y_radians = 70.0f32.to_radians();
        let z_near = 0.1;
        let z_far = (far_distance_chunks as f32 + 1.0) * (CHUNK_WIDTH as f32) * 1.5;

        let mut camera = Self {
            position,
            target,
            up,
            aspect_ratio,
            fov_y_radians,
            z_near,
            z_far,
            view_matrix: Mat4::IDENTITY,
            projection_matrix: Mat4::IDENTITY,
            view_projection_matrix: Mat4::IDENTITY,
            frustum: Frustum {
                planes: [Plane {
                    normal: Vec3::ZERO,
                    distance: 0.0,
                }; 6],
            },
            dirty: true,
        };
        camera.recalculate_matrices_and_frustum();
        camera
    }

    fn recalculate_matrices_and_frustum(&mut self) {
        self.view_matrix = Mat4::look_at_rh(self.position, self.target, self.up);
        self.projection_matrix = Mat4::perspective_rh(
            self.fov_y_radians,
            self.aspect_ratio,
            self.z_near,
            self.z_far,
        );
        self.view_projection_matrix = self.projection_matrix * self.view_matrix;
        self.calculate_frustum();
        self.dirty = false;
    }

    fn calculate_frustum(&mut self) {
        let m = self.view_projection_matrix;

        let p0 = m.row(3) + m.row(0);
        self.frustum.planes[0] = Plane {
            normal: Vec3::new(p0.x, p0.y, p0.z),
            distance: p0.w,
        };
        self.frustum.planes[0].normalize();

        let p1 = m.row(3) - m.row(0);
        self.frustum.planes[1] = Plane {
            normal: Vec3::new(p1.x, p1.y, p1.z),
            distance: p1.w,
        };
        self.frustum.planes[1].normalize();

        let p2 = m.row(3) + m.row(1);
        self.frustum.planes[2] = Plane {
            normal: Vec3::new(p2.x, p2.y, p2.z),
            distance: p2.w,
        };
        self.frustum.planes[2].normalize();

        let p3 = m.row(3) - m.row(1);
        self.frustum.planes[3] = Plane {
            normal: Vec3::new(p3.x, p3.y, p3.z),
            distance: p3.w,
        };
        self.frustum.planes[3].normalize();

        let p4 = m.row(2);
        self.frustum.planes[4] = Plane {
            normal: Vec3::new(p4.x, p4.y, p4.z),
            distance: p4.w,
        };
        self.frustum.planes[4].normalize();

        let p5 = m.row(3) - m.row(2);
        self.frustum.planes[5] = Plane {
            normal: Vec3::new(p5.x, p5.y, p5.z),
            distance: p5.w,
        };
        self.frustum.planes[5].normalize();
    }

    pub fn update_aspect_ratio(&mut self, width: f32, height: f32) {
        self.aspect_ratio = width / height;
        self.dirty = true;
    }

    pub fn set_position(&mut self, position: Vec3) {
        if self.position != position {
            self.position = position;
            self.dirty = true;
        }
    }

    pub fn set_target(&mut self, target: Vec3) {
        if self.target != target {
            self.target = target;
            self.dirty = true;
        }
    }

    pub fn set_position_target(&mut self, position: Vec3, target: Vec3) {
        let pos_changed = self.position != position;
        let target_changed = self.target != target;
        if pos_changed || target_changed {
            self.position = position;
            self.target = target;
            self.dirty = true;
        }
    }

    fn ensure_updated(&mut self) {
        if self.dirty {
            self.recalculate_matrices_and_frustum();
        }
    }

    pub fn view_matrix(&mut self) -> Mat4 {
        self.ensure_updated();
        self.view_matrix
    }

    pub fn projection_matrix(&mut self) -> Mat4 {
        self.ensure_updated();
        self.projection_matrix
    }

    pub fn frustum(&mut self) -> Frustum {
        self.ensure_updated();
        self.frustum
    }
}
