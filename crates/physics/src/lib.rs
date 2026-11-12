//! Rapier3D physics integration for GTASARUST (Z-up San Andreas world coordinates).

use glam::Vec3;
use rapier3d::prelude::*;

pub struct PhysicsWorld {
    pub gravity: Vector,
    pub integration_parameters: IntegrationParameters,
    pub physics_pipeline: PhysicsPipeline,
    pub island_manager: IslandManager,
    pub broad_phase: DefaultBroadPhase,
    pub narrow_phase: NarrowPhase,
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub impulse_joint_set: ImpulseJointSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub soft_body_set: SoftBodySet,
    pub ccd_solver: CCDSolver,
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new()
    }
}

impl PhysicsWorld {
    pub fn new() -> Self {
        let mut world = Self {
            gravity: Vector::new(0.0, 0.0, -9.81),
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            soft_body_set: SoftBodySet::new(),
            ccd_solver: CCDSolver::new(),
        };
        // Large ground plane at Z = 0 so fallback/streaming worlds always have base support
        world.add_static_box(Vec3::new(0.0, 0.0, -1.0), Vec3::new(3500.0, 3500.0, 1.0));
        world
    }

    /// Add a static box collider in Z-up world space.
    pub fn add_static_box(&mut self, center: Vec3, half_extents: Vec3) -> ColliderHandle {
        let collider = ColliderBuilder::cuboid(half_extents.x, half_extents.y, half_extents.z)
            .translation(Vector::new(center.x, center.y, center.z))
            .build();
        self.collider_set.insert(collider)
    }

    /// Add a static triangle mesh collider (e.g., from parsed `.col` geometry).
    pub fn add_static_trimesh(
        &mut self,
        vertices: &[[f32; 3]],
        indices: &[[u32; 3]],
        translation: Vec3,
    ) -> Option<ColliderHandle> {
        if vertices.is_empty() || indices.is_empty() {
            return None;
        }
        let pts: Vec<Vector> = vertices
            .iter()
            .map(|v| Vector::new(v[0], v[1], v[2]))
            .collect();
        let collider = ColliderBuilder::trimesh(pts, indices.to_vec())
            .ok()?
            .translation(Vector::new(translation.x, translation.y, translation.z))
            .build();
        Some(self.collider_set.insert(collider))
    }

    /// Cast a ray in Z-up world space and return hit distance and surface normal.
    pub fn raycast(&self, origin: Vec3, dir: Vec3, max_dist: f32) -> Option<(f32, Vec3)> {
        let query_pipeline = self.broad_phase.as_query_pipeline(
            self.narrow_phase.query_dispatcher(),
            &self.rigid_body_set,
            &self.collider_set,
            QueryFilter::default(),
        );
        let ray = Ray::new(
            Vector::new(origin.x, origin.y, origin.z),
            Vector::new(dir.x, dir.y, dir.z),
        );
        let (_, intersection) = query_pipeline.cast_ray_and_get_normal(&ray, max_dist, true)?;
        let n = intersection.normal;
        Some((intersection.time_of_impact, Vec3::new(n.x, n.y, n.z)))
    }

    pub fn step(&mut self, dt: f32) {
        self.integration_parameters.dt = dt;
        self.physics_pipeline.step(
            self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.soft_body_set,
            &mut self.ccd_solver,
            &(),
            &(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ground_raycast_hits_z_zero() {
        let mut world = PhysicsWorld::new();
        world.step(1.0 / 60.0);
        let hit = world.raycast(Vec3::new(2495.0, -1666.0, 10.0), Vec3::NEG_Z, 50.0);
        let (toi, normal) = hit.expect("ray should hit ground plane");
        assert!((toi - 10.0).abs() < 0.05);
        assert!(normal.z > 0.9);
    }
}
