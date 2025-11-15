use bevy::prelude::*;
use bevy_tnua::{TnuaAnimatingState, TnuaAnimatingStateDirective, prelude::*};

use crate::game::{
    configs::AnimationBlendingConfig,
    player::{Player, PlayerAssets},
};

use super::{models::AnimationState, animation_controller::determine_animation_state};

/// Resource to track the previous animation state before jumping
/// This allows us to return to the correct state after jump completes
#[derive(Resource, Default)]
pub struct PreviousAnimationState {
    pub state: Option<AnimationState>,
    pub was_moving: bool,
}

/// Stores the indices of animation nodes in the animation graph
#[derive(Resource)]
pub struct AnimationNodes {
    pub idle: AnimationNodeIndex,
    pub walk: AnimationNodeIndex,
    pub run: AnimationNodeIndex,
    pub movement_blend: AnimationNodeIndex,  // Blend node for walk-run
    pub standing_jump: AnimationNodeIndex,
    pub running_jump: AnimationNodeIndex,
}

/// Creates the animation graph with proper blending structure
///
/// Graph structure:
/// Root
///   ├─ Idle (direct child of root)
///   ├─ Movement Blend Node (blends between idle and movement)
///   │   ├─ Walk
///   │   └─ Run
///   ├─ Jump
///   └─ Fall
pub fn setup_animation_graph(
    mut commands: Commands,
    player_assets: Option<Res<PlayerAssets>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    animation_nodes: Option<Res<AnimationNodes>>,
    animation_player_query: Query<Entity, With<AnimationPlayer>>,
) {
    // If animation nodes exist, no need to process this anymore
    if animation_nodes.is_some() {
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

    // Create movement blend node (for walk-run blending)
    // Initial weight 0.0 means idle at start
    let movement_blend_node = graph.add_blend(0.0, root_node);

    // Add clips to the graph
    // Idle is direct child of root
    let idle_node = graph.add_clip(animations.idle.clone(), 1.0, root_node);

    // Walk and run are children of the movement blend node
    let walk_node = graph.add_clip(animations.walking.clone(), 1.0, movement_blend_node);
    let run_node = graph.add_clip(animations.running.clone(), 1.0, movement_blend_node);

    // Jump animations are children of root (includes full jump and landing sequence)
    let standing_jump_node = graph.add_clip(animations.standing_jump.clone(), 1.0, root_node);
    let running_jump_node = graph.add_clip(animations.running_jump.clone(), 1.0, root_node);

    // Store the graph and node indices
    let graph_handle = graphs.add(graph);

    commands.insert_resource(AnimationNodes {
        idle: idle_node,
        walk: walk_node,
        run: run_node,
        movement_blend: movement_blend_node,
        standing_jump: standing_jump_node,
        running_jump: running_jump_node,
    });

    // Store the graph handle
    commands.entity(animation_player_entity).insert(AnimationGraphHandle(graph_handle));

    info!("Animation graph with blending successfully created!");
}

/// Updates animation state based on Tnua controller state
pub fn update_animation_state(
    mut player_query: Query<
        (&TnuaController, &mut TnuaAnimatingState<AnimationState>),
        With<Player>,
    >,
    mut animation_player_query: Query<&mut AnimationPlayer>,
    animation_nodes: Option<Res<AnimationNodes>>,
    blend_config: Res<AnimationBlendingConfig>,
    mut previous_state: ResMut<PreviousAnimationState>,
) {
    let Ok(mut animation_player) = animation_player_query.single_mut() else {
        return;
    };
    let Some(animation_nodes) = animation_nodes else {
        return;
    };

    for (controller, mut animating_state) in player_query.iter_mut() {
        // Determine the new state from Tnua controller
        let new_state = determine_animation_state(controller);

        // Update animating state
        let animating_directive = animating_state.update_by_discriminant(new_state);

        // Apply blending based on directive
        match animating_directive {
            TnuaAnimatingStateDirective::Maintain { state } => {
                // State variant unchanged, just update blending
                apply_animation_blending(&mut animation_player, &animation_nodes, *state, &blend_config, &mut previous_state);
            }
            TnuaAnimatingStateDirective::Alter { old_state, state } => {
                // State variant changed, transition to new animation
                // Store previous state if transitioning to jump
                if matches!(state, AnimationState::Jumping) {
                    previous_state.state = old_state;
                    previous_state.was_moving = old_state.map_or(false, |s| matches!(s, AnimationState::Moving(_)));
                }
                apply_animation_blending(&mut animation_player, &animation_nodes, *state, &blend_config, &mut previous_state);
            }
        }
    }
}

/// Applies animation blending based on the current state
fn apply_animation_blending(
    animation_player: &mut AnimationPlayer,
    animation_nodes: &AnimationNodes,
    state: AnimationState,
    config: &AnimationBlendingConfig,
    previous_state: &mut PreviousAnimationState,
) {
    match state {
        AnimationState::Idle => {
            // Idle: play idle animation, movement blend weight = 0
            ensure_animation_playing(animation_player, animation_nodes.idle);

            // Set movement blend weight to 0 (fully idle)
            if let Some(blend_anim) = animation_player.animation_mut(animation_nodes.movement_blend) {
                blend_anim.set_weight(0.0);
            }

            // Set idle weight to 1.0
            if let Some(idle_anim) = animation_player.animation_mut(animation_nodes.idle) {
                idle_anim.set_weight(1.0);
            }
        }
        AnimationState::Moving(speed) => {
            // Moving: blend between idle and movement based on speed
            // Within movement, blend between walk and run based on speed

            ensure_animation_playing(animation_player, animation_nodes.idle);
            ensure_animation_playing(animation_player, animation_nodes.walk);
            ensure_animation_playing(animation_player, animation_nodes.run);

            // Get thresholds from config
            let idle_threshold = config.speed_thresholds.idle_threshold;
            let walk_speed = config.speed_thresholds.walk_speed;
            let run_speed = config.speed_thresholds.run_speed;

            // Movement blend weight: 0 at idle threshold, 1 at walk speed and above
            let movement_blend_weight = ((speed - idle_threshold) / (walk_speed - idle_threshold))
                .clamp(0.0, 1.0);

            // Walk-run blend within movement: 0 = all walk, 1 = all run
            let walk_run_factor = ((speed - walk_speed) / (run_speed - walk_speed))
                .clamp(0.0, 1.0);

            // Set blend node weight (controls idle vs movement)
            if let Some(blend_anim) = animation_player.animation_mut(animation_nodes.movement_blend) {
                blend_anim.set_weight(movement_blend_weight);
            }

            // Set idle weight (inverse of movement)
            if let Some(idle_anim) = animation_player.animation_mut(animation_nodes.idle) {
                idle_anim.set_weight(1.0 - movement_blend_weight);
            }

            // Set walk and run weights within the blend node
            let walk_weight = 1.0 - walk_run_factor;
            let run_weight = walk_run_factor;

            if let Some(walk_anim) = animation_player.animation_mut(animation_nodes.walk) {
                walk_anim.set_weight(walk_weight);
            }
            if let Some(run_anim) = animation_player.animation_mut(animation_nodes.run) {
                run_anim.set_weight(run_weight);
            }
        }
        AnimationState::Jumping => {
            // Choose jump animation based on whether we were moving
            let jump_node = if previous_state.was_moving {
                animation_nodes.running_jump
            } else {
                animation_nodes.standing_jump
            };

            // Play jump animation ONCE (no repeat)
            // Check if the jump animation is already playing
            if !animation_player.is_playing_animation(jump_node) {
                animation_player.play(jump_node);
                info!("Started {} animation (one-shot)",
                    if previous_state.was_moving { "running jump" } else { "standing jump" });
            }

            if let Some(jump_anim) = animation_player.animation_mut(jump_node) {
                jump_anim.set_weight(1.0);
            }

            // Stop other animations
            stop_animation(animation_player, animation_nodes.idle);
            stop_animation(animation_player, animation_nodes.walk);
            stop_animation(animation_player, animation_nodes.run);

            // Stop the other jump animation if it's playing
            let other_jump_node = if previous_state.was_moving {
                animation_nodes.standing_jump
            } else {
                animation_nodes.running_jump
            };
            stop_animation(animation_player, other_jump_node);
        }
        _ => {
            
        }
    }
}

/// Ensures an animation is playing, starting it if necessary
fn ensure_animation_playing(animation_player: &mut AnimationPlayer, node_index: AnimationNodeIndex) {
    if !animation_player.is_playing_animation(node_index) {
        animation_player.play(node_index).repeat();
    }
}

/// Stops an animation if it's currently playing
fn stop_animation(animation_player: &mut AnimationPlayer, node_index: AnimationNodeIndex) {
    if animation_player.is_playing_animation(node_index) {
        animation_player.stop(node_index);
    }
}
