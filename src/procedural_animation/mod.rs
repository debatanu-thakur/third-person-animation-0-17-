//! Procedural Animation System
//!
//! Overgrowth-style animation using 13 keyframe poses blended based on:
//! - Character velocity (speed + direction)
//! - Acceleration/deceleration
//! - Contact state (grounded, airborne)
//! - Terrain angle
//! - Foot phase (which foot is forward)

use bevy::prelude::*;

pub mod pose;
pub mod pose_library;
pub mod blending;
pub mod extraction;
pub mod stride;

pub use pose::{Pose, BoneTransform, PoseMetadata, PoseAssetLoader};
pub use pose_library::*;
pub use blending::*;
pub use extraction::*;
pub use stride::*;

/// Plugin for procedural animation system
pub struct ProceduralAnimationPlugin;

impl Plugin for ProceduralAnimationPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register types
            .register_type::<ProceduralAnimationController>()
            .register_type::<PoseBlendState>()
            // Initialize Pose asset type
            .init_asset::<Pose>()
            .init_asset_loader::<PoseAssetLoader>()
            // Extraction systems (only run when EXTRACT_POSES env var is set)
            .add_systems(Startup, extraction::setup_extraction_mode)
            .add_systems(Update, extraction::extract_poses_from_animations)
            // Animation systems
            .add_systems(Update, (
                blending::update_blend_weights,
                blending::apply_pose_blending,
            ).chain());
    }
}

/// Marker component for entities using procedural animation
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct ProceduralAnimationController {
    /// Whether the system is enabled
    pub enabled: bool,
    /// Current blend state
    pub blend_state: PoseBlendState,
}

/// Current blending state for procedural animation
#[derive(Reflect, Default, Clone)]
pub struct PoseBlendState {
    /// Active poses and their blend weights (sum should = 1.0)
    pub active_poses: Vec<(PoseId, f32)>,
    /// Current velocity magnitude (m/s)
    pub velocity: f32,
    /// Current acceleration (m/s²)
    pub acceleration: Vec3,
    /// Contact state
    pub contact_state: ContactState,
    /// Current foot phase (0.0 - 1.0, repeating)
    pub foot_phase: f32,
    /// Stride length (meters)
    pub stride_length: f32,
}

/// Contact state for character
#[derive(Reflect, Default, Clone, Copy, PartialEq, Debug)]
pub enum ContactState {
    #[default]
    Grounded,
    Airborne,
    Landing,
}

/// Identifier for each of the 13 keyframe poses
#[derive(Reflect, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PoseId {
    Idle,
    WalkLeftFootForward,
    WalkRightFootForward,
    RunLeftFootForward,
    RunRightFootForward,
    JumpTakeoff,
    JumpAirborne,
    JumpLanding,
    RollLeft,
    RollRight,
    AttackPunch,
    AttackKick,
    Crouch,
}

impl PoseId {
    /// Get all pose IDs in order
    pub fn all() -> [PoseId; 13] {
        use PoseId::*;
        [
            Idle,
            WalkLeftFootForward,
            WalkRightFootForward,
            RunLeftFootForward,
            RunRightFootForward,
            JumpTakeoff,
            JumpAirborne,
            JumpLanding,
            RollLeft,
            RollRight,
            AttackPunch,
            AttackKick,
            Crouch,
        ]
    }

    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            PoseId::Idle => "Idle",
            PoseId::WalkLeftFootForward => "Walk Left",
            PoseId::WalkRightFootForward => "Walk Right",
            PoseId::RunLeftFootForward => "Run Left",
            PoseId::RunRightFootForward => "Run Right",
            PoseId::JumpTakeoff => "Jump Takeoff",
            PoseId::JumpAirborne => "Jump Airborne",
            PoseId::JumpLanding => "Jump Landing",
            PoseId::RollLeft => "Roll Left",
            PoseId::RollRight => "Roll Right",
            PoseId::AttackPunch => "Attack Punch",
            PoseId::AttackKick => "Attack Kick",
            PoseId::Crouch => "Crouch",
        }
    }
}
