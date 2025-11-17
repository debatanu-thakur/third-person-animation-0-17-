use avian3d::{parry::na::inf, prelude::*};
use bevy::prelude::*;
use bevy::animation::AnimationEvent;
use bevy_tnua::prelude::*;
use bevy_tnua::builtins::TnuaBuiltinWalk;

use crate::{game::{parkour_animations::animations::{ParkourController, ParkourState, PlayingParkourAnimation}, player::Player}, screens::Screen};

// ============================================================================
// OBSTACLE TAGS - Add these to scene objects to classify them
// ============================================================================

/// Marker for walls that can be climbed (1.5m - 3m height)
#[derive(Component, Debug, Clone, Copy)]
pub struct ClimbableWall;

/// Marker for obstacles that can be vaulted over (0.5m - 1.5m height)
#[derive(Component, Debug, Clone, Copy)]
pub struct VaultableObstacle;

/// Marker for surfaces the player can slide under or slide on
#[derive(Component, Debug, Clone, Copy)]
pub struct SlideableObstacle;

/// Marker for vertical surfaces suitable for wall running
#[derive(Component, Debug, Clone, Copy)]
pub struct WallRunSurface;

/// Marker for gaps/edges where player can fall
#[derive(Component, Debug, Clone, Copy)]
pub struct Gap;

// ============================================================================
// DETECTION CONFIGURATION
// ============================================================================

/// Configuration for obstacle detection raycasting
#[derive(Resource)]
pub struct ObstacleDetectionConfig {
    /// How far ahead to detect obstacles (meters)
    pub detection_range: f32,
    /// Minimum velocity to trigger automatic actions (slide, wall run)
    pub min_velocity_for_auto_actions: f32,
    /// Height offset for center ray (from player origin)
    pub center_ray_height: f32,
    /// Height offset for upper ray (ledge detection)
    pub upper_ray_height: f32,
    /// Height offset for lower ray (gap/low obstacle detection)
    pub lower_ray_height: f32,
    /// Enable debug visualization of raycasts
    pub debug_draw_rays: bool,
}

impl Default for ObstacleDetectionConfig {
    fn default() -> Self {
        Self {
            detection_range: 2.0,
            min_velocity_for_auto_actions: 3.0,
            center_ray_height: 0.3,  // Chest height
            upper_ray_height: 0.9,   // Above head / ledge detection
            lower_ray_height: -0.6,   // Foot level
            debug_draw_rays: true,
        }
    }
}

// ============================================================================
// OBSTACLE DETECTION RESULTS
// ============================================================================

/// Types of obstacles that can be detected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObstacleType {
    /// No obstacle detected
    None,
    /// Low obstacle - step over or slide under
    LowObstacle,
    /// Medium obstacle - vault over
    MediumObstacle,
    /// Tall wall - climb or wall run
    TallWall,
    /// Ledge above (can hang or pull up)
    Ledge,
    /// Gap in floor (need to jump)
    FloorGap,
    /// Slope/ramp
    Slope,
}

/// Stores the result of obstacle detection for a player
#[derive(Component, Default)]
pub struct ObstacleDetectionResult {
    /// Type of obstacle detected
    pub obstacle_type: ObstacleType,
    /// Distance to the obstacle
    pub distance: f32,
    /// Height of the obstacle (if applicable)
    pub height: f32,
    /// World position where center ray hit
    pub hit_point: Option<Vec3>,
    /// World position where upper ray hit (ledge position for IK)
    pub ledge_point: Option<Vec3>,
    /// World position where lower ray hit
    pub lower_hit_point: Option<Vec3>,
    /// Surface normal of the obstacle
    pub surface_normal: Option<Vec3>,
    /// Entity of the detected obstacle
    pub obstacle_entity: Option<Entity>,
    /// Whether player is in range to interact
    pub in_interaction_range: bool,
}

impl Default for ObstacleType {
    fn default() -> Self {
        ObstacleType::None
    }
}

// ============================================================================
// IK TARGET COMPONENTS - For target matching during animations
// ============================================================================

/// IK target for left hand during climb/vault animations
#[derive(Component)]
pub struct LeftHandIKTarget {
    pub target_position: Vec3,
    pub weight: f32, // 0.0 = use animation, 1.0 = fully match target
}

/// IK target for right hand during climb/vault animations
#[derive(Component)]
pub struct RightHandIKTarget {
    pub target_position: Vec3,
    pub weight: f32,
}

/// IK target for left foot (for landing animations)
#[derive(Component)]
pub struct LeftFootIKTarget {
    pub target_position: Vec3,
    pub weight: f32,
}

/// IK target for right foot (for landing animations)
#[derive(Component)]
pub struct RightFootIKTarget {
    pub target_position: Vec3,
    pub weight: f32,
}

// ============================================================================
// ANIMATION STATE - Tracks which parkour action is active
// ============================================================================


// ============================================================================
// DETECTION SYSTEMS
// ============================================================================

/// Multi-ray raycasting system to detect obstacles ahead of player
pub fn detect_obstacles(
    mut player_query: Query<
        (Entity, &Transform, &LinearVelocity, &mut ObstacleDetectionResult),
        With<Player>,
    >,
    config: Res<ObstacleDetectionConfig>,
    spatial_query: SpatialQuery,
    mut gizmos: Gizmos,
) {
    for (player_entity, transform, velocity, mut detection) in player_query.iter_mut() {
        // Reset detection
        *detection = ObstacleDetectionResult::default();

        // Get player's forward direction
        let forward = transform.forward();
        let forward_vec = *forward; // Convert Dir3 to Vec3
        let player_pos = transform.translation;

        // Define ray origins
        let center_origin = player_pos + Vec3::Y * config.center_ray_height;
        let upper_origin = player_pos + Vec3::Y * config.upper_ray_height;
        let lower_origin = player_pos + Vec3::Y * config.lower_ray_height;

        // Ray direction and distance
        let ray_direction = forward; // Already Dir3
        let max_distance = config.detection_range;

        // Create filter to exclude player entity
        let mut filter = SpatialQueryFilter::default();
        filter.excluded_entities.insert(player_entity);

        // Cast rays
        let center_hit = spatial_query.cast_ray(
            center_origin,
            ray_direction,
            max_distance,
            true,
            &filter,
        );

        let upper_hit = spatial_query.cast_ray(
            upper_origin,
            ray_direction,
            max_distance,
            true,
            &filter,
        );

        let lower_hit = spatial_query.cast_ray(
            lower_origin,
            ray_direction,
            max_distance,
            true,
            &filter,
        );

        // Debug visualization
        if config.debug_draw_rays {
            // Center ray (yellow)
            let center_end = center_origin + forward_vec * max_distance;
            gizmos.line(center_origin, center_end, Color::srgb(1.0, 1.0, 0.0));

            // Upper ray (blue)
            let upper_end = upper_origin + forward_vec * max_distance;
            gizmos.line(upper_origin, upper_end, Color::srgb(0.0, 0.5, 1.0));

            // Lower ray (green)
            let lower_end = lower_origin + forward_vec * max_distance;
            gizmos.line(lower_origin, lower_end, Color::srgb(0.0, 1.0, 0.0));

            // Draw hit points
            if let Some(hit) = center_hit {
                let hit_pos = center_origin + forward_vec * hit.distance;
                gizmos.sphere(
                    Isometry3d::from_translation(hit_pos),
                    0.1,
                    Color::srgb(1.0, 0.0, 0.0),
                );
            }
            if let Some(hit) = upper_hit {
                let hit_pos = upper_origin + forward_vec * hit.distance;
                gizmos.sphere(
                    Isometry3d::from_translation(hit_pos),
                    0.1,
                    Color::srgb(0.0, 0.0, 1.0),
                );
            }
            if let Some(hit) = lower_hit {
                let hit_pos = lower_origin + forward_vec * hit.distance;
                gizmos.sphere(
                    Isometry3d::from_translation(hit_pos),
                    0.1,
                    Color::srgb(0.0, 1.0, 0.0),
                );
            }
        }

        // Analyze ray hits to determine obstacle type
        classify_obstacle(
            center_hit,
            upper_hit,
            lower_hit,
            center_origin,
            upper_origin,
            lower_origin,
            forward_vec,
            &mut detection,
        );

        // Check if in interaction range (closer range for manual actions)
        if let Some(dist) = detection.distance.into() {
            detection.in_interaction_range = dist < 1.5;
        }
    }
}

/// Classify obstacle based on ray hit patterns
fn classify_obstacle(
    center_hit: Option<RayHitData>,
    upper_hit: Option<RayHitData>,
    lower_hit: Option<RayHitData>,
    center_origin: Vec3,
    upper_origin: Vec3,
    lower_origin: Vec3,
    forward: Vec3,
    detection: &mut ObstacleDetectionResult,
) {
    match (center_hit, upper_hit, lower_hit) {
        // All three rays hit - tall wall
        (Some(center), Some(upper), Some(lower)) => {
            detection.obstacle_type = ObstacleType::TallWall;
            detection.distance = center.distance;
            detection.hit_point = Some(center_origin + forward * center.distance);
            detection.ledge_point = Some(upper_origin + forward * upper.distance);
            detection.lower_hit_point = Some(lower_origin + forward * lower.distance);

            // Calculate approximate height
            if let (Some(ledge), Some(lower)) = (detection.ledge_point, detection.lower_hit_point) {
                detection.height = ledge.y - lower.y;
            }
        }

        // Center and lower hit, no upper - medium obstacle (vault)
        (Some(center), None, Some(lower)) => {
            detection.obstacle_type = ObstacleType::MediumObstacle;
            detection.distance = center.distance;
            detection.hit_point = Some(center_origin + forward * center.distance);
            detection.lower_hit_point = Some(lower_origin + forward * lower.distance);

            if let (Some(hit), Some(lower)) = (detection.hit_point, detection.lower_hit_point) {
                detection.height = hit.y - lower.y;
            }
        }

        // Only center hit - low obstacle
        (Some(center), None, None) => {
            detection.obstacle_type = ObstacleType::LowObstacle;
            detection.distance = center.distance;
            detection.hit_point = Some(center_origin + forward * center.distance);
        }

        // Only upper hit - ledge above
        (None, Some(upper), None) => {
            detection.obstacle_type = ObstacleType::Ledge;
            detection.distance = upper.distance;
            detection.ledge_point = Some(upper_origin + forward * upper.distance);
        }

        // Center and upper hit, no lower - might be floating obstacle or gap edge
        (Some(center), Some(upper), None) => {
            detection.obstacle_type = ObstacleType::FloorGap;
            detection.distance = center.distance;
            detection.hit_point = Some(center_origin + forward * center.distance);
        }

        // No hits
        (None, None, None) => {
            detection.obstacle_type = ObstacleType::None;
        }

        // Other patterns - treat as low obstacle for now
        _ => {
            if let Some(center) = center_hit {
                detection.obstacle_type = ObstacleType::LowObstacle;
                detection.distance = center.distance;
                detection.hit_point = Some(center_origin + forward * center.distance);
            }
        }
    }
}

/// System to update parkour controller capabilities based on detection
pub fn update_parkour_capabilities(
    mut player_query: Query<(&ObstacleDetectionResult, &mut ParkourController), With<Player>>,
) {
    for (detection, mut parkour) in player_query.iter_mut() {
        // Reset capabilities
        parkour.can_vault = false;
        parkour.can_climb = false;
        parkour.can_wall_run = false;
        parkour.can_slide = false;

        // Set capabilities based on detected obstacle
        match detection.obstacle_type {
            ObstacleType::MediumObstacle => {
                if detection.in_interaction_range {
                    parkour.can_vault = true;
                }
            }
            ObstacleType::TallWall | ObstacleType::Ledge => {
                if detection.in_interaction_range {
                    parkour.can_climb = true;
                    parkour.can_wall_run = true;
                }
            }
            ObstacleType::LowObstacle => {
                parkour.can_slide = true;
            }
            _ => {}
        }
    }
}

/// Trigger parkour animations based on input and detection
/// CRITICAL: Does NOT update state during active parkour animations
pub fn trigger_parkour_actions(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<
        (
            &ObstacleDetectionResult,
            &mut ParkourController,
            &LinearVelocity,
        ),
        (With<Player>,
        Without<PlayingParkourAnimation>),
    >,
) {
    for (detection, mut parkour, velocity) in player_query.iter_mut() {
        // ⚠️ CRITICAL: Don't update state if parkour animation is active
        // The animation completion system will handle returning to locomotion

        let speed = velocity.length();

        // Update movement state based on velocity (ONLY when not doing parkour)
        parkour.state = match speed {
            s if s < 0.1 => ParkourState::Idle,
            s if s < 2.0 => ParkourState::Walking,
            s if s < 4.0 => ParkourState::Running,
            _ => ParkourState::Sprinting,
        };

        // Automatic actions (slides on slopes, etc.)
        if detection.obstacle_type == ObstacleType::LowObstacle && speed > 3.0 {
            // Auto-slide if running fast enough
            parkour.state = ParkourState::Sliding;
            info!("Auto-sliding under obstacle!");
        }

        // Manual parkour actions (require key press)
        if keyboard.just_pressed(KeyCode::Space) && detection.in_interaction_range {
            match detection.obstacle_type {
                ObstacleType::MediumObstacle if parkour.can_vault => {
                    parkour.state = ParkourState::Vaulting;
                    info!(
                        "Vaulting! Hit point: {:?}, Height: {}",
                        detection.hit_point, detection.height
                    );

                    // TODO: Set IK targets for hands to match obstacle top
                    if let Some(hit_point) = detection.hit_point {
                        info!("IK Target for hands: {:?}", hit_point);
                    }
                }
                ObstacleType::TallWall | ObstacleType::Ledge if parkour.can_climb => {
                    parkour.state = ParkourState::Climbing;
                    info!(
                        "Climbing! Ledge point: {:?}, Height: {}",
                        detection.ledge_point, detection.height
                    );

                    // TODO: Set IK targets for hands to match ledge
                    if let Some(ledge_point) = detection.ledge_point {
                        info!("IK Target for hands: {:?}", ledge_point);
                    }
                }
                _ => {}
            }
        }

        // Wall run (requires running speed and side input)
        if keyboard.pressed(KeyCode::ShiftLeft) && speed > 4.0 && parkour.can_wall_run {
            if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::KeyD) {
                parkour.state = ParkourState::WallRunning;
                info!("Wall running!");
            }
        }
    }
}

/// System to apply IK targets during parkour animations
/// PLACEHOLDER: This will integrate with bevy_ik once animations are set up
pub fn apply_ik_targets(
    player_query: Query<(&ParkourController, &ObstacleDetectionResult), With<Player>>,
) {
    for (parkour, detection) in player_query.iter() {
        match parkour.state {
            ParkourState::Vaulting => {
                // TODO: Apply IK to hands based on detection.hit_point
                // This will use bevy_ik to adjust hand positions
                if let Some(hit_point) = detection.hit_point {
                    // Left hand slightly to the left of hit point
                    // Right hand slightly to the right
                    // Apply IK chain to reach these positions
                }
            }
            ParkourState::Climbing | ParkourState::Hanging => {
                // TODO: Apply IK to hands based on detection.ledge_point
                if let Some(ledge_point) = detection.ledge_point {
                    // Both hands grab the ledge
                    // Apply IK chain to reach ledge position
                }
            }
            _ => {}
        }
    }
}

// ============================================================================
// TNUA CONTROL DURING PARKOUR
// ============================================================================

/// Disables Tnua's physics-based movement during parkour actions
/// This prevents fighting between animation root motion and physics movement
pub fn control_tnua_during_parkour(
    mut player_query: Query<(&ParkourController, &mut TnuaController), With<Player>>,
) {
    for (parkour, mut tnua_controller) in player_query.iter_mut() {
        // Check if we're in a parkour action (not normal locomotion)
        let is_parkour_action = matches!(
            parkour.state,
            ParkourState::Vaulting
                | ParkourState::Climbing
                | ParkourState::Hanging
                | ParkourState::WallRunning
                | ParkourState::Sliding
        );

        if is_parkour_action {
            // Disable Tnua's movement by setting basis to zero velocity
            // This stops physics from moving the character
            tnua_controller.basis(TnuaBuiltinWalk {
                desired_velocity: Vec3::ZERO,
                desired_forward: Dir3::new(Vec3::Z).ok(), // Keep a valid forward direction
                float_height: 1.5,
                ..Default::default()
            });
        }
        // When not in parkour, normal movement controls will set the basis
        // (handled in animations/controls.rs apply_controls)
    }
}

/// Makes rigidbody kinematic position during parkour to allow free Transform manipulation
/// This prevents physics from resetting the character position while animation plays
pub fn control_rigidbody_during_parkour(
    mut commands: Commands,
    player_query: Query<(Entity, &ParkourController), With<Player>>,
) {
    for (player_entity, parkour) in player_query.iter() {
        let is_parkour_action = matches!(
            parkour.state,
            ParkourState::Vaulting
                | ParkourState::Climbing
                | ParkourState::Hanging
                | ParkourState::Sliding
        );

        // if is_parkour_action {
        //     // Make kinematic so we can freely modify Transform
        //     commands.entity(player_entity).insert(RigidBody::Kinematic);
        // } else {
        //     // Restore dynamic for normal physics
        //     commands.entity(player_entity).insert(RigidBody::Dynamic);
        // }
    }
}

// ============================================================================
// ROOT MOTION EXTRACTION - Extract movement from animation root bone
// ============================================================================

/// Component to track root bone position for motion extraction
#[derive(Component)]
pub struct RootMotionTracker {
    /// Position where animation started (player Transform)
    pub animation_start_position: Vec3,
    /// Position of root bone when animation started (relative to player)
    pub root_bone_start_offset: Vec3,
}

impl Default for RootMotionTracker {
    fn default() -> Self {
        Self {
            animation_start_position: Vec3::ZERO,
            root_bone_start_offset: Vec3::ZERO,
        }
    }
}

/// Initializes root motion tracking when parkour animation starts
pub fn init_root_motion_tracker(
    mut commands: Commands,
    player_query: Query<(Entity, &Transform, &ParkourController, &Children, Option<&RootMotionTracker>), (With<Player>, Changed<ParkourController>)>,
    bone_query: Query<(&GlobalTransform, &Name)>,
) {
    for (entity, player_transform, parkour, children, tracker) in player_query.iter() {
        let is_parkour_action = matches!(
            parkour.state,
            ParkourState::Vaulting
                | ParkourState::Climbing
                | ParkourState::Sliding
        );

        if is_parkour_action && tracker.is_none() {
            // Find root bone
            let mut root_bone_pos = Vec3::ZERO;
            for child in children.iter() {
                if let Ok((bone_transform, bone_name)) = bone_query.get(child) {
                    if bone_name.as_str() == "mixamorig12:Hips" {
                        root_bone_pos = bone_transform.translation();
                        break;
                    }
                }
            }

            // Initialize tracker
            commands.entity(entity).insert(RootMotionTracker {
                animation_start_position: player_transform.translation,
                root_bone_start_offset: root_bone_pos - player_transform.translation,
            });
            info!("🎯 Root motion tracker initialized at {:?}", player_transform.translation);
        } else if !is_parkour_action && tracker.is_some() {
            // Remove tracker when exiting parkour
            commands.entity(entity).remove::<RootMotionTracker>();
        }
    }
}

/// Extracts root motion from animation and applies to character Transform
/// This prevents the "snap back" issue where animation moves mesh but not rigidbody
pub fn extract_and_apply_root_motion(
    mut player_query: Query<(&mut Transform, &ParkourController, &Children, &RootMotionTracker), With<Player>>,
    bone_query: Query<(&GlobalTransform, &Name)>,
) {
    for (mut player_transform, parkour, children, tracker) in player_query.iter_mut() {
        // Only extract root motion during parkour animations
        let is_parkour_action = matches!(
            parkour.state,
            ParkourState::Vaulting
                | ParkourState::Climbing
                | ParkourState::Sliding
        );

        if !is_parkour_action {
            continue;
        }

        // Find root bone (Hips bone contains the animation's root motion)
        let mut root_bone_pos: Option<Vec3> = None;

        for (bone_transform, bone_name) in bone_query.iter() {
                if bone_name.as_str() == "mixamorig12:Hips" {
                    root_bone_pos = Some(bone_transform.translation());
                    break;
                }
            }
        info!("player position - {}",player_transform.translation);
        let Some(current_root_pos) = root_bone_pos else {
            continue;
        };

        // Calculate how far the root bone has moved from start
        let root_delta = current_root_pos - (tracker.animation_start_position + tracker.root_bone_start_offset);

        // Apply only horizontal movement to player (XZ plane)
        // Keep Y controlled by physics/gravity
        // player_transform.translation.x = tracker.animation_start_position.x + root_delta.x;
        // player_transform.translation.z = tracker.animation_start_position.z + root_delta.z;
    }
}

/// DEPRECATED: Simplified root motion (causes snap-back)
/// Keeping for reference but should not be used
/// Use extract_and_apply_root_motion() instead
pub fn apply_parkour_root_motion_deprecated(
    mut player_query: Query<(&Transform, &ParkourController, &mut LinearVelocity), With<Player>>,
) {
    for (transform, parkour, mut velocity) in player_query.iter_mut() {
        // Apply forward movement based on parkour action
        let forward_speed = match parkour.state {
            ParkourState::Vaulting => 3.0,   // Move forward at 3 m/s during vault
            ParkourState::Climbing => 1.5,   // Move forward slower during climb
            ParkourState::Sliding => 4.0,    // Slide fast
            ParkourState::WallRunning => 3.5, // Wall run speed
            _ => {
                // Not in parkour, don't override velocity
                continue;
            }
        };

        // Apply forward velocity in character's facing direction
        let forward = transform.forward();
        let forward_movement = *forward * forward_speed;

        // Keep vertical velocity (gravity/jumping) but replace horizontal with parkour motion
        velocity.x = forward_movement.x;
        velocity.z = forward_movement.z;
        // Don't touch velocity.y - let gravity/physics handle vertical
    }
}
