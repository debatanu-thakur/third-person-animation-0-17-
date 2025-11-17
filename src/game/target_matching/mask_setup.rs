//! Animation mask configuration and setup

use bevy::{animation::AnimationTargetId, prelude::*, utils::HashMap};

use super::TargetBone;

/// Configuration for animation mask groups
#[derive(Resource, Debug, Clone)]
pub struct MaskGroupConfig {
    /// Map from bone name to mask group ID
    pub bone_to_group: HashMap<String, u32>,

    /// Character rig type
    pub rig_type: RigType,
}

/// Supported character rig types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RigType {
    Mixamo,
    Custom,
}

impl MaskGroupConfig {
    /// Create configuration for a Mixamo character rig
    ///
    /// Supports both prefixed (e.g., "mixamorig12:Hips") and unprefixed bone names
    pub fn for_mixamo() -> Self {
        Self::for_mixamo_with_prefix("mixamorig12")
    }

    /// Create configuration for a Mixamo character rig with custom prefix
    pub fn for_mixamo_with_prefix(prefix: &str) -> Self {
        let mut bone_to_group = HashMap::new();

        let add_bones = |map: &mut HashMap<String, u32>, bones: Vec<&str>, group: u32| {
            for bone in bones {
                // Add both with and without prefix for compatibility
                map.insert(bone.to_string(), group);
                map.insert(format!("{}:{}", prefix, bone), group);
            }
        };

        // Body group (0) - torso, spine, upper body
        add_bones(
            &mut bone_to_group,
            vec![
                "Hips", "Spine", "Spine1", "Spine2",
                "Neck", "Head", "HeadTop_End",
                "LeftShoulder", "RightShoulder",
            ],
            0,
        );

        // Left leg group (1)
        add_bones(
            &mut bone_to_group,
            vec!["LeftUpLeg", "LeftLeg", "LeftFoot", "LeftToeBase", "LeftToe_End"],
            1,
        );

        // Right leg group (2)
        add_bones(
            &mut bone_to_group,
            vec!["RightUpLeg", "RightLeg", "RightFoot", "RightToeBase", "RightToe_End"],
            2,
        );

        // Left arm group (3)
        add_bones(
            &mut bone_to_group,
            vec!["LeftArm", "LeftForeArm", "LeftHand"],
            3,
        );

        // Right arm group (4)
        add_bones(
            &mut bone_to_group,
            vec!["RightArm", "RightForeArm", "RightHand"],
            4,
        );

        // Head group (5) - just the head
        add_bones(&mut bone_to_group, vec!["Head"], 5);

        Self {
            bone_to_group,
            rig_type: RigType::Mixamo,
        }
    }

    /// Get the mask group for a bone name
    pub fn group_for_bone(&self, bone_name: &str) -> Option<u32> {
        self.bone_to_group.get(bone_name).copied()
    }

    /// Get all bones in a specific group
    pub fn bones_in_group(&self, group_id: u32) -> Vec<String> {
        self.bone_to_group
            .iter()
            .filter(|(_, &g)| g == group_id)
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Get the mask bitfield to affect only specific groups
    ///
    /// # Example
    /// ```ignore
    /// // Mask that only affects body (group 0)
    /// let mask = config.mask_for_groups(&[0]);  // 0b000001
    ///
    /// // Mask that affects body and left leg (groups 0 and 1)
    /// let mask = config.mask_for_groups(&[0, 1]);  // 0b000011
    /// ```
    pub fn mask_for_groups(&self, groups: &[u32]) -> u32 {
        let mut mask = 0u32;
        for &group in groups {
            mask |= 1 << group;
        }
        mask
    }

    /// Get the inverse mask (affects everything EXCEPT these groups)
    pub fn inverse_mask_for_groups(&self, groups: &[u32]) -> u32 {
        let forward_mask = self.mask_for_groups(groups);
        !forward_mask
    }

    /// Get mask that excludes a specific bone's group
    pub fn mask_excluding_bone(&self, bone: TargetBone) -> u32 {
        self.inverse_mask_for_groups(&[bone.mask_group()])
    }
}

/// System to automatically assign bones to mask groups
pub fn setup_animation_masks(
    mut commands: Commands,
    config: Option<Res<MaskGroupConfig>>,
    targets: Query<(Entity, &AnimationTargetId, &Name), Added<AnimationTargetId>>,
) {
    let Some(config) = config else {
        return;
    };

    for (entity, target_id, name) in targets.iter() {
        // Try to find this bone in our configuration
        if let Some(group_id) = config.group_for_bone(name.as_str()) {
            info!("Assigned bone '{}' to mask group {}", name, group_id);
            // The mask group assignment will happen when building the animation graph
            // We just track it here for debugging
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mixamo_config() {
        let config = MaskGroupConfig::for_mixamo();

        assert_eq!(config.group_for_bone("LeftFoot"), Some(1));
        assert_eq!(config.group_for_bone("RightFoot"), Some(2));
        assert_eq!(config.group_for_bone("Hips"), Some(0));
    }

    #[test]
    fn test_mask_generation() {
        let config = MaskGroupConfig::for_mixamo();

        // Only group 0 (body)
        assert_eq!(config.mask_for_groups(&[0]), 0b000001);

        // Groups 0 and 1 (body + left leg)
        assert_eq!(config.mask_for_groups(&[0, 1]), 0b000011);

        // All groups
        assert_eq!(config.mask_for_groups(&[0, 1, 2, 3, 4, 5]), 0b111111);
    }

    #[test]
    fn test_inverse_mask() {
        let config = MaskGroupConfig::for_mixamo();

        // Exclude left leg (group 1)
        let mask = config.inverse_mask_for_groups(&[1]);
        // Should affect everything except group 1
        assert_eq!(mask & 0b000010, 0); // Group 1 not set
        assert_ne!(mask & 0b000001, 0); // Group 0 is set
    }
}
