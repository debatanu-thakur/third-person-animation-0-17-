//! Integration with bevy_mod_inverse_kinematics

use bevy::prelude::*;
use bevy_mod_inverse_kinematics::{IkConstraint, IkPoleTarget};

use super::{BoneMap, TargetBone, TargetMatchRequest};

/// Setup IK constraint for a target matching request
pub fn setup_ik_for_target_match(
    commands: &mut Commands,
    request: &TargetMatchRequest,
    bone_map: &BoneMap,
    target_entity: Entity,
) -> Option<Entity> {
    let bone = request.bone;
    let bone_entity = bone_map.get(bone)?;

    // Create IK target entity at the target position
    let ik_target = commands
        .spawn((
            Name::new(format!("{:?}_IK_Target", bone)),
            Transform::from_translation(request.target_position),
            Visibility::default(),
        ))
        .id();

    // Setup pole target for natural bending (e.g., knee direction)
    let pole_target = if matches!(bone, TargetBone::LeftFoot | TargetBone::RightFoot) {
        // For legs, pole target should point forward (knee direction)
        let pole_pos = request.target_position + Vec3::new(0.0, 0.0, 1.0);
        Some(commands.spawn((
            Name::new(format!("{:?}_Pole_Target", bone)),
            Transform::from_translation(pole_pos),
            Visibility::default(),
        )).id())
    } else {
        None
    };

    // Apply IK constraint to the end bone
    let chain_length = bone.mixamo_chain().len();
    commands.entity(bone_entity).insert(IkConstraint {
        chain_length,
        iterations: 20,
        target: ik_target,
        pole_target,
        pole_angle: 0.0,
        enabled: true,
    });

    Some(ik_target)
}

/// Cleanup IK components after target matching completes
pub fn cleanup_ik_constraints(
    commands: &mut Commands,
    bone_map: &BoneMap,
    bone: TargetBone,
) {
    if let Some(bone_entity) = bone_map.get(bone) {
        commands.entity(bone_entity).remove::<IkConstraint>();
    }
}

/// Update IK target position during matching
pub fn update_ik_target(
    mut targets: Query<&mut Transform, With<IkPoleTarget>>,
    new_position: Vec3,
    target_entity: Entity,
) {
    if let Ok(mut transform) = targets.get_mut(target_entity) {
        transform.translation = new_position;
    }
}
