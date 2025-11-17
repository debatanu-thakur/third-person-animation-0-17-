//! Hand placement system for detecting walls and placing hands on surfaces

use bevy::prelude::*;
use avian3d::prelude::*;

use super::{
    player::Player,
    target_matching::{BoneMap, TargetBone, TargetMatchRequest},
};

/// Component that enables automatic hand placement on walls
#[derive(Component, Clone, Debug)]
pub struct HandPlacementEnabled {
    /// How far forward to raycast for walls
    pub raycast_distance: f32,

    /// Offset from wall surface to place hands
    pub hand_offset: f32,

    /// How often to update hand positions (seconds)
    pub update_interval: f32,

    /// Internal timer
    #[doc(hidden)]
    pub timer: Timer,
}

impl Default for HandPlacementEnabled {
    fn default() -> Self {
        Self {
            raycast_distance: 1.5,
            hand_offset: 0.1,
            update_interval: 0.1,
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
        }
    }
}

impl HandPlacementEnabled {
    pub fn for_testing() -> Self {
        Self {
            raycast_distance: 2.0,
            hand_offset: 0.05,
            update_interval: 0.05,
            timer: Timer::from_seconds(0.05, TimerMode::Repeating),
        }
    }
}

/// Plugin for hand placement system
pub struct HandPlacementPlugin;

impl Plugin for HandPlacementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_hand_placement);
    }
}

/// System that detects walls in front of hands and requests target matching
fn update_hand_placement(
    mut commands: Commands,
    time: Res<Time>,
    spatial_query: SpatialQuery,
    mut players: Query<
        (Entity, &GlobalTransform, &BoneMap, &mut HandPlacementEnabled),
        With<Player>,
    >,
    hand_transforms: Query<&GlobalTransform>,
) {
    for (player_entity, player_transform, bone_map, mut hand_placement) in players.iter_mut() {
        // Update timer
        hand_placement.timer.tick(time.delta());

        if !hand_placement.timer.just_finished() {
            continue;
        }

        // Get forward direction of player
        let forward = player_transform.forward();

        // Check left hand
        if let Some(left_hand_entity) = bone_map.get(TargetBone::LeftHand) {
            if let Ok(hand_transform) = hand_transforms.get(left_hand_entity) {
                let hand_pos = hand_transform.translation();

                if let Some(wall_pos) = raycast_for_wall(
                    &spatial_query,
                    hand_pos,
                    forward.as_vec3(),
                    hand_placement.raycast_distance,
                    hand_placement.hand_offset,
                    player_entity,
                ) {
                    info!("Left hand raycast hit wall at: {:?}", wall_pos);

                    // Create target match request
                    commands.entity(player_entity).insert(
                        TargetMatchRequest::new(
                            TargetBone::LeftHand,
                            wall_pos,
                            0.5, // 0.5 second animation duration
                        )
                    );
                }
            }
        }

        // Check right hand
        if let Some(right_hand_entity) = bone_map.get(TargetBone::RightHand) {
            if let Ok(hand_transform) = hand_transforms.get(right_hand_entity) {
                let hand_pos = hand_transform.translation();

                if let Some(wall_pos) = raycast_for_wall(
                    &spatial_query,
                    hand_pos,
                    forward.as_vec3(),
                    hand_placement.raycast_distance,
                    hand_placement.hand_offset,
                    player_entity,
                ) {
                    info!("Right hand raycast hit wall at: {:?}", wall_pos);

                    // Create target match request
                    commands.entity(player_entity).insert(
                        TargetMatchRequest::new(
                            TargetBone::RightHand,
                            wall_pos,
                            0.5, // 0.5 second animation duration
                        )
                    );
                }
            }
        }
    }
}

/// Raycast forward from a hand position to find walls
fn raycast_for_wall(
    spatial_query: &SpatialQuery,
    from_position: Vec3,
    forward_direction: Vec3,
    max_distance: f32,
    offset: f32,
    player_entity: Entity,
) -> Option<Vec3> {
    // Exclude player entity from raycast
    let filter = SpatialQueryFilter::from_excluded_entities([player_entity]);

    // Cast ray forward from hand position
    if let Some(hit) = spatial_query.cast_ray(
        from_position,
        Direction3d::new(forward_direction).ok()?,
        max_distance,
        true,
        &filter,
    ) {
        // Calculate hit point and offset it slightly away from wall
        let hit_point = from_position + forward_direction * hit.distance;
        let wall_normal = hit.normal;

        // Place hand slightly in front of wall surface
        Some(hit_point + wall_normal * offset)
    } else {
        None
    }
}
