//! Animation Target Matching Plugin
//!
//! This plugin provides Unity-style animation target matching for Bevy 0.17,
//! allowing precise control over where character body parts land during animations.
//!
//! # Features
//!
//! - Animation masking for selective bone control
//! - Procedural curve generation for target positions
//! - IK integration for natural limb movement
//! - Reusable across different character rigs
//!
//! # Usage
//!
//! ```rust,ignore
//! app.add_plugins(TargetMatchingPlugin);
//! app.insert_resource(MaskGroupConfig::for_mixamo());
//!
//! // Request target matching
//! commands.entity(player).insert(TargetMatchRequest {
//!     bone: TargetBone::LeftFoot,
//!     target_position: ledge_position,
//!     match_window: (0.0, 0.8),
//!     animation_duration: 1.2,
//! });
//! ```

mod components;
mod curve_generator;
mod ik_integration;
mod mask_setup;
mod systems;

pub use components::*;
pub use mask_setup::MaskGroupConfig;

use bevy::prelude::*;

/// The main target matching plugin
pub struct TargetMatchingPlugin;

impl Plugin for TargetMatchingPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register types for reflection
            .register_type::<TargetMatchRequest>()
            .register_type::<TargetMatchingState>()
            .register_type::<TargetBone>()

            // Add systems
            .add_systems(
                Update,
                (
                    systems::build_bone_map,  // Build bone map for new characters
                    systems::retry_bone_map_if_empty,  // Retry if scene wasn't loaded yet
                    systems::handle_target_match_requests,  // Creates IK constraints
                    // systems::update_active_matching,  // DISABLED: Conflicts with IK solver
                    systems::debug_visualize_targets,
                )
                    .chain(),
            );

        info!("TargetMatchingPlugin initialized");
    }
}

/// Convenience trait for adding target matching to characters
pub trait TargetMatchingExt {
    /// Request a target match for a specific bone
    fn match_target(
        &mut self,
        bone: TargetBone,
        target_position: Vec3,
        animation_duration: f32,
    ) -> &mut Self;
}

impl TargetMatchingExt for EntityCommands<'_> {
    fn match_target(
        &mut self,
        bone: TargetBone,
        target_position: Vec3,
        animation_duration: f32,
    ) -> &mut Self {
        self.insert(TargetMatchRequest {
            bone,
            target_position,
            match_window: (0.0, 0.8), // Default: match from start to 80%
            animation_duration,
        });
        self
    }
}
