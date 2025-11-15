use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single bone's pose data (position and rotation relative to parent)
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct BonePose {
    /// Bone name (e.g., "LeftHand", "RightHand", "Spine")
    pub bone_name: String,
    /// Position relative to parent bone
    pub position: Vec3,
    /// Rotation as quaternion
    pub rotation: Quat,
}

/// A complete pose at a specific time in the animation
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct KeyPose {
    /// Time in seconds from start of animation
    pub time: f32,
    /// Optional description (e.g., "hands_on_obstacle", "mid_vault", "landing")
    pub description: Option<String>,
    /// Bone transforms for this pose
    pub bones: Vec<BonePose>,
}

/// Complete animation defined by keyframe poses
#[derive(Debug, Clone, Serialize, Deserialize, Asset, Reflect)]
pub struct ParkourPoseAnimation {
    /// Animation name (e.g., "standing_vault", "running_vault")
    pub name: String,
    /// Total duration of the animation
    pub duration: f32,
    /// Key poses to interpolate between
    pub key_poses: Vec<KeyPose>,
    /// Optional: root motion data if we need it later
    pub root_motion: Option<Vec3>,
}

/// Resource that holds all loaded parkour pose animations
#[derive(Resource, Default)]
pub struct ParkourPoseLibrary {
    pub animations: HashMap<String, Handle<ParkourPoseAnimation>>,
}

/// Bones we care about for parkour animations
pub const CRITICAL_BONES: &[&str] = &[
    // Hands (for IK targets)
    "mixamorig:LeftHand",
    "mixamorig:RightHand",
    "mixamorig:LeftForeArm",
    "mixamorig:RightForeArm",
    "mixamorig:LeftArm",
    "mixamorig:RightArm",

    // Feet (for IK targets)
    "mixamorig:LeftFoot",
    "mixamorig:RightFoot",
    "mixamorig:LeftLeg",
    "mixamorig:RightLeg",
    "mixamorig:LeftUpLeg",
    "mixamorig:RightUpLeg",

    // Core (for body lean/rotation)
    "mixamorig:Spine",
    "mixamorig:Spine1",
    "mixamorig:Spine2",
    "mixamorig:Hips",

    // Head (for look direction)
    "mixamorig:Head",
    "mixamorig:Neck",
];

/// Debug component to mark which animation slot is currently playing
#[derive(Component)]
pub struct DebugAnimationSlot {
    pub slot_number: u32,
    pub animation_name: String,
}

use crate::screens::Screen;
use std::fs;

/// Resource to track which debug animation is currently playing
#[derive(Resource, Default)]
pub struct DebugAnimationState {
    pub current_slot: Option<u32>,
    pub animation_name: String,
    pub animation_start_time: f32,
}

/// System to handle numeric key presses for debug animations
pub fn handle_debug_animation_keys(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<DebugAnimationState>,
    player_assets: Option<Res<crate::game::player::PlayerAssets>>,
    mut animation_player_query: Query<&mut AnimationPlayer>,
    mut transitions_query: Query<&mut AnimationTransitions>,
    time: Res<Time>,
) {
    let Some(assets) = player_assets else {
        return;
    };

    let Ok(mut player) = animation_player_query.single_mut() else {
        return;
    };

    let Ok(mut transitions) = transitions_query.single_mut() else {
        return;
    };

    // Map numeric keys to debug animation slots
    let key_mapping = [
        (KeyCode::Digit1, 1, &assets.animations.debug_slot_1, "debug_1"),
        (KeyCode::Digit2, 2, &assets.animations.debug_slot_2, "debug_2"),
        (KeyCode::Digit3, 3, &assets.animations.debug_slot_3, "debug_3"),
        (KeyCode::Digit4, 4, &assets.animations.debug_slot_4, "debug_4"),
        (KeyCode::Digit5, 5, &assets.animations.debug_slot_5, "debug_5"),
        (KeyCode::Digit6, 6, &assets.animations.debug_slot_6, "debug_6"),
        (KeyCode::Digit7, 7, &assets.animations.debug_slot_7, "debug_7"),
        (KeyCode::Digit8, 8, &assets.animations.debug_slot_8, "debug_8"),
        (KeyCode::Digit9, 9, &assets.animations.debug_slot_9, "debug_9"),
        (KeyCode::Digit0, 0, &assets.animations.debug_slot_0, "debug_0"),
    ];

    for (key, slot_num, animation_handle, anim_name) in key_mapping {
        if keyboard.just_pressed(key) {
            if let Some(handle) = animation_handle {
                info!("▶ Playing debug animation slot {}: {}", slot_num, anim_name);
                transitions.play(&mut player, handle.clone(), Duration::from_millis(200));
                state.current_slot = Some(slot_num);
                state.animation_name = anim_name.to_string();
                state.animation_start_time = time.elapsed_secs();
            } else {
                warn!("Debug slot {} has no animation loaded. Add '{}' to your GLTF.", slot_num, anim_name);
            }
        }
    }
}

/// System to extract bone transforms on F12 press
pub fn extract_bone_poses(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<DebugAnimationState>,
    bone_query: Query<(&Name, &GlobalTransform), With<AnimationTarget>>,
    time: Res<Time>,
) {
    if !keyboard.just_pressed(KeyCode::F12) {
        return;
    }

    if state.current_slot.is_none() {
        warn!("No debug animation playing! Press 1-9 to play a debug animation first.");
        return;
    }

    info!("📸 Extracting bone poses...");

    let current_time = time.elapsed_secs() - state.animation_start_time;
    let mut bones = Vec::new();

    // Extract transforms for critical bones
    for (name, global_transform) in bone_query.iter() {
        let bone_name = name.as_str();

        // Only extract bones we care about
        if CRITICAL_BONES.contains(&bone_name) {
            let (_, rotation, translation) = global_transform.to_scale_rotation_translation();

            bones.push(BonePose {
                bone_name: bone_name.to_string(),
                position: translation,
                rotation,
            });
        }
    }

    if bones.is_empty() {
        error!("No bones found! Make sure the animation is playing.");
        return;
    }

    // Create a KeyPose
    let key_pose = KeyPose {
        time: current_time,
        description: Some(format!("Extracted at {:.2}s", current_time)),
        bones,
    };

    // Convert to RON format
    let ron_config = ron::ser::PrettyConfig::new()
        .depth_limit(4)
        .separate_tuple_members(true)
        .enumerate_arrays(true);

    match ron::ser::to_string_pretty(&key_pose, ron_config) {
        Ok(ron_string) => {
            let filename = format!(
                "assets/parkour_poses/{}_{:.2}s.ron",
                state.animation_name,
                current_time
            );

            match fs::write(&filename, ron_string) {
                Ok(_) => {
                    info!("✅ Saved bone poses to: {}", filename);
                    info!("   Extracted {} bones at time {:.2}s", key_pose.bones.len(), current_time);
                }
                Err(e) => {
                    error!("Failed to write file: {}", e);
                }
            }
        }
        Err(e) => {
            error!("Failed to serialize to RON: {}", e);
        }
    }
}

// ============================================================================
// POSE INTERPOLATION SYSTEM
// ============================================================================

/// Component attached to player to track active procedural pose animation
#[derive(Component)]
pub struct ActivePoseAnimation {
    pub animation: Handle<ParkourPoseAnimation>,
    pub start_time: f32,
    pub looping: bool,
}

/// Interpolate between two bone poses
fn interpolate_bone_pose(a: &BonePose, b: &BonePose, t: f32) -> BonePose {
    BonePose {
        bone_name: a.bone_name.clone(),
        position: a.position.lerp(b.position, t),
        rotation: a.rotation.slerp(b.rotation, t),
    }
}

/// Find the two key poses to interpolate between for a given time
fn find_surrounding_poses<'a>(
    animation: &'a ParkourPoseAnimation,
    time: f32,
) -> Option<(&'a KeyPose, &'a KeyPose, f32)> {
    if animation.key_poses.is_empty() {
        return None;
    }

    // Find the poses before and after the current time
    let mut before_pose = &animation.key_poses[0];
    let mut after_pose = &animation.key_poses[0];

    for i in 0..animation.key_poses.len() {
        let pose = &animation.key_poses[i];
        if pose.time <= time {
            before_pose = pose;
        }
        if pose.time >= time {
            after_pose = pose;
            break;
        }
    }

    // Calculate interpolation factor
    let time_range = after_pose.time - before_pose.time;
    let t = if time_range > 0.0 {
        (time - before_pose.time) / time_range
    } else {
        0.0
    };

    Some((before_pose, after_pose, t))
}

/// System to apply procedural pose animations
pub fn apply_pose_animation(
    mut player_query: Query<&ActivePoseAnimation>,
    pose_assets: Res<Assets<ParkourPoseAnimation>>,
    mut bone_query: Query<(&Name, &mut Transform), With<AnimationTarget>>,
    time: Res<Time>,
) {
    let Ok(active_pose) = player_query.single_mut() else {
        return;
    };

    let Some(animation) = pose_assets.get(&active_pose.animation) else {
        return;
    };

    // Calculate current time in animation
    let elapsed = time.elapsed_secs() - active_pose.start_time;
    let current_time = if active_pose.looping {
        elapsed % animation.duration
    } else {
        elapsed.min(animation.duration)
    };

    // Find surrounding poses
    let Some((before_pose, after_pose, t)) = find_surrounding_poses(animation, current_time) else {
        return;
    };

    // Apply interpolated bone transforms
    for before_bone in &before_pose.bones {
        // Find corresponding bone in after_pose
        let Some(after_bone) = after_pose.bones.iter()
            .find(|b| b.bone_name == before_bone.bone_name) else {
            continue;
        };

        // Interpolate between the two poses
        let interpolated = interpolate_bone_pose(before_bone, after_bone, t);

        // Find the actual bone entity and apply transform
        for (bone_name, mut bone_transform) in bone_query.iter_mut() {
            if bone_name.as_str() == interpolated.bone_name {
                bone_transform.translation = interpolated.position;
                bone_transform.rotation = interpolated.rotation;
            }
        }
    }
}

// ============================================================================
// POSE ANIMATION LOADER
// ============================================================================

/// Example system to load a pose animation from assets
/// You can trigger this when starting a parkour action
pub fn load_pose_animation_example(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
    player_query: Query<Entity, With<crate::game::player::Player>>,
) {
    // Example: Press P to load and play a vault animation
    if keyboard.just_pressed(KeyCode::KeyP) {
        if let Ok(player_entity) = player_query.single() {
            // Load a pose animation from assets
            let pose_animation: Handle<ParkourPoseAnimation> =
                asset_server.load("parkour_poses/standing_vault.ron");

            commands.entity(player_entity).insert(ActivePoseAnimation {
                animation: pose_animation,
                start_time: 0.0, // Will be set by time system
                looping: false,
            });

            info!("Loading procedural vault animation from RON file");
        }
    }
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<ParkourPoseLibrary>();
    app.init_resource::<DebugAnimationState>();
    app.init_asset::<ParkourPoseAnimation>();
    app.register_asset_reflect::<ParkourPoseAnimation>();

    // Register RON asset loader for ParkourPoseAnimation
    app.init_asset_loader::<bevy::asset::io::embedded::EmbeddedAssetLoader>();

    // Add debug systems (only run during gameplay)
    app.add_systems(
        Update,
        (
            handle_debug_animation_keys,
            extract_bone_poses,
            // Pose interpolation system (will be active when you have pose animations)
            // Commented out for now - enable when you have RON files ready
            // apply_pose_animation,
            // load_pose_animation_example,
        )
            .chain()
            .run_if(in_state(Screen::Gameplay)),
    );
}
