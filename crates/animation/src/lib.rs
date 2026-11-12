//! Skeletal animation state machine and GTA:SA `.ifp` (`ANP3`) bone pose evaluator.

use glam::{Mat4, Quat, Vec3};
use gtasarust_gta_formats::{
    ifp::{IfpClip, IfpFullArchive},
    rw::{DffFrame, DffSkin},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AnimState {
    #[default]
    Idle,
    Walk,
    Run,
    Sprint,
    Jump,
    Fall,
    Crouch,
    Swim,
    MeleePunch,
    AimWeapon,
    Drive,
    EnterVehicle,
}

/// Interpolated local pose for a single bone (`bone_id`).
#[derive(Debug, Clone, Copy)]
pub struct BonePose {
    pub rotation: Quat,
    pub translation: Option<Vec3>,
}

/// Runtime library of decoded `.ifp` animation clips keyed by lowercase clip name.
#[derive(Debug, Clone, Default)]
pub struct IfpPoseLibrary {
    pub clips: HashMap<String, IfpClip>,
}

impl IfpPoseLibrary {
    pub fn insert_archive(&mut self, archive: IfpFullArchive) -> usize {
        let count = archive.clips.len();
        for clip in archive.clips {
            self.clips.insert(clip.name.to_ascii_lowercase(), clip);
        }
        count
    }

    pub fn is_empty(&self) -> bool {
        self.clips.is_empty()
    }

    /// Evaluate interpolated bone transforms for `clip_name` at `time_sec`.
    pub fn sample_clip(
        &self,
        clip_name: &str,
        time_sec: f32,
        looped: bool,
    ) -> HashMap<i32, BonePose> {
        let mut out = HashMap::new();
        let Some(clip) = self.clips.get(&clip_name.to_ascii_lowercase()) else {
            return out;
        };

        let sample_t = if clip.duration_seconds > 1e-4 {
            if looped {
                time_sec.rem_euclid(clip.duration_seconds)
            } else {
                time_sec.clamp(0.0, clip.duration_seconds)
            }
        } else {
            0.0
        };

        for bone in &clip.bones {
            if bone.keyframes.is_empty() {
                continue;
            }
            if bone.keyframes.len() == 1 || sample_t <= bone.keyframes[0].time {
                let k = &bone.keyframes[0];
                out.insert(
                    bone.bone_id,
                    BonePose {
                        rotation: Quat::from_array(k.rotation).normalize(),
                        translation: k.translation.map(Vec3::from_array),
                    },
                );
                continue;
            }

            let mut k0 = &bone.keyframes[0];
            let mut k1 = bone.keyframes.last().unwrap_or(k0);
            for win in bone.keyframes.windows(2) {
                if sample_t >= win[0].time && sample_t <= win[1].time {
                    k0 = &win[0];
                    k1 = &win[1];
                    break;
                }
            }
            if sample_t >= k1.time {
                k0 = k1;
            }

            let span = (k1.time - k0.time).max(1e-5);
            let alpha = ((sample_t - k0.time) / span).clamp(0.0, 1.0);
            let q0 = Quat::from_array(k0.rotation).normalize();
            let mut q1 = Quat::from_array(k1.rotation).normalize();
            if q0.dot(q1) < 0.0 {
                q1 = -q1;
            }
            let rotation = q0.slerp(q1, alpha).normalize();
            let translation = match (k0.translation, k1.translation) {
                (Some(t0), Some(t1)) => {
                    Some(Vec3::from_array(t0).lerp(Vec3::from_array(t1), alpha))
                }
                (Some(t0), None) => Some(Vec3::from_array(t0)),
                (None, Some(t1)) => Some(Vec3::from_array(t1)),
                (None, None) => None,
            };
            out.insert(
                bone.bone_id,
                BonePose {
                    rotation,
                    translation,
                },
            );
        }

        out
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationController {
    pub current: AnimState,
    pub previous: AnimState,
    pub state_time: f32,
    pub blend_alpha: f32,
    pub blend_duration: f32,
}

impl Default for AnimationController {
    fn default() -> Self {
        Self {
            current: AnimState::Idle,
            previous: AnimState::Idle,
            state_time: 0.0,
            blend_alpha: 1.0,
            blend_duration: 0.15,
        }
    }
}

impl AnimationController {
    pub fn transition(&mut self, next: AnimState) {
        if self.current != next {
            self.previous = self.current;
            self.current = next;
            self.state_time = 0.0;
            self.blend_alpha = 0.0;
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.state_time += dt;
        if self.blend_alpha < 1.0 {
            self.blend_alpha = (self.blend_alpha + dt / self.blend_duration).min(1.0);
        }
    }

    /// Map any `AnimState` to original `ped.ifp` animation clip identifier.
    pub fn clip_name_for(state: AnimState) -> &'static str {
        match state {
            AnimState::Idle => "idle_stance",
            AnimState::Walk => "walk_civi",
            AnimState::Run => "run_civi",
            AnimState::Sprint => "sprint_civi",
            AnimState::Jump => "jump_launch",
            AnimState::Fall => "fall_fall",
            AnimState::Crouch => "crouch_roll_l",
            AnimState::Swim => "swim_breast",
            AnimState::MeleePunch => "fighta_1",
            AnimState::AimWeapon => "gun_stand",
            AnimState::Drive => "car_sit",
            AnimState::EnterVehicle => "car_open_lhs",
        }
    }

    /// Map active state to original `ped.ifp` animation clip identifier.
    pub fn ifp_clip_name(&self) -> &'static str {
        Self::clip_name_for(self.current)
    }

    /// Sample and blend current/previous `ped.ifp` bone poses.
    pub fn sample_poses(&self, lib: &IfpPoseLibrary) -> HashMap<i32, BonePose> {
        let curr_looped = !matches!(self.current, AnimState::Jump | AnimState::EnterVehicle);
        let mut curr = lib.sample_clip(
            Self::clip_name_for(self.current),
            self.state_time,
            curr_looped,
        );
        if self.blend_alpha < 0.999 && self.previous != self.current {
            let prev = lib.sample_clip(Self::clip_name_for(self.previous), self.state_time, true);
            let alpha = self.blend_alpha.clamp(0.0, 1.0);
            for (bone_id, curr_pose) in &mut curr {
                if let Some(prev_pose) = prev.get(bone_id) {
                    let mut target_rot = curr_pose.rotation;
                    if prev_pose.rotation.dot(target_rot) < 0.0 {
                        target_rot = -target_rot;
                    }
                    curr_pose.rotation = prev_pose.rotation.slerp(target_rot, alpha).normalize();
                }
            }
        }
        curr
    }
}

/// GTA:SA `eBoneTag::BONE_R_HAND` used for held weapon attachment matrices.
pub const BONE_R_HAND: i32 = 24;

fn compute_frame_world_matrices(frames: &[DffFrame], poses: &HashMap<i32, BonePose>) -> Vec<Mat4> {
    let locals: Vec<Mat4> = frames
        .iter()
        .enumerate()
        .map(|(idx, f)| {
            let base_pos = Vec3::from_array(f.position);
            if idx > 0
                && let Some(node_id) = f.hanim_node_id
                && let Some(pose) = poses.get(&node_id)
            {
                return Mat4::from_rotation_translation(pose.rotation, base_pos);
            }
            if idx == 0
                && let Some(pose) = poses.get(&0)
                && let Some(tr) = pose.translation
            {
                // Apply subtle vertical bob from root while keeping locomotion handled by physics
                return Mat4::from_cols(
                    Vec3::from_array(f.rotation[0]).extend(0.0),
                    Vec3::from_array(f.rotation[1]).extend(0.0),
                    Vec3::from_array(f.rotation[2]).extend(0.0),
                    Vec3::new(base_pos.x, base_pos.y, base_pos.z + tr.z * 0.25).extend(1.0),
                );
            }
            Mat4::from_cols(
                Vec3::from_array(f.rotation[0]).extend(0.0),
                Vec3::from_array(f.rotation[1]).extend(0.0),
                Vec3::from_array(f.rotation[2]).extend(0.0),
                base_pos.extend(1.0),
            )
        })
        .collect();

    let mut worlds = Vec::with_capacity(frames.len());
    for idx in 0..frames.len() {
        let mut acc = locals[idx];
        let mut cur = frames[idx].parent;
        let mut depth = 0usize;
        while cur >= 0 && (cur as usize) < frames.len() && depth < frames.len() {
            let p_idx = cur as usize;
            acc = locals[p_idx] * acc;
            cur = frames[p_idx].parent;
            depth += 1;
        }
        worlds.push(acc);
    }
    worlds
}

/// Evaluate the rig-space world transform of a specific GTA:SA bone (`BONE_R_HAND`, etc.).
pub fn compute_bone_world_matrix(
    frames: &[DffFrame],
    bone_id: i32,
    poses: &HashMap<i32, BonePose>,
) -> Option<Mat4> {
    if frames.is_empty() {
        return None;
    }
    let idx = frames
        .iter()
        .position(|f| f.hanim_node_id == Some(bone_id))?;
    let worlds = compute_frame_world_matrices(frames, poses);
    worlds.get(idx).copied()
}

/// Evaluate world-space-to-bind skinning matrices for each bone of a RenderWare `DffSkin`.
pub fn compute_skin_matrices(
    frames: &[DffFrame],
    skin: &DffSkin,
    poses: &HashMap<i32, BonePose>,
) -> Vec<Mat4> {
    if frames.is_empty() || skin.inv_bind_matrices.is_empty() {
        return Vec::new();
    }

    let worlds = compute_frame_world_matrices(frames, poses);

    let mut frame_by_node_id = HashMap::new();
    for (fi, f) in frames.iter().enumerate() {
        if let Some(nid) = f.hanim_node_id {
            frame_by_node_id.entry(nid).or_insert(fi);
        }
    }

    let hanim_order = frames
        .iter()
        .find(|f| !f.hanim_nodes.is_empty())
        .map(|f| f.hanim_nodes.as_slice())
        .unwrap_or(&[]);

    skin.inv_bind_matrices
        .iter()
        .enumerate()
        .map(|(bone_idx, inv)| {
            let inv_mat = Mat4::from_cols_array(inv);
            let frame_idx = hanim_order
                .get(bone_idx)
                .and_then(|hn| frame_by_node_id.get(&hn.node_id).copied())
                .or_else(|| frame_by_node_id.get(&(bone_idx as i32)).copied());
            if let Some(fi) = frame_idx {
                worlds[fi] * inv_mat
            } else {
                Mat4::IDENTITY
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gtasarust_gta_formats::ifp::{IfpBoneTrack, IfpKeyframe};
    use gtasarust_gta_formats::rw::DffHAnimNode;

    #[test]
    fn animation_blends_on_transition() {
        let mut ctrl = AnimationController::default();
        ctrl.transition(AnimState::Sprint);
        assert_eq!(ctrl.previous, AnimState::Idle);
        assert_eq!(ctrl.blend_alpha, 0.0);
        ctrl.update(0.15);
        assert!((ctrl.blend_alpha - 1.0).abs() < 0.001);
        assert_eq!(ctrl.ifp_clip_name(), "sprint_civi");
    }

    #[test]
    fn samples_ifp_keyframes_and_computes_skin_matrices() {
        let mut lib = IfpPoseLibrary::default();
        lib.insert_archive(IfpFullArchive {
            version: "ANP3".into(),
            package_name: "ped".into(),
            clips: vec![IfpClip {
                name: "idle_stance".into(),
                duration_seconds: 1.0,
                bones: vec![IfpBoneTrack {
                    name: "Head".into(),
                    bone_id: 5,
                    frame_type: 3,
                    keyframes: vec![
                        IfpKeyframe {
                            time: 0.0,
                            rotation: Quat::IDENTITY.to_array(),
                            translation: None,
                        },
                        IfpKeyframe {
                            time: 1.0,
                            rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2).to_array(),
                            translation: None,
                        },
                    ],
                }],
            }],
        });

        let poses = lib.sample_clip("IDLE_STANCE", 0.5, true);
        let pose = poses.get(&5).expect("bone 5 sampled");
        let (_, angle) = pose.rotation.to_axis_angle();
        assert!((angle - std::f32::consts::FRAC_PI_4).abs() < 0.01);

        let frames = vec![
            DffFrame {
                rotation: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
                position: [0.0, 0.0, 0.0],
                parent: -1,
                flags: 0,
                name: "Root".into(),
                hanim_node_id: Some(0),
                hanim_nodes: vec![DffHAnimNode {
                    node_id: 5,
                    node_index: 0,
                    flags: 0,
                }],
            },
            DffFrame {
                rotation: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
                position: [0.0, 0.0, 0.7],
                parent: 0,
                flags: 0,
                name: "Head".into(),
                hanim_node_id: Some(5),
                hanim_nodes: Vec::new(),
            },
        ];
        let skin = DffSkin {
            num_bones: 1,
            bone_indices: vec![],
            bone_weights: vec![],
            inv_bind_matrices: vec![Mat4::IDENTITY.to_cols_array()],
        };
        let mats = compute_skin_matrices(&frames, &skin, &poses);
        assert_eq!(mats.len(), 1);
        assert!((mats[0].w_axis.z - 0.7).abs() < 1e-4);

        let bone_mat =
            compute_bone_world_matrix(&frames, 5, &poses).expect("bone 5 matrix should exist");
        assert!((bone_mat.w_axis.z - 0.7).abs() < 1e-4);
    }
}
