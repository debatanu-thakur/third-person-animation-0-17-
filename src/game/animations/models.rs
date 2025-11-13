use bevy::prelude::*;

/// Current animation state of the player
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum AnimationState {
    Idle,
    Running(f32),
    Jumping,
    Falling,
    Walk,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self::Idle
    }
}

/// Component that stores the animation graph and player for a character
#[derive(Component)]
pub struct CharacterAnimationController {
    pub graph: Handle<AnimationGraph>,
    pub animation_player: Entity,
}
