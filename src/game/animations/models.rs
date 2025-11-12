use bevy::prelude::*;

pub enum AnimationState {
    Standing,
    Running(f32),
    Jumping,
    Falling,
}

#[derive(Resource)]
struct AnimationNodes {
    standing: AnimationNodeIndex,
    running: AnimationNodeIndex,
    jumping: AnimationNodeIndex,
    falling: AnimationNodeIndex,
}
