use bevy::prelude::*;
use std::collections::HashMap;
use crate::screens::Screen;

mod assets;
pub use assets::{ParkourGltfAssets, ParkourAnimations, extract_parkour_animation_clips};

// ============================================================================
// PARKOUR ANIMATION LIBRARY
// ============================================================================

/// Converts ParkourAnimations into ParkourAnimationLibrary
/// This runs once after animations are extracted from GLTF
pub fn create_parkour_library(
    mut commands: Commands,
    parkour_animations: Option<Res<ParkourAnimations>>,
    library: Option<Res<ParkourAnimationLibrary>>,
) {
    // Only run once
    if library.is_some() {
        return;
    }

    let Some(animations) = parkour_animations else {
        return;
    };

    info!("🎨 Creating parkour animation library from extracted animations");

    commands.insert_resource(ParkourAnimationLibrary {
        vault_clip: animations.vault.clone(),
        climb_clip: animations.climb.clone(),
        slide_clip: animations.slide.clone(),
        wall_run_left_clip: animations.wall_run_left.clone(),
        wall_run_right_clip: animations.wall_run_right.clone(),
        roll_clip: animations.roll.clone(),
    });
}

/// Resource holding animation library
#[derive(Resource)]
pub struct ParkourAnimationLibrary {
    pub vault_clip: Handle<AnimationClip>,
    pub climb_clip: Handle<AnimationClip>,
    pub slide_clip: Handle<AnimationClip>,
    pub wall_run_left_clip: Handle<AnimationClip>,
    pub wall_run_right_clip: Handle<AnimationClip>,
    pub roll_clip: Handle<AnimationClip>,
}

// ============================================================================
// ANIMATION SAMPLING DATA STRUCTURES
// ============================================================================

/// Sampled bone transform at a specific time
#[derive(Debug, Clone)]
pub struct SampledBoneTransform {
    pub bone_name: String,
    pub translation: Vec3,
    pub rotation: Quat,
    pub time: f32,
}

/// Keyframe data extracted from animation
#[derive(Debug, Clone)]
pub struct AnimationKeyframe {
    pub time: f32,
    pub bones: Vec<SampledBoneTransform>,
}

// ============================================================================
// ANIMATION SAMPLING RESOURCES
// ============================================================================

/// Stores sampled animation poses for IK targeting
#[derive(Resource, Default)]
pub struct SampledParkourPoses {
    /// Vault animation samples at key times (0.0s, 0.25s, 0.5s, 0.75s, 1.0s)
    pub vault_samples: HashMap<String, Vec<SampledBoneTransform>>, // time_key -> bone_transforms

    /// Climb animation samples
    pub climb_samples: HashMap<String, Vec<SampledBoneTransform>>,

    /// Slide animation samples
    pub slide_samples: HashMap<String, Vec<SampledBoneTransform>>,

    /// Whether sampling is complete
    pub sampled: bool,
}

impl SampledParkourPoses {
    /// Get hand position from vault animation at specific time
    pub fn get_vault_hand_pos(&self, time: f32, hand: &str) -> Option<Vec3> {
        let time_key = format!("{:.2}", time);
        if let Some(bones) = self.vault_samples.get(&time_key) {
            for bone in bones {
                if bone.bone_name.contains(hand) {
                    return Some(bone.translation);
                }
            }
        }
        None
    }
}

/// Marker component for temporary sampling entities
#[derive(Component)]
pub struct AnimationSampler {
    pub animation_name: String,
    pub sample_times: Vec<f32>,
    pub current_sample_index: usize,
    pub samples_collected: Vec<(f32, Vec<(String, Vec3, Quat)>)>,
}

// ============================================================================
// DEBUG: TEST ANIMATION PLAYBACK
// ============================================================================

/// Test system to play parkour animation on character (press 'O' to test)
pub fn test_parkour_animation_playback(
    keyboard: Res<ButtonInput<KeyCode>>,
    library: Option<Res<ParkourAnimationLibrary>>,
    mut player_query: Query<(&mut AnimationPlayer, &AnimationGraphHandle), With<crate::game::player::Player>>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyO) {
        return;
    }

    let Some(library) = library else {
        warn!("Parkour animations not loaded yet!");
        return;
    };

    let Ok((mut player, current_graph_handle)) = player_query.single_mut() else {
        warn!("No player with AnimationPlayer found!");
        return;
    };

    info!("🧪 Testing vault animation playback on character...");

    // Get the current animation graph
    if let Some(graph) = animation_graphs.get_mut(current_graph_handle) {
        // Add the vault animation to the current graph
        let vault_node = graph.add_clip(library.vault_clip.clone(), 1.0, graph.root);

        // Play the vault animation
        player.play(vault_node).repeat();

        info!("✅ Playing vault animation!");
        info!("   If the character animates → Retargeting works! ✅");
        info!("   If nothing happens → Bone names don't match ❌");
        info!("   Press '1' to return to normal walk/run animation");
    } else {
        warn!("Could not access animation graph!");
    }
}

// ============================================================================
// DEBUG: SAMPLE AND PRINT
// ============================================================================

/// Debug system to print animation library info (press 'P')
pub fn debug_sample_animation(
    keyboard: Res<ButtonInput<KeyCode>>,
    library: Option<Res<ParkourAnimationLibrary>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyP) {
        return;
    }

    let Some(library) = library else {
        warn!("Parkour animations not loaded yet!");
        return;
    };

    info!("📊 Parkour animation library ready:");
    info!("   Vault clip: {:?}", library.vault_clip);
    info!("   Climb clip: {:?}", library.climb_clip);
    info!("   Slide clip: {:?}", library.slide_clip);
    info!("");
    info!("💡 Press 'O' to test vault animation playback on character");
}

// ============================================================================
// PLUGIN
// ============================================================================

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<ParkourGltfAssets>();
    app.init_resource::<SampledParkourPoses>();

    app.add_systems(
        Update,
        (
            // Asset loading (runs once when GLTF loads)
            extract_parkour_animation_clips,
            create_parkour_library,

            // Debug systems
            test_parkour_animation_playback,
            debug_sample_animation,
        )
            .chain()
            .run_if(in_state(Screen::Gameplay)),
    );
}
