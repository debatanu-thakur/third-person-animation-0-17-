use std::time::Duration;

use bevy::{animation, ecs::error::info, prelude::*};
use bevy_tnua::{TnuaAnimatingState, TnuaAnimatingStateDirective, builtins::TnuaBuiltinJumpState, prelude::*};

use crate::game::{
    player::{self, MovementController, Player, PlayerAssets},
    obstacle_detection::detection::{ParkourController, ParkourState},
    parkour_animations::ParkourAnimationLibrary,
};

use super::models::{AnimationState, CharacterAnimationController, MovementTimer};

/// Stores the indices of animation nodes in the animation graph
#[derive(Resource)]
pub struct AnimationNodes {
    pub idle: AnimationNodeIndex,
    pub walk: AnimationNodeIndex,
    pub run: AnimationNodeIndex,
    pub jump: AnimationNodeIndex,
    pub running_jump: AnimationNodeIndex,
    pub fall: AnimationNodeIndex,
    // Parkour animations
    pub vault: AnimationNodeIndex,
    pub climb: AnimationNodeIndex,
    pub slide: AnimationNodeIndex,
    pub wall_run: AnimationNodeIndex,
}

/// Creates the animation graph with all clips and transitions
pub fn setup_animation_graph(
    mut commands: Commands,
    player_assets: Option<Res<PlayerAssets>>,
    parkour_animations: Option<Res<ParkourAnimationLibrary>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    animation_nodes: Option<Res<AnimationNodes>>,
    mut animation_player_query: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    // If animation nodes exist, no need to process this anymore
    if let Some(_) = animation_nodes {
        warn!("Animation nodes exists, no need to process");
        return;
    }

    let Some(player_assets) = player_assets else {
        warn!("Player assets not found");
        return;
    };

    let Some(parkour_animations) = parkour_animations else {
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

    // Add parkour animation clips
    let vault_node = graph.add_clip(parkour_animations.vault_clip.clone(), 1.0, root_node);
    let climb_node = graph.add_clip(parkour_animations.climb_clip.clone(), 1.0, root_node);
    let slide_node = graph.add_clip(parkour_animations.slide_clip.clone(), 1.0, root_node);
    let wall_run_node = graph.add_clip(parkour_animations.wall_run_left_clip.clone(), 1.0, root_node);

    // Store the graph and node indices
    let graph_handle = graphs.add(graph);

    commands.insert_resource(AnimationNodes {
        idle: idle_node,
        walk: walk_node,
        run: run_node,
        jump: jump_node,
        fall: fall_node,
        running_jump: running_jump_node,
        vault: vault_node,
        climb: climb_node,
        slide: slide_node,
        wall_run: wall_run_node,
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


/// Updates animation state based on Tnua controller state and parkour actions
pub fn update_animation_state(
    mut player_query: Query<
        (
            &TnuaController,
            &mut TnuaAnimatingState<AnimationState>,
            &mut MovementTimer,
            &ParkourController,
        ),
        With<Player>,
    >,
    mut animation_player_query: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    animation_nodes: Option<Res<AnimationNodes>>,
    time: Res<Time>,
) {
    let Ok((mut animation_player, mut transitions)) = animation_player_query.single_mut() else {
        return;
    };
    let Some(animation_nodes) = animation_nodes else {
        return;
    };

    for (controller, mut animating_state, mut movement_timer, parkour) in player_query.iter_mut() {
        info!("For player queries");
        let new_state = determine_animation_state(controller, &mut movement_timer, &time, parkour);
        apply_animation_state(
            &mut animating_state,
            new_state,
            &mut animation_player,
            &mut transitions,
            &animation_nodes,
        );
    }
}

/// Determines which animation state to use based on parkour state and Tnua controller
pub fn determine_animation_state(
    controller: &TnuaController,
    movement_timer: &mut MovementTimer,
    time: &Time,
    parkour: &ParkourController,
) -> AnimationState {
    // Parkour actions override normal movement animations
    let parkour_animation = match parkour.state {
        ParkourState::Vaulting => Some(AnimationState::Vaulting),
        ParkourState::Climbing => Some(AnimationState::Climbing),
        ParkourState::Sliding => Some(AnimationState::Sliding),
        ParkourState::WallRunning => Some(AnimationState::WallRunning),
        // Not performing parkour action, check normal movement
        _ => None,
    };

    // If performing parkour action, return that state
    if let Some(parkour_state) = parkour_animation {
        movement_timer.time_in_state = Duration::ZERO;
        info!("I am here");
        return parkour_state;
    }

    // Otherwise, determine state from Tnua controller
    const IDLE_THRESHOLD: f32 = 0.1; // Below this = idle
    const WALK_TO_RUN_DURATION: Duration = Duration::from_millis(200); // Walk for 200 milli second before transitioning to run

    let current_status_for_animating = match controller.action_name() {
        Some(TnuaBuiltinJump::NAME) | Some("jump") => {
            // Jump action is active - reset timer since we're not walking/running
            movement_timer.time_in_state = Duration::ZERO;
            movement_timer.is_transitioning = false;
            AnimationState::Jumping
        }
        // Tnua should only have the `action_name` of the actions you feed to it. If it has
        // anything else - consider it a bug.
        Some(other) => {
            warn!("Unknown action {other}");
            movement_timer.time_in_state = Duration::ZERO;
            movement_timer.is_transitioning = false;
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
                movement_timer.time_in_state = Duration::ZERO;
                movement_timer.is_transitioning = false;
                return AnimationState::Idle;
            };

            let speed = basis_state.running_velocity.length();

            if speed < IDLE_THRESHOLD {
                // Player stopped moving - reset timer
                movement_timer.time_in_state = Duration::ZERO;
                movement_timer.is_transitioning = false;
                AnimationState::Idle
            } else {
                // Player is moving - increment timer
                movement_timer.time_in_state += time.delta();

                // After 1 second of walking, transition to running
                if movement_timer.time_in_state >= WALK_TO_RUN_DURATION {
                    AnimationState::Running
                } else {
                    AnimationState::Walking
                }
            }
        }
    };
    info!("Current state is {:?}", current_status_for_animating);
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
            info!("Maintain state is - {:?}", state);

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
            info!("Changed state is - {:?}", state);
            match state {
                AnimationState::Idle => {
                    // Transition from Walking → Idle: 200ms
                    // Transition from Running → Idle happens via Walking first
                    let transition_duration = match old_state {
                        Some(AnimationState::Walking) => Duration::from_millis(200),
                        Some(AnimationState::Running) => Duration::from_millis(200),
                        _ => Duration::ZERO,
                    };
                    transitions
                        .play(animation_player, animation_nodes.idle, transition_duration)
                        .repeat();
                }
                AnimationState::Walking => {
                    // Idle → Walking: immediate transition
                    // Running → Walking: 500ms transition (when slowing down)
                    let transition_duration = match old_state {
                        Some(AnimationState::Idle) => Duration::ZERO,
                        Some(AnimationState::Running) => Duration::from_millis(500),
                        _ => Duration::from_millis(200),
                    };
                    transitions
                        .play(animation_player, animation_nodes.walk, transition_duration)
                        .repeat();
                }
                AnimationState::Running => {
                    // Walking → Running: 500ms transition (after 1 second of walking)
                    transitions
                        .play(animation_player, animation_nodes.run, Duration::from_millis(500))
                        .repeat();
                },
                AnimationState::Jumping => {
                    // Play appropriate jump animation based on previous state
                    match old_state {
                        Some(AnimationState::Walking) | Some(AnimationState::Running) => {
                            // Running jump when jumping while moving
                            transitions
                                .play(
                                    animation_player,
                                    animation_nodes.running_jump,
                                    Duration::from_millis(50),
                                )
                                .set_speed(1.2);
                            info!("Playing running jump (one-shot)");
                        }
                        Some(AnimationState::Idle) | None => {
                            // Standing jump when jumping from idle or initial state
                            transitions.play(
                                animation_player,
                                animation_nodes.jump,
                                Duration::from_millis(50),
                            );
                            info!("Playing standing jump (one-shot)");
                        }
                        _ => {
                            // Default to standing jump for any other state
                            transitions.play(
                                animation_player,
                                animation_nodes.jump,
                                Duration::from_millis(50),
                            );
                        }
                    }
                }
                // PARKOUR ANIMATIONS
                AnimationState::Vaulting => {
                    // Use Mixamo vault animation
                    transitions
                        .play(animation_player, animation_nodes.vault, Duration::from_millis(100))
                        .set_speed(1.0);
                    info!("🏃 Playing vault animation");
                }
                AnimationState::Climbing => {
                    // Use Mixamo climb animation
                    transitions
                        .play(animation_player, animation_nodes.climb, Duration::from_millis(100))
                        .set_speed(1.0);
                    info!("🧗 Playing climb animation");
                }
                AnimationState::Sliding => {
                    // Use Mixamo slide animation
                    transitions
                        .play(animation_player, animation_nodes.slide, Duration::from_millis(100))
                        .set_speed(1.0);
                    info!("🛝 Playing slide animation");
                }
                AnimationState::WallRunning => {
                    // Use Mixamo wall run animation
                    transitions
                        .play(animation_player, animation_nodes.wall_run, Duration::from_millis(100))
                        .repeat()
                        .set_speed(1.0);
                    info!("🏃‍♂️ Playing wall run animation");
                }
            }
        }
    }
}
