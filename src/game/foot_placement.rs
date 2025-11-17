//! Dynamic foot placement system for slopes and uneven terrain
//!
//! This module provides automatic foot adjustment using the target matching plugin.
//! Feet will raycast down to find ground and adjust position accordingly.

use avian3d::prelude::*;
use bevy::prelude::*;

use super::player::Player;
use super::target_matching::{BoneMap, TargetBone, TargetMatchRequest};
use crate::screens::Screen;

/// Plugin for dynamic foot placement on slopes
pub struct FootPlacementPlugin;

impl Plugin for FootPlacementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_foot_placement.run_if(in_state(Screen::Gameplay)),
        );

        info!("FootPlacementPlugin initialized");
    }
}

/// Component to enable dynamic foot placement on a character
#[derive(Component, Debug)]
pub struct FootPlacementEnabled {
    /// Maximum distance to raycast downward for ground detection
    pub raycast_distance: f32,

    /// How high above the detected ground to place the foot (prevents clipping)
    pub foot_offset: f32,

    /// How frequently to update foot placement (in seconds)
    pub update_interval: f32,

    /// Minimum slope angle (degrees) before foot placement activates
    /// Set to 0.0 to always use foot placement
    pub min_slope_angle: f32,

    /// Internal timer for update intervals
    timer: Timer,
}

impl Default for FootPlacementEnabled {
    fn default() -> Self {
        Self {
            raycast_distance: 2.0,
            foot_offset: 0.05, // 5cm above ground
            update_interval: 0.1, // 10 updates per second
            min_slope_angle: 5.0, // Only activate on slopes > 5 degrees
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
        }
    }
}

impl FootPlacementEnabled {
    /// Create with custom settings
    pub fn new(raycast_distance: f32, foot_offset: f32, update_interval: f32) -> Self {
        Self {
            raycast_distance,
            foot_offset,
            update_interval,
            timer: Timer::from_seconds(update_interval, TimerMode::Repeating),
            ..default()
        }
    }

    /// Create for gentle slopes (more sensitive, smoother)
    pub fn for_gentle_slopes() -> Self {
        Self {
            raycast_distance: 1.5,
            foot_offset: 0.02,
            update_interval: 0.05, // 20 updates per second
            min_slope_angle: 2.0,
            timer: Timer::from_seconds(0.05, TimerMode::Repeating),
        }
    }

    /// Create for steep terrain (less frequent updates, better performance)
    pub fn for_steep_terrain() -> Self {
        Self {
            raycast_distance: 3.0,
            foot_offset: 0.08,
            update_interval: 0.15,
            min_slope_angle: 10.0,
            timer: Timer::from_seconds(0.15, TimerMode::Repeating),
        }
    }

    /// Create for testing - always active, even on flat ground
    pub fn for_testing() -> Self {
        Self {
            raycast_distance: 5.0,   // Look far down
            foot_offset: 0.05,
            update_interval: 0.1,
            min_slope_angle: 0.0,    // ALWAYS ACTIVE - no slope requirement
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
        }
    }
}

/// System that detects ground beneath feet and requests target matching
fn update_foot_placement(
    mut commands: Commands,
    time: Res<Time>,
    spatial_query: SpatialQuery,
    mut players: Query<
        (Entity, &GlobalTransform, &BoneMap, &mut FootPlacementEnabled),
        With<Player>,
    >,
    foot_transforms: Query<&GlobalTransform>,
) {
    for (player_entity, player_transform, bone_map, mut foot_placement) in players.iter_mut() {
        // Update timer
        foot_placement.timer.tick(time.delta());

        if !foot_placement.timer.just_finished() {
            continue;
        }

        // Debug: Check if bone map is populated
        if bone_map.bones.is_empty() {
            warn!("BoneMap is empty for player {:?}", player_entity);
            continue;
        }

        trace!("Foot placement update - bone map has {} bones", bone_map.bones.len());

        // Optionally check if we're on a slope before activating
        if foot_placement.min_slope_angle > 0.0 {
            if let Some(ground_normal) = detect_ground_normal(
                &spatial_query,
                player_transform,
                player_entity,
            ) {
                let slope_angle = ground_normal.angle_between(Vec3::Y).to_degrees();

                if slope_angle < foot_placement.min_slope_angle {
                    // On flat ground, skip foot placement
                    continue;
                }
            }
        }

        // Process left foot
        if let Some(left_foot_entity) = bone_map.get(TargetBone::LeftFoot) {
            trace!("Found left foot entity: {:?}", left_foot_entity);
            if let Ok(left_foot_transform) = foot_transforms.get(left_foot_entity) {
                trace!("Left foot position: {:?}", left_foot_transform.translation());
                if let Some(target_pos) = raycast_for_ground(
                    &spatial_query,
                    left_foot_transform.translation(),
                    foot_placement.raycast_distance,
                    foot_placement.foot_offset,
                    player_entity, // Exclude player from raycast
                ) {
                    info!("Left foot raycast hit ground at: {:?}", target_pos);
                    commands.entity(player_entity).insert(TargetMatchRequest::new(
                        TargetBone::LeftFoot,
                        target_pos,
                        foot_placement.update_interval,
                    ));
                } else {
                    trace!("Left foot raycast missed ground");
                }
            } else {
                warn!("Left foot entity has no GlobalTransform");
            }
        } else {
            warn!("Left foot not found in bone map");
        }

        // Process right foot
        if let Some(right_foot_entity) = bone_map.get(TargetBone::RightFoot) {
            trace!("Found right foot entity: {:?}", right_foot_entity);
            if let Ok(right_foot_transform) = foot_transforms.get(right_foot_entity) {
                trace!("Right foot position: {:?}", right_foot_transform.translation());
                if let Some(target_pos) = raycast_for_ground(
                    &spatial_query,
                    right_foot_transform.translation(),
                    foot_placement.raycast_distance,
                    foot_placement.foot_offset,
                    player_entity, // Exclude player from raycast
                ) {
                    info!("Right foot raycast hit ground at: {:?}", target_pos);
                    commands.entity(player_entity).insert(TargetMatchRequest::new(
                        TargetBone::RightFoot,
                        target_pos,
                        foot_placement.update_interval,
                    ));
                } else {
                    trace!("Right foot raycast missed ground");
                }
            } else {
                warn!("Right foot entity has no GlobalTransform");
            }
        } else {
            warn!("Right foot not found in bone map");
        }
    }
}

/// Raycast downward from a position to find ground
///
/// Excludes the player entity to prevent self-collision
fn raycast_for_ground(
    spatial_query: &SpatialQuery,
    from_position: Vec3,
    max_distance: f32,
    offset: f32,
    player_entity: Entity,
) -> Option<Vec3> {
    let ray_origin = from_position;
    let ray_direction = Dir3::NEG_Y;

    // Create filter that excludes the player entity
    let filter = SpatialQueryFilter::from_excluded_entities([player_entity]);

    trace!(
        "Raycasting from {:?} down for {} units",
        ray_origin,
        max_distance
    );

    // Cast ray downward
    if let Some(hit) = spatial_query.cast_ray(
        ray_origin,
        ray_direction,
        max_distance,
        true, // Should hit all (except excluded)
        &filter,
    ) {
        // Return hit position with offset applied
        // Calculate hit point from ray origin, direction, and distance
        let hit_point = ray_origin + *ray_direction * hit.distance;
        let final_pos = hit_point + Vec3::Y * offset;
        trace!("Raycast hit at distance {}, final position: {:?}", hit.distance, final_pos);
        Some(final_pos)
    } else {
        trace!("Raycast did not hit anything");
        None
    }
}

/// Detect the ground normal beneath the player for slope detection
///
/// Excludes the player entity to prevent self-collision
fn detect_ground_normal(
    spatial_query: &SpatialQuery,
    player_transform: &GlobalTransform,
    player_entity: Entity,
) -> Option<Vec3> {
    let ray_origin = player_transform.translation();
    let ray_direction = Dir3::NEG_Y;

    // Create filter that excludes the player entity
    let filter = SpatialQueryFilter::from_excluded_entities([player_entity]);

    if let Some(hit) = spatial_query.cast_ray(
        ray_origin,
        ray_direction,
        2.0,
        true,
        &filter,
    ) {
        Some(hit.normal)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_foot_placement_defaults() {
        let fp = FootPlacementEnabled::default();
        assert_eq!(fp.raycast_distance, 2.0);
        assert_eq!(fp.foot_offset, 0.05);
        assert_eq!(fp.update_interval, 0.1);
    }

    #[test]
    fn test_gentle_slopes_config() {
        let fp = FootPlacementEnabled::for_gentle_slopes();
        assert!(fp.update_interval < 0.1); // More frequent
        assert!(fp.min_slope_angle < 5.0); // More sensitive
    }
}
