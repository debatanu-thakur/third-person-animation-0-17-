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
        Update,
        (
            detect_obstacles,
            update_parkour_capabilities,
            trigger_parkour_actions,
            start_parkour_animation_tracking,
            control_tnua_during_parkour,
            apply_parkour_root_motion,
            detect_parkour_animation_completion,
            apply_ik_targets,
        )
            .chain()
            .run_if(in_state(Screen::Gameplay)),
    );
}
