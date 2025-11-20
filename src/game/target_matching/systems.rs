//! Core systems for target matching

use bevy::prelude::*;

use super::{
    components::*,
    curve_generator::{generate_target_curve, EasingFunction},
    ik_integration::setup_ik_for_target_match,
};

/// Handle new target match requests
pub fn handle_target_match_requests(
    mut commands: Commands,
    mut requests: Query<
        (Entity, &TargetMatchRequest, &mut TargetMatchingState, Option<&BoneMap>),
        Changed<TargetMatchRequest>,  // Changed instead of Added - triggers every update!
    >,
    ik_constraints: Query<&crate::ik::IkConstraint>,
    mut ik_targets: Query<&mut Transform>,
    time: Res<Time>,
) {
    for (entity, request, mut state, bone_map) in requests.iter_mut() {
        info!(
            "Processing target match request for {:?} to position {:?}",
            request.bone, request.target_position
        );

        // Initialize matching state
        *state = TargetMatchingState::Matching {
            request: request.clone(),
            start_time: time.elapsed_secs(),
            curve_handle: None,
        };

        // Setup or update IK constraint for this target
        if let Some(bone_map) = bone_map {
            if let Some(bone_entity) = bone_map.get(request.bone) {
                // Check if IK constraint already exists on this bone
                if let Ok(ik_constraint) = ik_constraints.get(bone_entity) {
                    // Update existing IK target position
                    if let Ok(mut target_transform) = ik_targets.get_mut(ik_constraint.target) {
                        target_transform.translation = request.target_position;
                        info!("✓ Updated IK target for {:?} to {:?}", request.bone, request.target_position);
                    } else {
                        warn!("IK target entity not found for {:?}", request.bone);
                    }
                } else {
                    // Create new IK constraint (first time)
                    if let Some(ik_target) = setup_ik_for_target_match(&mut commands, request, bone_map, entity) {
                        info!("✓ IK constraint created for {:?} with target entity {:?}", request.bone, ik_target);
                    } else {
                        warn!("Failed to create IK constraint for {:?}", request.bone);
                    }
                }
            }
        } else {
            warn!("No BoneMap available for entity {:?}, cannot setup IK", entity);
        }

        info!("Target matching initiated/updated for entity {:?}", entity);
    }
}

/// Update active target matching operations
pub fn update_active_matching(
    mut commands: Commands,
    mut matching: Query<(Entity, &mut TargetMatchingState, &TargetMatchRequest, &BoneMap)>,
    mut bone_transforms: Query<&mut Transform>,
    time: Res<Time>,
) {
    for (entity, mut state, request, bone_map) in matching.iter_mut() {
        if let TargetMatchingState::Matching { start_time, .. } = *state {
            let elapsed = time.elapsed_secs() - start_time;
            let duration = request.match_duration();

            // Get the bone entity we need to move
            if let Some(bone_entity) = bone_map.get(request.bone) {
                if let Ok(mut bone_transform) = bone_transforms.get_mut(bone_entity) {
                    // Calculate interpolation progress (0.0 to 1.0)
                    let t = (elapsed / duration).clamp(0.0, 1.0);

                    // Use smooth easing for natural movement
                    let t_eased = ease_in_out_cubic(t);

                    let target_pos = request.target_position;
                    let current_pos = bone_transform.translation;

                    // Lerp toward target position aggressively for visibility
                    bone_transform.translation = current_pos.lerp(target_pos, t_eased * 0.8);

                    if elapsed < 0.1 || (elapsed % 0.5) < 0.016 {  // Log occasionally
                        info!(
                            "Moving {:?} bone from {:?} toward {:?} (progress: {:.2}, t_eased: {:.2})",
                            request.bone,
                            current_pos,
                            target_pos,
                            t,
                            t_eased
                        );
                    }
                } else {
                    warn!("Could not get mutable Transform for bone entity {:?}", bone_entity);
                }
            } else {
                warn!("Bone {:?} not found in BoneMap", request.bone);
            }

            // Check if matching duration has elapsed
            if elapsed >= duration {
                info!("Target matching completed for {:?}", request.bone);

                *state = TargetMatchingState::Complete {
                    bone: request.bone,
                };

                // Remove the request component
                commands.entity(entity).remove::<TargetMatchRequest>();
            }
        }
    }
}

/// Smooth cubic easing function for natural movement
fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

/// Debug visualization of targets and bones
pub fn debug_visualize_targets(
    mut gizmos: Gizmos,
    debug_settings: Option<Res<TargetMatchDebugSettings>>,
    matching: Query<(&TargetMatchRequest, &TargetMatchingState)>,
    bones: Query<(&GlobalTransform, &Name), Without<TargetMatchRequest>>,
) {
    let Some(settings) = debug_settings else {
        return;
    };

    if !settings.show_targets && !settings.show_bones {
        return;
    }

    // Draw target positions
    if settings.show_targets {
        for (request, state) in matching.iter() {
            if state.is_matching() {
                // Draw sphere at target position
                gizmos.sphere(
                    Isometry3d::from_translation(request.target_position),
                    settings.gizmo_size,
                    settings.target_color,
                );

                // Draw label
                // Note: Gizmos don't support text yet, but we can draw a line pointing up
                gizmos.line(
                    request.target_position,
                    request.target_position + Vec3::Y * 0.5,
                    settings.target_color,
                );
            }
        }
    }

    // Draw bone positions (if enabled)
    if settings.show_bones {
        for (transform, name) in bones.iter() {
            // Only show bones that might be targets
            if name.as_str().contains("Foot")
                || name.as_str().contains("Hand")
                || name.as_str().contains("Head")
            {
                gizmos.sphere(
                    Isometry3d::from_translation(transform.translation()),
                    settings.gizmo_size * 0.5,
                    Color::srgb(0.0, 1.0, 0.0),
                );
            }
        }
    }
}

/// System to build bone map from scene hierarchy
/// Searches through the character's children to find bone entities
pub fn build_bone_map(
    mut commands: Commands,
    characters: Query<Entity, (With<TargetMatchEnabled>, Without<BoneMap>)>,
    children_query: Query<&Children>,
    names: Query<&Name>,
) {
    for character_entity in characters.iter() {
        info!("Attempting to build bone map for entity {:?}", character_entity);

        let mut bone_map = BoneMap::default();
        let mut bones_found = 0;

        // Recursively search all descendants for bone entities
        let mut to_search = vec![character_entity];
        let mut searched_count = 0;

        while let Some(entity) = to_search.pop() {
            searched_count += 1;

            // Check if this entity has a name that matches a bone
            if let Ok(name) = names.get(entity) {
                if let Some(target_bone) = name_to_target_bone(name.as_str()) {
                    bone_map.insert(target_bone, entity);
                    bones_found += 1;
                    info!("✓ Found bone '{}' -> {:?} (entity {:?})", name, target_bone, entity);
                }
            }

            // Add children to search queue
            if let Ok(children) = children_query.get(entity) {
                to_search.extend(children.iter());
            }
        }

        info!("Searched {} entities, found {} bones", searched_count, bones_found);

        if !bone_map.bones.is_empty() {
            commands.entity(character_entity).insert(bone_map);
            info!(
                "✓ Built bone map for entity {:?} with {} bones",
                character_entity,
                bones_found
            );
        } else {
            warn!(
                "⚠️  No bones found for entity {:?} after searching {} entities. \
                Make sure the character scene is loaded and has bones named 'mixamorig12:LeftFoot', etc.",
                character_entity,
                searched_count
            );
        }
    }
}

/// System to retry building bone map if it's empty (scene might load later)
pub fn retry_bone_map_if_empty(
    mut commands: Commands,
    mut characters: Query<(Entity, &mut BoneMap), With<TargetMatchEnabled>>,
    children_query: Query<&Children>,
    names: Query<&Name>,
) {
    for (character_entity, mut bone_map) in characters.iter_mut() {
        // Only retry if bone map is empty
        if !bone_map.bones.is_empty() {
            continue;
        }

        trace!("Retrying bone map build for entity {:?}", character_entity);

        let mut bones_found = 0;
        let mut to_search = vec![character_entity];

        while let Some(entity) = to_search.pop() {
            if let Ok(name) = names.get(entity) {
                if let Some(target_bone) = name_to_target_bone(name.as_str()) {
                    bone_map.insert(target_bone, entity);
                    bones_found += 1;
                    info!("✓ Found bone '{}' -> {:?} on retry", name, target_bone);
                }
            }

            if let Ok(children) = children_query.get(entity) {
                to_search.extend(children.iter());
            }
        }

        if bones_found > 0 {
            info!("✓ Bone map retry successful: found {} bones", bones_found);
        }
    }
}

/// Helper to convert bone name to TargetBone enum
///
/// Handles both prefixed ("mixamorig12:LeftFoot") and unprefixed ("LeftFoot") names
fn name_to_target_bone(name: &str) -> Option<TargetBone> {
    // Strip prefix if present
    let bone_name = if let Some((_prefix, suffix)) = name.split_once(':') {
        suffix
    } else {
        name
    };

    match bone_name {
        "LeftFoot" => Some(TargetBone::LeftFoot),
        "RightFoot" => Some(TargetBone::RightFoot),
        "LeftHand" => Some(TargetBone::LeftHand),
        "RightHand" => Some(TargetBone::RightHand),
        "Head" => Some(TargetBone::Head),
        "Hips" => Some(TargetBone::Hips),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_to_target_bone() {
        assert_eq!(name_to_target_bone("LeftFoot"), Some(TargetBone::LeftFoot));
        assert_eq!(name_to_target_bone("RightHand"), Some(TargetBone::RightHand));
        assert_eq!(name_to_target_bone("Unknown"), None);
    }
}
