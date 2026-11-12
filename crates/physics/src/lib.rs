//! Rapier3D physics integration for GTASARUST (Z-up San Andreas world coordinates).

use glam::{Quat, Vec3};
pub use rapier3d::prelude::ColliderHandle;
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
        self.add_static_oriented_box(Vec3::ZERO, half_extents, center, Quat::IDENTITY)
    }

    /// Add a static oriented box collider transformed from model space into Z-up world space.
    pub fn add_static_oriented_box(
        &mut self,
        local_center: Vec3,
        half_extents: Vec3,
        instance_pos: Vec3,
        instance_rot: Quat,
    ) -> ColliderHandle {
        let he = half_extents.abs().max(Vec3::splat(0.02));
        let world_center = instance_pos + instance_rot * local_center;
        let rot = instance_rot.normalize();
        let axis = rot.to_scaled_axis();
        let collider = ColliderBuilder::cuboid(he.x, he.y, he.z)
            .translation(Vector::new(world_center.x, world_center.y, world_center.z))
            .rotation(AngVector::new(axis.x, axis.y, axis.z))
            .build();
        self.collider_set.insert(collider)
    }

    /// Add a static sphere collider transformed from model space into Z-up world space.
    pub fn add_static_sphere(
        &mut self,
        local_center: Vec3,
        radius: f32,
        instance_pos: Vec3,
        instance_rot: Quat,
    ) -> ColliderHandle {
        let world_center = instance_pos + instance_rot * local_center;
        let collider = ColliderBuilder::ball(radius.max(0.05))
            .translation(Vector::new(world_center.x, world_center.y, world_center.z))
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
        self.add_static_transformed_trimesh(vertices, indices, translation, Quat::IDENTITY)
    }

    /// Add a static triangle mesh collider with full world translation and rotation.
    pub fn add_static_transformed_trimesh(
        &mut self,
        vertices: &[[f32; 3]],
        indices: &[[u32; 3]],
        translation: Vec3,
        rotation: Quat,
    ) -> Option<ColliderHandle> {
        if vertices.is_empty() || indices.is_empty() {
            return None;
        }
        let valid_indices: Vec<[u32; 3]> = indices
            .iter()
            .copied()
            .filter(|&[a, b, c]| {
                let (ai, bi, ci) = (a as usize, b as usize, c as usize);
                if ai >= vertices.len() || bi >= vertices.len() || ci >= vertices.len() {
                    return false;
                }
                if a == b || b == c || a == c {
                    return false;
                }
                let va = Vec3::from_array(vertices[ai]);
                let vb = Vec3::from_array(vertices[bi]);
                let vc = Vec3::from_array(vertices[ci]);
                if !va.is_finite() || !vb.is_finite() || !vc.is_finite() {
                    return false;
                }
                (vb - va).cross(vc - va).length_squared() > 1e-10
            })
            .collect();
        if valid_indices.is_empty() {
            return None;
        }
        let pts: Vec<Vector> = vertices
            .iter()
            .map(|v| {
                if v[0].is_finite() && v[1].is_finite() && v[2].is_finite() {
                    Vector::new(v[0], v[1], v[2])
                } else {
                    Vector::ZERO
                }
            })
            .collect();
        let axis = rotation.normalize().to_scaled_axis();
        let collider = ColliderBuilder::trimesh(pts, valid_indices)
            .ok()?
            .translation(Vector::new(translation.x, translation.y, translation.z))
            .rotation(AngVector::new(axis.x, axis.y, axis.z))
            .build();
        Some(self.collider_set.insert(collider))
    }

    /// Remove a previously registered static collider (e.g., when a streaming sector unloads).
    pub fn remove_collider(&mut self, handle: ColliderHandle) {
        self.collider_set.remove(
            handle,
            &mut self.island_manager,
            &mut self.rigid_body_set,
            &mut self.soft_body_set,
            true,
        );
    }

    /// Refresh broad-phase acceleration structures so newly added colliders respond to raycasts immediately.
    pub fn sync_queries(&mut self) {
        self.step(1.0 / 60.0);
    }

    /// Cast a downward probe from `(x, y, ceiling_z)` and return the highest surface Z within `max_drop`.
    pub fn ground_height_below(
        &self,
        x: f32,
        y: f32,
        ceiling_z: f32,
        max_drop: f32,
    ) -> Option<f32> {
        let origin = Vec3::new(x, y, ceiling_z);
        let (toi, _normal) = self.raycast(origin, Vec3::NEG_Z, max_drop)?;
        Some(ceiling_z - toi)
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

    #[test]
    fn transformed_trimesh_and_oriented_box_hit_and_remove_cleanly() {
        let mut world = PhysicsWorld::new();
        let verts = [
            [-5.0, -5.0, 0.0],
            [5.0, -5.0, 0.0],
            [5.0, 5.0, 0.0],
            [-5.0, 5.0, 0.0],
        ];
        let tris = [[0, 1, 2], [0, 2, 3], [0, 0, 1]]; // last tri degenerate, filtered out
        let h_tri = world
            .add_static_transformed_trimesh(
                &verts,
                &tris,
                Vec3::new(2490.0, -1670.0, 13.0),
                Quat::from_rotation_z(0.5),
            )
            .expect("valid trimesh collider");
        let h_box = world.add_static_oriented_box(
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(2.0, 2.0, 1.0),
            Vec3::new(2510.0, -1670.0, 14.0),
            Quat::from_rotation_z(0.25),
        );
        world.sync_queries();

        let road_z = world
            .ground_height_below(2490.0, -1670.0, 30.0, 40.0)
            .expect("trimesh hit");
        assert!((road_z - 13.0).abs() < 0.05, "got {road_z}");

        let box_top = world
            .ground_height_below(2510.0, -1670.0, 30.0, 40.0)
            .expect("box hit");
        assert!((box_top - 16.0).abs() < 0.05, "got {box_top}");

        world.remove_collider(h_tri);
        world.remove_collider(h_box);
        world.sync_queries();

        let fallback_z = world
            .ground_height_below(2490.0, -1670.0, 30.0, 40.0)
            .expect("base plane hit");
        assert!(fallback_z.abs() < 0.05, "got {fallback_z}");
    }
}
