use bevy::prelude::*;
use std::time::Duration;

/// Current animation state of the player
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum AnimationState {
    Idle,
    Walking,
    Running,
    Jumping,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self::Idle
    }
}

/// Tracks how long player has been in Walking state to determine when to transition to Run
#[derive(Component)]
pub struct MovementTimer {
    /// Time spent in current movement state
    pub time_in_state: Duration,
    /// Whether we're currently transitioning
    pub is_transitioning: bool,
}

impl Default for MovementTimer {
    fn default() -> Self {
        Self {
            time_in_state: Duration::ZERO,
            is_transitioning: false,
        }
    }
}

/// Component that stores the animation graph and player for a character
#[derive(Component)]
pub struct CharacterAnimationController {
    pub graph: Handle<AnimationGraph>,
    pub animation_player: Entity,
}
