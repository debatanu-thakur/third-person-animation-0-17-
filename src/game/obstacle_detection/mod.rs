pub mod detection;
use bevy::prelude::*;

use crate::{game::obstacle_detection::detection::*, screens::Screen};
// ============================================================================
// PLUGIN
// ============================================================================

pub(super) fn plugin(app: &mut App) {
    // Register reflection types for animation events
    app.register_type::<ParkourState>();
    app.register_type::<ParkourAnimationComplete>();
    app.register_type::<ParkourAnimationBlendToIdle>();

    // Register observers for animation events
    app.add_observer(on_parkour_blend_to_idle);         // Blend start (early)
    app.add_observer(on_parkour_animation_complete);    // Completion (fallback)

    // Insert config resource
    app.init_resource::<ObstacleDetectionConfig>();

    // Add detection systems
    app.add_systems(
        FixedUpdate,
        (
            detect_obstacles,
            update_parkour_capabilities,
            trigger_parkour_actions,
            start_parkour_animation_tracking,
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
