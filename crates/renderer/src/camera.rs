use glam::{Mat4, Vec3, Vec4};

/// Perspective third-person / vehicle camera operating in Z-up world space.
#[derive(Debug, Clone)]
pub struct CameraState {
    pub position: Vec3,
    pub target: Vec3,
    pub fov_radians: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            position: Vec3::new(2487.0, -1688.0, 5.0),
            target: Vec3::new(2495.3, -1688.2, 1.5),
            fov_radians: 70.0_f32.to_radians(),
            aspect: 16.0 / 9.0,
            near: 0.2,
            far: 1500.0,
        }
    }
}

impl CameraState {
    pub fn update_third_person(&mut self, target_pos: Vec3, yaw: f32, pitch: f32, distance: f32) {
        let look_target = target_pos + Vec3::new(0.0, 0.0, 1.4);
        let dir = Vec3::new(
            yaw.cos() * pitch.cos(),
            yaw.sin() * pitch.cos(),
            pitch.sin(),
        )
        .normalize();
        self.target = look_target;
        self.position = look_target - dir * distance;
        if self.position.z < 0.4 {
            self.position.z = 0.4;
        }
    }

    pub fn view_matrix(&self) -> Mat4 {
        glam::camera::rh::view::look_at_mat4(self.position, self.target, Vec3::Z)
    }

    pub fn proj_matrix(&self) -> Mat4 {
        glam::camera::rh::proj::directx::perspective(
            self.fov_radians,
            self.aspect.max(0.1),
            self.near,
            self.far,
        )
    }

    pub fn view_proj_matrix(&self) -> Mat4 {
        self.proj_matrix() * self.view_matrix()
    }

    pub fn frustum(&self) -> Frustum {
        Frustum::from_view_proj(self.view_proj_matrix())
    }
}

/// Six frustum planes for fast bounding-sphere culling.
#[derive(Debug, Clone)]
pub struct Frustum {
    pub planes: [Vec4; 6],
}

impl Frustum {
    pub fn from_view_proj(m: Mat4) -> Self {
        let r0 = m.row(0);
        let r1 = m.row(1);
        let r2 = m.row(2);
        let r3 = m.row(3);
        let raw = [
            r3 + r0, // left
            r3 - r0, // right
            r3 + r1, // bottom
            r3 - r1, // top
            r2,      // near (0..1 wgpu clip Z)
            r3 - r2, // far
        ];
        let planes = raw.map(|p| {
            let len = p.truncate().length().max(1e-6);
            p / len
        });
        Self { planes }
    }

    pub fn intersects_sphere(&self, center: Vec3, radius: f32) -> bool {
        for p in &self.planes {
            let d = p.x * center.x + p.y * center.y + p.z * center.z + p.w;
            if d < -radius {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frustum_accepts_target_and_culls_behind_camera() {
        let mut cam = CameraState::default();
        cam.update_third_person(Vec3::new(100.0, 0.0, 0.0), 0.0, -0.1, 6.0);
        let frustum = cam.frustum();
        assert!(frustum.intersects_sphere(Vec3::new(100.0, 0.0, 1.0), 1.0));
        assert!(!frustum.intersects_sphere(Vec3::new(50.0, 0.0, 1.0), 1.0));
    }
}
