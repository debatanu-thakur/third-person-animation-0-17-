use bevy::prelude::*;
use bevy::animation::*;
use bevy_tnua::TnuaToggle;
use bevy_tnua::prelude::TnuaController;

use crate::game::player::Player;

// ============================================================================
// ANIMATION COMPLETION DETECTION
// ============================================================================

#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub enum ParkourState {
    #[default]
    Idle,
    Walking,
    Running,
    Sprinting,
    /// Vaulting over obstacle
    Vaulting,
    /// Climbing up wall
    Climbing,
    /// Hanging on ledge
    Hanging,
    /// Wall running
    WallRunning,
    /// Sliding under/on obstacle
    Sliding,
    /// Jumping over gap
    Jumping,
    /// Landing from height
    Landing,
}

/// Event fired when a parkour animation completes
/// This is embedded in animation clips and fired automatically by Bevy
#[derive(AnimationEvent, Clone, Reflect)]
pub struct ParkourAnimationComplete {
    /// Which parkour action just completed
    pub action: ParkourState,
}

#[derive(AnimationEvent, Clone, Reflect)]
pub struct ParkourAnimationStart {
    /// Which parkour action just completed
    pub action: ParkourState,
}

/// Event fired when parkour animation should start blending to locomotion
/// Fired before animation ends to allow smooth transition
#[derive(AnimationEvent, Clone, Reflect)]
pub struct ParkourAnimationBlendToIdle {
    /// Which parkour action is blending out
    pub action: ParkourState,
}

#[derive(Component, Default)]
pub struct ParkourController {
    pub state: ParkourState,
    pub can_vault: bool,
    pub can_climb: bool,
    pub can_wall_run: bool,
    pub can_slide: bool,
}

#[derive(Component, Default)]
pub struct PlayingParkourAnimation;

// ============================================================================
// EVENT-DRIVEN ANIMATION COMPLETION
// ============================================================================
// Note: Time-based completion system removed - fully event-driven now
// Animation events (ParkourAnimationBlendToIdle + ParkourAnimationComplete)
// embedded in clips handle all completion timing automatically

/// Observer function that handles parkour animation blend start events
/// Triggers smooth transition to locomotion before animation ends
pub fn on_parkour_blend_to_idle(
    trigger: On<ParkourAnimationBlendToIdle>,
    mut player_query: Query<&mut ParkourController, With<Player>>,
) {
    let event = trigger.event();
    info!("🎨 Blend event: Starting transition from {:?} to Idle", event.action);

    // Transition to idle - AnimationController will blend over duration
    for mut parkour in player_query.iter_mut() {
        if parkour.state == event.action {
            parkour.state = ParkourState::Idle;
            info!("✅ Blend started: {:?} → Idle (smooth transition)", event.action);
        }
    }
}

/// Observer function that handles parkour animation completion events
/// This is triggered automatically when animation clips fire ParkourAnimationComplete
pub fn on_parkour_animation_complete(
    trigger: On<ParkourAnimationComplete>,
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut ParkourController), With<Player>>,
) {
    let event = trigger.event();
    info!("🎬 Animation event received: {:?} completed", event.action);

    // Return player to idle state (fallback if blend didn't happen)
    for (player_entity, mut parkour) in player_query.iter_mut() {
        if parkour.state == event.action {
            parkour.state = ParkourState::Idle;
            info!("✅ Animation event: Returning to Idle from {:?}", event.action);
        }
        commands.entity(player_entity).insert(TnuaToggle::Enabled);
        commands.entity(player_entity).remove::<PlayingParkourAnimation>();
    }
}

pub fn on_parkour_animation_start(
    trigger: On<ParkourAnimationStart>,
    mut commands: Commands,
    mut player_query: Query<Entity, With<Player>>,
) {
    let event = trigger.event();
    info!("🎬 Animation start event received: {:?}", event.action);

    // Return player to idle state (fallback if blend didn't happen)
    for player_entity in player_query.iter_mut() {
        commands.entity(player_entity).insert(TnuaToggle::Disabled);
    }
}

