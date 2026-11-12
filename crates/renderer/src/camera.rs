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
        self.update_third_person_collided(target_pos, yaw, pitch, distance, None);
    }

    /// Update third-person orbit camera and pull forward cleanly if a `.col` ray hits a wall/ground.
    pub fn update_third_person_collided(
        &mut self,
        target_pos: Vec3,
        yaw: f32,
        pitch: f32,
        distance: f32,
        ray_hit: Option<&dyn Fn(Vec3, Vec3, f32) -> Option<f32>>,
    ) {
        let look_target = target_pos + Vec3::new(0.0, 0.0, 1.35);
        let dir = Vec3::new(
            yaw.cos() * pitch.cos(),
            yaw.sin() * pitch.cos(),
            pitch.sin(),
        )
        .normalize();
        let cam_ray_dir = -dir;
        let mut effective_dist = distance.clamp(1.6, 12.0);
        if let Some(probe) = ray_hit
            && let Some(hit_dist) = probe(look_target, cam_ray_dir, effective_dist + 0.35)
        {
            effective_dist = (hit_dist - 0.35).clamp(0.75, effective_dist);
        }
        self.target = look_target;
        self.position = look_target + cam_ray_dir * effective_dist;
    }

    /// GTA:SA over-the-shoulder weapon aiming camera: pulls closer, shifts right of CJ's shoulder,
    /// and projects the aim target along the camera forward axis so shots hit the HUD reticle.
    pub fn update_over_shoulder(
        &mut self,
        target_pos: Vec3,
        yaw: f32,
        pitch: f32,
        ray_hit: Option<&dyn Fn(Vec3, Vec3, f32) -> Option<f32>>,
    ) {
        let dir = Vec3::new(
            yaw.cos() * pitch.cos(),
            yaw.sin() * pitch.cos(),
            pitch.sin(),
        )
        .normalize();
        let right = Vec3::new(yaw.sin(), -yaw.cos(), 0.0).normalize();
        let pivot = target_pos + Vec3::new(0.0, 0.0, 1.52) + right * 0.48;
        let cam_ray_dir = -dir;
        let mut back_dist = 2.05_f32;
        if let Some(probe) = ray_hit
            && let Some(hit_dist) = probe(pivot, cam_ray_dir, back_dist + 0.3)
        {
            back_dist = (hit_dist - 0.3).clamp(0.55, back_dist);
        }
        self.position = pivot + cam_ray_dir * back_dist;
        self.target = pivot + dir * 24.0;
    }

    /// World-space aim ray `(origin, normalized_direction)` originating from camera through reticle center.
    pub fn aim_ray(&self) -> (Vec3, Vec3) {
        let dir = (self.target - self.position).normalize_or_zero();
        let safe_dir = if dir.length_squared() > 1e-6 {
            dir
        } else {
            Vec3::X
        };
        (self.position, safe_dir)
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

    #[test]
    fn collided_camera_pulls_closer_when_ray_hits_obstacle() {
        let mut cam = CameraState::default();
        let hit_wall = |_orig: Vec3, _dir: Vec3, _max: f32| Some(2.0_f32);
        cam.update_third_person_collided(Vec3::ZERO, 0.0, 0.0, 8.0, Some(&hit_wall));
        let dist = cam.position.distance(cam.target);
        assert!(
            (dist - 1.65).abs() < 1e-3,
            "Expected collided distance ~1.65, got {dist}"
        );
    }

    #[test]
    fn over_shoulder_camera_offsets_right_and_aligns_aim_ray() {
        let mut cam = CameraState::default();
        cam.update_over_shoulder(Vec3::new(10.0, 20.0, 1.0), 0.0, 0.0, None);
        assert!(
            cam.position.x < 10.0,
            "camera should sit behind player along -X"
        );
        assert!(
            cam.position.y < 20.0,
            "camera should shift to right (-Y) shoulder when yaw=0"
        );
        let (_, dir) = cam.aim_ray();
        assert!((dir.x - 1.0).abs() < 1e-4 && dir.y.abs() < 1e-4);
    }
}
