use bevy::prelude::*;
use bevy_tnua::{TnuaAnimatingState, TnuaAnimatingStateDirective, builtins::TnuaBuiltinJumpState, prelude::*};

use crate::game::player::{self, Player, PlayerAssets};

use super::models::{AnimationState, CharacterAnimationController};

/// Stores the indices of animation nodes in the animation graph
#[derive(Resource)]
pub struct AnimationNodes {
    pub idle: AnimationNodeIndex,
    pub walk: AnimationNodeIndex,
    pub run: AnimationNodeIndex,
    pub jump: AnimationNodeIndex,
    pub fall: AnimationNodeIndex,
}

/// Creates the animation graph with all clips and transitions
pub fn setup_animation_graph(
    mut commands: Commands,
    player_assets: Option<Res<PlayerAssets>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    animation_nodes: Option<Res<AnimationNodes>>,
    animation_player_query: Query<Entity, With<AnimationPlayer>>,
) {
    // If animation nodes exist, no need to process this anymore
    if let Some(_) = animation_nodes {
        return;
    }

    let Some(player_assets) = player_assets else {
        return;
    };
    let Ok(animation_player_entity) = animation_player_query.single() else {
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
    });

    // Store the graph handle as a resource for easy access
    commands.entity(animation_player_entity).insert(AnimationGraphHandle(graph_handle));

    info!("Animation graph successfully created with unified GLTF animations!");
}


/// Updates animation state based on Tnua controller state
pub fn update_animation_state(
    mut player_query: Query<
        (&TnuaController, &mut TnuaAnimatingState<AnimationState>),
        With<Player>,
    >,
    mut animation_player_query: Query<&mut AnimationPlayer>,
    animation_nodes: Option<Res<AnimationNodes>>,
) {
    let Ok(mut animation_player) = animation_player_query.single_mut() else {
        return;
    };
    let Some(animation_nodes) = animation_nodes else {
        return;
    };
    for (controller, mut animating_state) in player_query.iter_mut() {
        let new_state = determine_animation_state(controller);
        apply_animation_state(&mut animating_state, new_state, &mut animation_player, &animation_nodes);

    }
}

/// Determines which animation state to use based on Tnua controller
fn determine_animation_state(controller: &TnuaController) -> AnimationState {
    let current_status_for_animating = match controller.action_name() {
        Some(TnuaBuiltinJump::NAME) => {
            // In case of jump, we want to cast it so that we can get the concrete jump state.
            let (_, jump_state) = controller
                .concrete_action::<TnuaBuiltinJump>()
                .expect("action name mismatch");
            // Depending on the state of the jump, we need to decide if we want to play the jump
            // animation or the fall animation.
            match jump_state {
                TnuaBuiltinJumpState::NoJump => AnimationState::Idle,
                TnuaBuiltinJumpState::StartingJump { .. } => AnimationState::Jumping,
                TnuaBuiltinJumpState::SlowDownTooFastSlopeJump { .. } => AnimationState::Jumping,
                TnuaBuiltinJumpState::MaintainingJump { .. } => AnimationState::Jumping,
                TnuaBuiltinJumpState::StoppedMaintainingJump => AnimationState::Jumping,
                TnuaBuiltinJumpState::FallSection => AnimationState::Falling,
            }
        }
        // Tnua should only have the `action_name` of the actions you feed to it. If it has
        // anything else - consider it a bug.
        Some(other) => {
            warn!("Unknown action {other}");
            AnimationState::Idle
        },
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
            if basis_state.standing_on_entity().is_none() {
                // The walk basis keeps track of what the character is standing on. If it doesn't
                // stand on anything, `standing_on_entity` will be empty - which means the
                // character has walked off a cliff and needs to fall.
                AnimationState::Falling
            } else {
                // Speed threshold for idle
                const IDLE_THRESHOLD: f32 = 0.1;  // Below this = idle

                let speed = basis_state.running_velocity.length();
                if speed < IDLE_THRESHOLD {
                    AnimationState::Idle
                } else {
                    // Any movement uses the Moving state with the actual speed
                    // The blend between walk and run animations will be handled automatically
                    AnimationState::Moving(speed)
                }
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
    animation_nodes: &AnimationNodes,
) {
     let animating_directive = animating_state.update_by_discriminant(new_state);
     match animating_directive {
        TnuaAnimatingStateDirective::Maintain { state } => {
            // `Maintain` means that we did not switch to a different variant, so there is no need
            // to change animations.

            // For the Moving state, even when the state variant remains the same, the speed can
            // change. We need to update the blend weights to smoothly transition between walk and run.
            if let AnimationState::Moving(speed) = state {
                update_walk_run_blend(animation_player, animation_nodes, *speed);
            }
        }
        TnuaAnimatingStateDirective::Alter {
            old_state: _,
            state,
        } => {
            // `Alter` means that we have switched to a different variant and need to play a
            // different animation.

            // First - stop the currently running animation. We don't check which one is running
            // here because we just assume it belongs to the old state, but more sophisticated code
            // can try to phase from the old animation to the new one.
            animation_player.stop_all();

            // Depending on the new state, we choose the animation to run and its parameters
            match state {
                AnimationState::Idle => {
                    animation_player
                        .start(animation_nodes.idle)
                        .set_speed(1.0)
                        .repeat();
                }
                AnimationState::Moving(speed) => {
                    // Start both walk and run animations simultaneously
                    // They will be blended based on their weights
                    animation_player
                        .start(animation_nodes.walk)
                        .set_speed(1.0)
                        .repeat();
                    animation_player
                        .start(animation_nodes.run)
                        .set_speed(1.0)
                        .repeat();
                    // Set the initial blend weights based on speed
                    update_walk_run_blend(animation_player, animation_nodes, *speed);
                }
                AnimationState::Jumping => {
                    info!("I am jumping");
                }
                AnimationState::Falling => {
                    info!("I am falling");
                }
            }
        }
    }
}

/// Updates the walk-run blend weights based on current speed
/// This provides smooth transitions between walking and running animations
fn update_walk_run_blend(
    animation_player: &mut AnimationPlayer,
    animation_nodes: &AnimationNodes,
    speed: f32,
) {
    // Speed thresholds that match MovementController
    const WALK_SPEED: f32 = 2.0;
    const RUN_SPEED: f32 = 8.0;  // Updated to match the new run_speed

    // Calculate blend factor: 0.0 = 100% walk, 1.0 = 100% run
    // Clamp between walk and run speeds, then normalize to 0.0-1.0
    let blend_factor = ((speed - WALK_SPEED) / (RUN_SPEED - WALK_SPEED)).clamp(0.0, 1.0);

    // Set weights for walk and run animations
    // Walk weight decreases as speed increases, run weight increases
    let walk_weight = 1.0 - blend_factor;
    let run_weight = blend_factor;

    // Apply the weights to the playing animations
    animation_player.animation_mut(animation_nodes.walk).map(|anim| anim.set_weight(walk_weight));
    animation_player.animation_mut(animation_nodes.run).map(|anim| anim.set_weight(run_weight));
}
