use bevy::prelude::*;
use crate::screens::Screen;
use crate::game::obstacle_detection::detection::{ParkourController, ParkourState, ParkourAnimationComplete, ParkourAnimationBlendToIdle};

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

/// Marker to track if completion events have been added
#[derive(Resource, Default)]
struct AnimationEventsAdded;

/// Adds blend and completion events to parkour animation clips
/// Blend events fire before animation ends for smooth transitions
/// This runs once after clips are loaded
pub fn add_completion_events_to_clips(
    mut commands: Commands,
    library: Option<Res<ParkourAnimationLibrary>>,
    events_added: Option<Res<AnimationEventsAdded>>,
    mut animation_clips: ResMut<Assets<AnimationClip>>,
) {
    // Only run once
    if events_added.is_some() {
        return;
    }

    let Some(library) = library else {
        return;
    };

    info!("🎬 Adding blend and completion events to parkour animations...");

    // Blend duration (how long before end to start blending)
    const BLEND_TIME: f32 = 0.3; // Start blending 300ms before end

    // Add events to vault animation
    if let Some(vault_clip) = animation_clips.get_mut(&library.vault_clip) {
        let duration = vault_clip.duration();

        // Blend event (300ms before end)
        vault_clip.add_event(
            (duration - BLEND_TIME).max(0.0),
            ParkourAnimationBlendToIdle {
                action: ParkourState::Vaulting,
            },
        );

        // Completion event (at end - fallback)
        vault_clip.add_event(
            duration,
            ParkourAnimationComplete {
                action: ParkourState::Vaulting,
            },
        );
        info!("  ✅ Vault: blend@{:.2}s, complete@{:.2}s", duration - BLEND_TIME, duration);
    }

    // Add events to climb animation
    if let Some(climb_clip) = animation_clips.get_mut(&library.climb_clip) {
        let duration = climb_clip.duration();

        climb_clip.add_event(
            (duration - BLEND_TIME).max(0.0),
            ParkourAnimationBlendToIdle {
                action: ParkourState::Climbing,
            },
        );

        climb_clip.add_event(
            duration,
            ParkourAnimationComplete {
                action: ParkourState::Climbing,
            },
        );
        info!("  ✅ Climb: blend@{:.2}s, complete@{:.2}s", duration - BLEND_TIME, duration);
    }

    // Add events to slide animation
    if let Some(slide_clip) = animation_clips.get_mut(&library.slide_clip) {
        let duration = slide_clip.duration();

        slide_clip.add_event(
            (duration - BLEND_TIME).max(0.0),
            ParkourAnimationBlendToIdle {
                action: ParkourState::Sliding,
            },
        );

        slide_clip.add_event(
            duration,
            ParkourAnimationComplete {
                action: ParkourState::Sliding,
            },
        );
        info!("  ✅ Slide: blend@{:.2}s, complete@{:.2}s", duration - BLEND_TIME, duration);
    }

    // Mark as complete
    commands.insert_resource(AnimationEventsAdded);
    info!("🎬 Animation events added successfully! (smooth blending enabled)");
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
            add_completion_events_to_clips,  // Add completion events to animations

            // Debug systems
            test_trigger_vault_animation,      // 'V' key - trigger vault animation
        )
            .run_if(in_state(Screen::Gameplay)),
    );
}
