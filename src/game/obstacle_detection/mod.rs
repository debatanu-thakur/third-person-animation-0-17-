pub mod detection;
use bevy::prelude::*;

use crate::{game::obstacle_detection::detection::*, screens::Screen};
// ============================================================================
// PLUGIN
// ============================================================================

pub(super) fn plugin(app: &mut App) {
    // Insert config resource
    app.init_resource::<ObstacleDetectionConfig>();

    // Add detection systems
    app.add_systems(
        FixedUpdate,
        (
            detect_obstacles,
            update_parkour_capabilities,
            trigger_parkour_actions,
            init_root_motion_tracker,          // Initialize tracker when parkour starts
            control_tnua_during_parkour,
            control_rigidbody_during_parkour,  // Make kinematic during parkour
            extract_and_apply_root_motion,     // Extract root motion from animation
            // Note: Time-based completion removed - using event-driven completion
            // Animation events (on_parkour_blend_to_idle observer) handle completion
            apply_ik_targets,
        )
            .chain()
            .run_if(in_state(Screen::Gameplay)),
    );
}
