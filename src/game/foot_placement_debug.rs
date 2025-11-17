//! Debug commands for foot placement testing

use bevy::prelude::*;
use super::{FootPlacementEnabled, Player};
use crate::game::target_matching::{BoneMap, TargetBone};

/// Diagnostic system to check foot placement status
pub fn diagnose_foot_placement(
    players: Query<(Entity, &FootPlacementEnabled, &BoneMap), With<Player>>,
) {
    for (entity, foot_placement, bone_map) in players.iter() {
        info!("=== Foot Placement Diagnostics ===");
        info!("Player entity: {:?}", entity);
        info!("Raycast distance: {}", foot_placement.raycast_distance);
        info!("Foot offset: {}", foot_placement.foot_offset);
        info!("Update interval: {}", foot_placement.update_interval);
        info!("Min slope angle: {}", foot_placement.min_slope_angle);
        info!("Bone map size: {}", bone_map.bones.len());

        if bone_map.bones.is_empty() {
            warn!("⚠️  BoneMap is EMPTY - bones not discovered!");
        } else {
            info!("✓ BoneMap populated with bones:");
            for (bone_type, bone_entity) in &bone_map.bones {
                info!("  - {:?} -> {:?}", bone_type, bone_entity);
            }
        }

        if foot_placement.min_slope_angle > 0.0 {
            warn!("⚠️  min_slope_angle is {} - will only activate on slopes", foot_placement.min_slope_angle);
            info!("   Try setting FootPlacementEnabled {{ min_slope_angle: 0.0, ..default() }}");
        }
    }
}
