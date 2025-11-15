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

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<ParkourPoseLibrary>();
    app.init_resource::<DebugAnimationState>();
    app.init_asset::<ParkourPoseAnimation>();
    app.register_asset_reflect::<ParkourPoseAnimation>();

    // Add debug systems (only run during gameplay)
    app.add_systems(
        Update,
        (
            handle_debug_animation_keys,
            extract_bone_poses,
        )
            .chain()
            .run_if(in_state(Screen::Gameplay)),
    );
}
