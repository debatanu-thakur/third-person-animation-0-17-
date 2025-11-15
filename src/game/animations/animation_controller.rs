use std::time::Duration;

use bevy::{animation, prelude::*};
use bevy_tnua::{TnuaAnimatingState, TnuaAnimatingStateDirective, builtins::TnuaBuiltinJumpState, prelude::*};

use crate::game::player::{self, MovementController, Player, PlayerAssets};

use super::models::{AnimationState, CharacterAnimationController};

/// Stores the indices of animation nodes in the animation graph
#[derive(Resource)]
pub struct AnimationNodes {
    pub idle: AnimationNodeIndex,
    pub walk: AnimationNodeIndex,
    pub run: AnimationNodeIndex,
    pub jump: AnimationNodeIndex,
    pub running_jump: AnimationNodeIndex,
    pub fall: AnimationNodeIndex,
}

/// Creates the animation graph with all clips and transitions
pub fn setup_animation_graph(
    mut commands: Commands,
    player_assets: Option<Res<PlayerAssets>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    animation_nodes: Option<Res<AnimationNodes>>,
    mut animation_player_query: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    // If animation nodes exist, no need to process this anymore
    if let Some(_) = animation_nodes {
        return;
    }

    let Some(player_assets) = player_assets else {
        return;
    };
    // This needs to be all players
    let Ok((animation_player_entity, mut animation_player)) = animation_player_query.single_mut() else {
        return;
    };

    let mut graph = AnimationGraph::new();
    let root_node = graph.root;

    let animations = &player_assets.animations;

    // Add all animation clips as nodes
    let idle_node = graph.add_clip(animations.idle.clone(), 1.0, root_node);
    let walk_node = graph.add_clip(animations.walking.clone(), 1.0, root_node);
    let run_node = graph.add_clip(animations.running.clone(), 1.0, root_node);
    let jump_node = graph.add_clip(animations.standing_jump.clone(), 1.0, root_node);
    let running_jump_node = graph.add_clip(animations.running_jump.clone(), 1.0, root_node);
    // Note: Reusing standing_jump for falling since we don't have a dedicated falling animation yet
    let fall_node = graph.add_clip(animations.standing_jump.clone(), 1.0, root_node);

    // Store the graph and node indices
    let graph_handle = graphs.add(graph);

    commands.insert_resource(AnimationNodes {
        idle: idle_node,
        walk: walk_node,
        run: run_node,
        jump: jump_node,
        fall: fall_node,
        running_jump: running_jump_node,
    });
    let mut transitions = AnimationTransitions::new();
    transitions
        .play(
            &mut animation_player,
            idle_node,
            Duration::ZERO)
        .repeat();

    // Store the graph handle as a resource for easy access
    commands
    .entity(animation_player_entity)
    .insert(AnimationGraphHandle(graph_handle))
    .insert(transitions)
    ;

    info!("Animation graph successfully created with unified GLTF animations!");
}


/// Updates animation state based on Tnua controller state
pub fn update_animation_state(
    mut player_query: Query<
        (&TnuaController, &mut TnuaAnimatingState<AnimationState>),
        With<Player>,
    >,
    mut animation_player_query: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    animation_nodes: Option<Res<AnimationNodes>>,
) {
    let Ok((mut animation_player, mut transitions)) = animation_player_query.single_mut() else {
        return;
    };
    let Some(animation_nodes) = animation_nodes else {
        return;
    };

    for (controller, mut animating_state) in player_query.iter_mut() {
        let new_state = determine_animation_state(controller);
        apply_animation_state(&mut animating_state, new_state, &mut animation_player, &mut transitions, &animation_nodes);

    }
}

/// Determines which animation state to use based on Tnua controller
pub fn determine_animation_state(controller: &TnuaController) -> AnimationState {
    let current_status_for_animating = match controller.action_name() {
        Some(TnuaBuiltinJump::NAME) => {
            // Jump action is active - play the full jump animation sequence
            // The standing_jump animation includes the full jump and landing, so we
            // don't need to check the jump state or handle falling separately
            AnimationState::Jumping
        }
        Some("jump") => {
            AnimationState::Jumping
        }
        // Tnua should only have the `action_name` of the actions you feed to it. If it has
        // anything else - consider it a bug.
        Some(other) => {
            warn!("Unknown action {other}");
            AnimationState::Idle
        }
        // No action name means that no action is currently being performed - which means the
        // animation should be decided by the basis.
        None => {
            // If there is no action going on, we'll base the animation on the state of the
            // basis.
            let Some((_, basis_state)) = controller.concrete_basis::<TnuaBuiltinWalk>() else {
                // Since we only use the walk basis in this example, if we can't get get this
                // basis' state it probably means the system ran before any basis was set, so we
                // just skip this frame.
                return AnimationState::Idle;
            };

            // Speed threshold for idle
            const IDLE_THRESHOLD: f32 = 0.1;  // Below this = idle

            const WALK_THRESHOLD: f32 = 2.0;  // Below this = idle

            let speed = basis_state.running_velocity.length();
            if speed < IDLE_THRESHOLD {
                AnimationState::Idle
            } else if speed <= WALK_THRESHOLD {
                AnimationState::Walking
            }
            else {
                // Any movement uses the Moving state with the actual speed
                // The blend between walk and run animations will be handled automatically
                AnimationState::Running(speed)
            }
        }
    };
    current_status_for_animating

}

/// Applies the current animation state to the animation player with blending
fn apply_animation_state(
    animating_state: &mut TnuaAnimatingState<AnimationState>,
    new_state: AnimationState,
    animation_player: &mut AnimationPlayer,
    transitions: &mut AnimationTransitions,
    animation_nodes: &AnimationNodes,
) {
     let animating_directive = animating_state.update_by_discriminant(new_state);

     match animating_directive {
        TnuaAnimatingStateDirective::Maintain { state } => {
            // info!("Maintained");
            // `Maintain` means that we did not switch to a different variant, so there is no need
            // to change animations.

            // For the Moving state, even when the state variant remains the same, the speed can
            // change. We need to update the blend weights to smoothly transition between walk and run.

        }
        TnuaAnimatingStateDirective::Alter {
            old_state,
            state,
        } => {
            // info!("Altered");
            // `Alter` means that we have switched to a different variant and need to play a
            // different animation.

            // First - stop the currently running animation. We don't check which one is running
            // here because we just assume it belongs to the old state, but more sophisticated code
            // can try to phase from the old animation to the new one.
            // animation_player.stop_all();

            // Depending on the new state, we choose the animation to run and its parameters
            match state {
                AnimationState::Idle => {
                    transitions.play(
                        animation_player,
                        animation_nodes.idle,
                         Duration::from_millis(200)).repeat();
                },
                AnimationState::Walking => {
                    transitions
                    .play(
                        animation_player,
                        animation_nodes.walk,
                        Duration::from_millis(200)).repeat();
                },
                AnimationState::Moving(_) => {
                    transitions
                    .play(
                        animation_player,
                        animation_nodes.run,
                        Duration::from_millis(500)).repeat();
                },
                AnimationState::Running(_) => {
                    transitions
                    .play(
                        animation_player,
                        animation_nodes.run,
                        Duration::from_millis(500))
                        .repeat()
                        .set_speed(1.2);
                },
                AnimationState::Jumping => {
                    // info!("Jumping");
                    match old_state.unwrap() {
                        AnimationState::Walking |
                        AnimationState::Running(_) => {
                        transitions
                        .play(
                            animation_player,
                        animation_nodes.running_jump,
                        Duration::from_millis(50))
                            .set_speed(1.2);
                        }
                        _ => {

                        }
                    }
                }
            }
        }
    }
}


fn update_moving_speed(
    animation_player: &mut AnimationPlayer,
    speed: f32,
) {
    let Some((&animation_index, _)) = animation_player.playing_animations().next() else {
        return;
    };
    let animation = animation_player.animation_mut(animation_index).unwrap();
    animation.set_speed(speed);
}
