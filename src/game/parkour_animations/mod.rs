use bevy::prelude::*;
use crate::screens::Screen;
use crate::game::obstacle_detection::detection::{ParkourController, ParkourState};

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
// DEBUG: TRIGGER VAULT STATE FOR TESTING
// ============================================================================

/// Test system to trigger vault animation by setting parkour state (press 'V')
pub fn test_trigger_vault_animation(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut ParkourController, With<crate::game::player::Player>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyV) {
        return;
    }

    let Ok(mut parkour) = player_query.single_mut() else {
        warn!("❌ No player with ParkourController found!");
        return;
    };

    // Toggle between Vaulting and Idle
    if matches!(parkour.state, ParkourState::Vaulting) {
        parkour.state = ParkourState::Idle;
        info!("🛑 Vault animation stopped (state = Idle)");
    } else {
        parkour.state = ParkourState::Vaulting;
        info!("");
        info!("🧪 ============================================");
        info!("🧪 VAULT ANIMATION TEST TRIGGERED");
        info!("🧪 ============================================");
        info!("✅ Set parkour state to Vaulting");
        info!("   The animation controller will now play vault animation");
        info!("");
        info!("   👀 WATCH YOUR CHARACTER:");
        info!("   ✅ If character does vaulting motion → RETARGETING WORKS!");
        info!("   ❌ If character freezes/T-poses → Bone mismatch");
        info!("");
        info!("   Press 'V' again to stop");
        info!("🧪 ============================================");
        info!("");
    }
}

// ============================================================================
// PLUGIN
// ============================================================================

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<ParkourGltfAssets>();

    app.add_systems(
        Update,
        (
            // Asset loading (runs once when GLTF loads)
            extract_parkour_animation_clips,
            create_parkour_library,

            // Debug systems
            test_trigger_vault_animation,      // 'V' key - trigger vault animation
        )
            .run_if(in_state(Screen::Gameplay)),
    );
}
