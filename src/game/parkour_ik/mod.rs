use bevy::prelude::*;
use avian3d::prelude::*;
mod setup;
use crate::{
    game::{
        obstacle_detection::detection::*,
        parkour_ik::setup::*,
        player::Player
    }, ik::*, screens::Screen
};



// ============================================================================
// DEBUG LOGGING SYSTEM
// ============================================================================

/// Writes IK debug information to RON file for troubleshooting
pub fn write_ik_debug_info(
    parkour_query: Query<&ParkourController, With<Player>>,
    bone_query: Query<(Entity, &GlobalTransform, &Name)>,
    ik_constraint_query: Query<(Entity, &Name, &IkConstraint)>,
    left_foot_target_query: Query<&Transform, (With<LeftFootIkTarget>, Without<RightFootIkTarget>)>,
    right_foot_target_query: Query<&Transform, With<RightFootIkTarget>>,
    config: Res<LocomotionIkConfig>,
    time: Res<Time>,
) {
    // Only write once per second to avoid spam
    static mut LAST_WRITE: f32 = 0.0;
    let current_time = time.elapsed_secs();

    unsafe {
        if current_time - LAST_WRITE < 1.0 {
            return;
        }
        LAST_WRITE = current_time;
    }

    let mut debug_info = String::new();
    debug_info.push_str("(\n");
    debug_info.push_str(&format!("  timestamp: {},\n", current_time));

    // Config status
    debug_info.push_str("  config: (\n");
    debug_info.push_str(&format!("    enabled: {},\n", config.enabled));
    debug_info.push_str(&format!("    max_ground_distance: {},\n", config.max_ground_distance));
    debug_info.push_str(&format!("    foot_height_offset: {},\n", config.foot_height_offset));
    debug_info.push_str(&format!("    adjustment_strength: {},\n", config.adjustment_strength));
    debug_info.push_str("  ),\n");

    // Parkour state
    if let Ok(parkour) = parkour_query.single() {
        debug_info.push_str(&format!("  parkour_state: \"{:?}\",\n", parkour.state));
        let is_normal = !matches!(
            parkour.state,
            ParkourState::Vaulting | ParkourState::Climbing |
            ParkourState::Sliding | ParkourState::Hanging
        );
        debug_info.push_str(&format!("  ik_should_be_active: {},\n", is_normal));
    } else {
        debug_info.push_str("  parkour_state: \"Not Found\",\n");
        debug_info.push_str("  ik_should_be_active: false,\n");
    }

    // Bone entities
    debug_info.push_str("  bones_found: (\n");
    let mut found_left_foot = false;
    let mut found_right_foot = false;
    let mut left_foot_pos = Vec3::ZERO;
    let mut right_foot_pos = Vec3::ZERO;

    for (_entity, transform, name) in bone_query.iter() {
        match name.as_str() {
            "mixamorig12:LeftFoot" => {
                found_left_foot = true;
                left_foot_pos = transform.translation();
                debug_info.push_str(&format!("    left_foot: \"Found\",\n"));
                debug_info.push_str(&format!("    left_foot_pos: ({}, {}, {}),\n",
                    left_foot_pos.x, left_foot_pos.y, left_foot_pos.z));
            }
            "mixamorig12:RightFoot" => {
                found_right_foot = true;
                right_foot_pos = transform.translation();
                debug_info.push_str(&format!("    right_foot: \"Found\",\n"));
                debug_info.push_str(&format!("    right_foot_pos: ({}, {}, {}),\n",
                    right_foot_pos.x, right_foot_pos.y, right_foot_pos.z));
            }
            _ => {}
        }
    }

    if !found_left_foot {
        debug_info.push_str("    left_foot: \"Not Found\",\n");
    }
    if !found_right_foot {
        debug_info.push_str("    right_foot: \"Not Found\",\n");
    }
    debug_info.push_str("  ),\n");

    // IK Constraints
    debug_info.push_str("  ik_constraints: [\n");
    for (_entity, name, constraint) in ik_constraint_query.iter() {
        if name.as_str().contains("Foot") {
            debug_info.push_str("    (\n");
            debug_info.push_str(&format!("      bone: \"{}\",\n", name.as_str()));
            debug_info.push_str(&format!("      enabled: {},\n", constraint.enabled));
            debug_info.push_str(&format!("      chain_length: {},\n", constraint.chain_length));
            debug_info.push_str(&format!("      iterations: {},\n", constraint.iterations));
            debug_info.push_str("    ),\n");
        }
    }
    debug_info.push_str("  ],\n");

    // IK Targets
    debug_info.push_str("  ik_targets: (\n");
    if let Ok(transform) = left_foot_target_query.single() {
        debug_info.push_str(&format!("    left_foot_target: ({}, {}, {}),\n",
            transform.translation.x, transform.translation.y, transform.translation.z));
    } else {
        debug_info.push_str("    left_foot_target: \"Not Found\",\n");
    }

    if let Ok(transform) = right_foot_target_query.single() {
        debug_info.push_str(&format!("    right_foot_target: ({}, {}, {}),\n",
            transform.translation.x, transform.translation.y, transform.translation.z));
    } else {
        debug_info.push_str("    right_foot_target: \"Not Found\",\n");
    }
    debug_info.push_str("  ),\n");

    debug_info.push_str(")\n");

    // Write to file
    let _ = std::fs::write("assets/debug/ik_debug.ron", debug_info);
}

// ============================================================================
// PLUGIN
// ============================================================================

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<IkConfig>();
    app.init_resource::<LocomotionIkConfig>();
    app.add_plugins(InverseKinematicsPlugin);

    // IK setup happens once after player model loads
    app.add_systems(
        FixedUpdate,
        setup_ik_chains.run_if(in_state(Screen::Gameplay)),
    );

    // Parkour IK update systems run every frame during gameplay
    app.add_systems(
        FixedUpdate,
        (
            update_ik_targets_from_obstacles,
            toggle_ik_constraints,
            visualize_ik_targets,
        )
            .chain()
            .run_if(in_state(Screen::Gameplay)),
    );

    // Locomotion foot IK systems (for basic movement)
    app.add_systems(
        FixedUpdate,
        (
            update_locomotion_foot_ik,
            visualize_locomotion_foot_ik,
            write_ik_debug_info,  // Debug logging to RON file
        )
            .run_if(in_state(Screen::Gameplay)),
    );
}
