use bevy::prelude::*;

/// Current animation state of the player
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum AnimationState {
    Idle,
    Running,
    Jumping,
    Falling,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self::Idle
    }
}

/// Stores the indices of animation nodes in the animation graph
#[derive(Resource)]
pub struct AnimationNodes {
    pub idle: AnimationNodeIndex,
    pub run: AnimationNodeIndex,
    pub jump: AnimationNodeIndex,
    pub fall: AnimationNodeIndex,
}

/// Component that stores the animation graph and player for a character
#[derive(Component)]
pub struct CharacterAnimationController {
    pub graph: Handle<AnimationGraph>,
    pub animation_player: Entity,
}
