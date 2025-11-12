use bevy::prelude::*;
use bevy_tnua::prelude::*;

use crate::game::player::Player;

use super::{
    animation_assets::PlayerAnimations,
    models::{AnimationState, CharacterAnimationController},
};

/// Stores the indices of animation nodes in the animation graph
#[derive(Resource)]
pub struct AnimationNodes {
    pub idle: AnimationNodeIndex,
    pub run: AnimationNodeIndex,
    pub jump: AnimationNodeIndex,
    pub fall: AnimationNodeIndex,
}

/// Creates the animation graph with all clips and transitions
pub fn setup_animation_graph(
    mut commands: Commands,
    animations: Res<PlayerAnimations>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    animation_clips: Res<Assets<AnimationClip>>,
) {
    // Check if animation clips are actually loaded
    let clips_loaded = animation_clips.get(&animations.idle).is_some()
        && animation_clips.get(&animations.run).is_some()
        && animation_clips.get(&animations.jump).is_some()
        && animation_clips.get(&animations.falling).is_some();

    if !clips_loaded {
        warn!("Animation clips not yet loaded or invalid. Skipping animation graph setup.");
        warn!("Make sure you have downloaded actual Mixamo animations (not placeholder files).");
        return;
    }

    let mut graph = AnimationGraph::new();

    // Add all animation clips as nodes
    let idle_node = graph.add_clip(animations.idle.clone(), 1.0, graph.root);
    let run_node = graph.add_clip(animations.run.clone(), 1.0, graph.root);
    let jump_node = graph.add_clip(animations.jump.clone(), 1.0, graph.root);
    let fall_node = graph.add_clip(animations.falling.clone(), 1.0, graph.root);

    // Store the graph and node indices
    let graph_handle = graphs.add(graph);

    commands.insert_resource(AnimationNodes {
        idle: idle_node,
        run: run_node,
        jump: jump_node,
        fall: fall_node,
    });

    // Store the graph handle as a resource for easy access
    commands.insert_resource(AnimationGraphHandle(graph_handle));

    info!("Animation graph successfully created with Mixamo animations!");
}

/// Resource to store the animation graph handle
#[derive(Resource)]
pub struct AnimationGraphHandle(pub Handle<AnimationGraph>);

/// Attaches animation controller to newly spawned players
pub fn attach_animation_controller(
    mut commands: Commands,
    player_query: Query<(Entity, &Children), (With<Player>, Without<CharacterAnimationController>)>,
    scene_query: Query<&Children>,
    animation_player_query: Query<Entity, With<AnimationPlayer>>,
    graph_handle: Option<Res<AnimationGraphHandle>>,
) {
    // Wait until animation graph is ready
    let Some(graph_handle) = graph_handle else {
        return;
    };

    for (player_entity, children) in player_query.iter() {
        // Find the AnimationPlayer in the character's scene hierarchy
        let mut animation_player_entity = None;

        for child in children.iter() {
            if let Ok(grandchildren) = scene_query.get(child) {
                for grandchild in grandchildren.iter() {
                    if animation_player_query.contains(grandchild) {
                        animation_player_entity = Some(grandchild);
                        break;
                    }
                }
            }
            if animation_player_entity.is_some() {
                break;
            }
        }

        if let Some(anim_entity) = animation_player_entity {
            commands.entity(player_entity).insert((
                AnimationState::Idle,
                CharacterAnimationController {
                    graph: graph_handle.0.clone(),
                    animation_player: anim_entity,
                },
            ));

            info!("Animation controller attached to player");
        }
    }
}

/// Updates animation state based on Tnua controller state
pub fn update_animation_state(
    mut player_query: Query<
        (&TnuaController, &mut AnimationState),
        With<Player>,
    >,
) {
    for (controller, mut anim_state) in player_query.iter_mut() {
        let new_state = determine_animation_state(controller);

        if *anim_state != new_state {
            *anim_state = new_state;
        }
    }
}

/// Determines which animation state to use based on Tnua controller
fn determine_animation_state(controller: &TnuaController) -> AnimationState {
    // Check if character has a walking basis
    if let Some((_basis, basis_state)) = controller.concrete_basis::<TnuaBuiltinWalk>() {
        // Check if grounded by checking standing_offset magnitude
        // Small standing_offset means character is close to/on the ground
        let is_grounded = basis_state.standing_offset.length() < 0.1;

        if is_grounded {
            // Grounded - check if moving based on running velocity
            let is_moving = basis_state.running_velocity.length_squared() > 0.01;
            if is_moving {
                AnimationState::Running
            } else {
                AnimationState::Idle
            }
        } else {
            // In air - check vertical velocity to distinguish jump from fall
            let vertical_velocity = basis_state.running_velocity.y;
            if vertical_velocity > 0.1 {
                AnimationState::Jumping
            } else {
                AnimationState::Falling
            }
        }
    } else {
        // No basis state - default to idle
        AnimationState::Idle
    }
}

/// Applies the current animation state to the animation player with blending
pub fn apply_animation_state(
    player_query: Query<(&AnimationState, &CharacterAnimationController), (With<Player>, Changed<AnimationState>)>,
    mut animation_players: Query<&mut AnimationPlayer>,
    nodes: Res<AnimationNodes>,
) {
    for (anim_state, controller) in player_query.iter() {
        if let Ok(mut player) = animation_players.get_mut(controller.animation_player) {
            let node_index = match anim_state {
                AnimationState::Idle => nodes.idle,
                AnimationState::Running => nodes.run,
                AnimationState::Jumping => nodes.jump,
                AnimationState::Falling => nodes.fall,
            };

            // Start the animation and repeat it (Bevy 0.17 API)
            player.start(node_index).repeat();
        }
    }
}
