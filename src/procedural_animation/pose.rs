//! Pose data structures for storing keyframe poses

use bevy::{
    asset::{AssetLoader, AsyncReadExt},
    prelude::*,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single keyframe pose containing transforms for all bones
#[derive(Clone, Debug, Serialize, Deserialize, Asset, TypePath)]
pub struct Pose {
    /// Name of this pose
    pub name: String,
    /// Bone transforms (bone name -> local transform)
    pub bone_transforms: HashMap<String, BoneTransform>,
    /// Optional metadata
    pub metadata: PoseMetadata,
}

/// Transform data for a single bone
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoneTransform {
    /// Local translation
    pub translation: Vec3,
    /// Local rotation (quaternion)
    pub rotation: Quat,
    /// Local scale
    pub scale: Vec3,
}

impl From<Transform> for BoneTransform {
    fn from(transform: Transform) -> Self {
        Self {
            translation: transform.translation,
            rotation: transform.rotation,
            scale: transform.scale,
        }
    }
}

impl From<BoneTransform> for Transform {
    fn from(bone_transform: BoneTransform) -> Self {
        Transform {
            translation: bone_transform.translation,
            rotation: bone_transform.rotation,
            scale: bone_transform.scale,
        }
    }
}

/// Metadata about a pose
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PoseMetadata {
    /// Source animation this was extracted from
    pub source_animation: Option<String>,
    /// Frame number in source animation
    pub source_frame: Option<f32>,
    /// Time in seconds in source animation
    pub source_time: Option<f32>,
    /// Notes about this pose
    pub notes: Option<String>,
}

impl Pose {
    /// Create a new empty pose
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            bone_transforms: HashMap::new(),
            metadata: PoseMetadata::default(),
        }
    }

    /// Add a bone transform to this pose
    pub fn with_bone(mut self, bone_name: impl Into<String>, transform: Transform) -> Self {
        self.bone_transforms.insert(bone_name.into(), transform.into());
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: PoseMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Blend this pose with another pose
    pub fn blend(&self, other: &Pose, weight: f32) -> Pose {
        let mut result = Pose::new(format!("{}_{}_blend", self.name, other.name));

        // Blend all bones that exist in both poses
        for (bone_name, transform_a) in &self.bone_transforms {
            if let Some(transform_b) = other.bone_transforms.get(bone_name) {
                let blended = BoneTransform {
                    translation: transform_a.translation.lerp(transform_b.translation, weight),
                    rotation: transform_a.rotation.slerp(transform_b.rotation, weight),
                    scale: transform_a.scale.lerp(transform_b.scale, weight),
                };
                result.bone_transforms.insert(bone_name.clone(), blended);
            } else {
                // Bone only exists in pose A, use it directly
                result.bone_transforms.insert(bone_name.clone(), transform_a.clone());
            }
        }

        // Add bones that only exist in pose B
        for (bone_name, transform_b) in &other.bone_transforms {
            if !self.bone_transforms.contains_key(bone_name) {
                result.bone_transforms.insert(bone_name.clone(), transform_b.clone());
            }
        }

        result
    }

    /// Blend multiple poses with weights
    pub fn blend_multiple(poses: &[(Pose, f32)]) -> Option<Pose> {
        if poses.is_empty() {
            return None;
        }

        if poses.len() == 1 {
            return Some(poses[0].0.clone());
        }

        // Start with first pose
        let mut result = poses[0].0.clone();
        let mut total_weight = poses[0].1;

        // Blend in remaining poses
        for (pose, weight) in &poses[1..] {
            result = result.blend(pose, *weight / (total_weight + weight));
            total_weight += weight;
        }

        Some(result)
    }
}

/// Asset loader for Pose RON files
#[derive(Default)]
pub struct PoseAssetLoader;

impl AssetLoader for PoseAssetLoader {
    type Asset = Pose;
    type Settings = ();
    type Error = anyhow::Error;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let pose: Pose = ron::de::from_bytes(&bytes)?;
        Ok(pose)
    }

    fn extensions(&self) -> &[&str] {
        &["pose.ron"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pose_blend() {
        let pose_a = Pose::new("A")
            .with_bone("LeftFoot", Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)));

        let pose_b = Pose::new("B")
            .with_bone("LeftFoot", Transform::from_translation(Vec3::new(1.0, 1.0, 1.0)));

        let blended = pose_a.blend(&pose_b, 0.5);

        let left_foot = blended.bone_transforms.get("LeftFoot").unwrap();
        assert_eq!(left_foot.translation, Vec3::new(0.5, 0.5, 0.5));
    }
}
