//! Debug utilities for target matching system

use bevy::prelude::*;
use super::target_matching::{BoneMap, TargetBone};
use super::player::Player;

/// Diagnostic system to verify bone entities have Transform components
pub fn diagnose_bone_components(
    players: Query<(Entity, &BoneMap), With<Player>>,
    bone_query: Query<(
        Option<&Transform>,
        Option<&GlobalTransform>,
        Option<&Name>,
    )>,
) {
    for (player_entity, bone_map) in players.iter() {
        info!("=== Bone Component Diagnostics ===");
        info!("Player entity: {:?}", player_entity);
        info!("Checking components for {} bones", bone_map.bones.len());

        for (bone_type, bone_entity) in &bone_map.bones {
            if let Ok((transform, global_transform, name)) = bone_query.get(*bone_entity) {
                info!(
                    "  {:?} (entity {:?}, name: {:?})",
                    bone_type,
                    bone_entity,
                    name.map(|n| n.as_str())
                );
                info!(
                    "    - Has Transform: {}",
                    if transform.is_some() { "YES ✓" } else { "NO ✗" }
                );
                info!(
                    "    - Has GlobalTransform: {}",
                    if global_transform.is_some() { "YES ✓" } else { "NO ✗" }
                );
                if let Some(t) = transform {
                    info!("    - Local position: {:?}", t.translation);
                }
                if let Some(gt) = global_transform {
                    info!("    - Global position: {:?}", gt.translation());
                }
            } else {
                warn!("  {:?} - Could not query entity {:?}", bone_type, bone_entity);
            }
        }
    }
}
