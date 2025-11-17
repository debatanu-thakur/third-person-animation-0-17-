//! Core components for target matching system

use bevy::prelude::*;
use std::time::Duration;

/// Which bone to match to a target position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum TargetBone {
    LeftFoot,
    RightFoot,
    LeftHand,
    RightHand,
    Head,
    Hips,
}

impl TargetBone {
    /// Get the typical Mixamo bone name for this target (without prefix)
    pub fn mixamo_name(&self) -> &'static str {
        match self {
            TargetBone::LeftFoot => "LeftFoot",
            TargetBone::RightFoot => "RightFoot",
            TargetBone::LeftHand => "LeftHand",
            TargetBone::RightHand => "RightHand",
            TargetBone::Head => "Head",
            TargetBone::Hips => "Hips",
        }
    }

    /// Get the full Mixamo bone name with prefix (e.g., "mixamorig12:LeftFoot")
    pub fn mixamo_full_name(&self, prefix: &str) -> String {
        format!("{}:{}", prefix, self.mixamo_name())
    }

    /// Get the bone chain for IK (from root to end effector) without prefix
    pub fn mixamo_chain(&self) -> Vec<&'static str> {
        match self {
            TargetBone::LeftFoot => vec!["LeftUpLeg", "LeftLeg", "LeftFoot"],
            TargetBone::RightFoot => vec!["RightUpLeg", "RightLeg", "RightFoot"],
            TargetBone::LeftHand => vec!["LeftArm", "LeftForeArm", "LeftHand"],
            TargetBone::RightHand => vec!["RightArm", "RightForeArm", "RightHand"],
            TargetBone::Head => vec!["Neck", "Head"],
            TargetBone::Hips => vec!["Hips"],
        }
    }

    /// Get the bone chain with prefix (e.g., "mixamorig12:LeftUpLeg")
    pub fn mixamo_chain_with_prefix(&self, prefix: &str) -> Vec<String> {
        self.mixamo_chain()
            .iter()
            .map(|name| format!("{}:{}", prefix, name))
            .collect()
    }

    /// Get the mask group ID for this bone
    pub fn mask_group(&self) -> u32 {
        match self {
            TargetBone::LeftFoot => 1,
            TargetBone::RightFoot => 2,
            TargetBone::LeftHand => 3,
            TargetBone::RightHand => 4,
            TargetBone::Head => 5,
            TargetBone::Hips => 0, // Body group
        }
    }
}

/// Request to match a bone to a target position during an animation
#[derive(Component, Debug, Clone, Reflect)]
pub struct TargetMatchRequest {
    /// Which bone to match
    pub bone: TargetBone,

    /// World-space target position
    pub target_position: Vec3,

    /// Time window for matching (normalized 0.0 to 1.0)
    /// (start_time, end_time) - e.g., (0.0, 0.8) means match from beginning to 80% through
    pub match_window: (f32, f32),

    /// Total duration of the animation in seconds
    pub animation_duration: f32,
}

impl TargetMatchRequest {
    /// Create a new target match request with default window
    pub fn new(bone: TargetBone, target_position: Vec3, animation_duration: f32) -> Self {
        Self {
            bone,
            target_position,
            match_window: (0.0, 0.8),
            animation_duration,
        }
    }

    /// Set custom match window
    pub fn with_window(mut self, start: f32, end: f32) -> Self {
        self.match_window = (start, end);
        self
    }

    /// Get the actual time range in seconds
    pub fn time_range(&self) -> (f32, f32) {
        (
            self.match_window.0 * self.animation_duration,
            self.match_window.1 * self.animation_duration,
        )
    }

    /// Get the duration of the matching window in seconds
    pub fn match_duration(&self) -> f32 {
        let (start, end) = self.time_range();
        end - start
    }
}

/// Current state of target matching for an entity
#[derive(Component, Debug, Clone, Reflect)]
pub enum TargetMatchingState {
    /// Not currently matching
    Idle,

    /// Actively matching to target
    Matching {
        request: TargetMatchRequest,
        start_time: f32,
        /// Handle to the generated curve animation clip
        curve_handle: Option<Handle<AnimationClip>>,
    },

    /// Matching completed
    Complete {
        bone: TargetBone,
    },
}

impl Default for TargetMatchingState {
    fn default() -> Self {
        Self::Idle
    }
}

impl TargetMatchingState {
    /// Check if currently matching
    pub fn is_matching(&self) -> bool {
        matches!(self, Self::Matching { .. })
    }

    /// Get the active request if matching
    pub fn active_request(&self) -> Option<&TargetMatchRequest> {
        match self {
            Self::Matching { request, .. } => Some(request),
            _ => None,
        }
    }
}

/// Marker component for entities that support target matching
#[derive(Component, Debug, Default, Reflect)]
pub struct TargetMatchEnabled;

/// Component storing bone entity references for quick lookup
#[derive(Component, Debug, Default)]
pub struct BoneMap {
    pub bones: std::collections::HashMap<TargetBone, Entity>,
}

impl BoneMap {
    /// Get the entity for a specific bone
    pub fn get(&self, bone: TargetBone) -> Option<Entity> {
        self.bones.get(&bone).copied()
    }

    /// Insert or update a bone entity
    pub fn insert(&mut self, bone: TargetBone, entity: Entity) {
        self.bones.insert(bone, entity);
    }
}

/// Debug visualization settings
#[derive(Resource, Debug, Clone, Reflect)]
pub struct TargetMatchDebugSettings {
    /// Show target positions as gizmos
    pub show_targets: bool,

    /// Show bone positions
    pub show_bones: bool,

    /// Show trajectories
    pub show_trajectories: bool,

    /// Color for target visualization
    pub target_color: Color,

    /// Size of debug spheres
    pub gizmo_size: f32,
}

impl Default for TargetMatchDebugSettings {
    fn default() -> Self {
        Self {
            show_targets: true,
            show_bones: true,
            show_trajectories: false,
            target_color: Color::srgb(1.0, 0.0, 0.0),
            gizmo_size: 0.1,
        }
    }
}
