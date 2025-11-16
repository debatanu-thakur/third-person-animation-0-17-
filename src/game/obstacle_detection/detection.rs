use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua::builtins::TnuaBuiltinWalk;

use crate::{game::player::Player, screens::Screen};

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
            center_ray_height: 1.0,  // Chest height
            upper_ray_height: 1.8,   // Above head / ledge detection
            lower_ray_height: 0.3,   // Foot level
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

/// Current parkour animation state
#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParkourState {
    #[default]
    Idle,
    Walking,
    Running,
    Sprinting,
    /// Vaulting over obstacle
    Vaulting,
    /// Climbing up wall
    Climbing,
    /// Hanging on ledge
    Hanging,
    /// Wall running
    WallRunning,
    /// Sliding under/on obstacle
    Sliding,
    /// Jumping over gap
    Jumping,
    /// Landing from height
    Landing,
}

/// Component to track player's parkour state
#[derive(Component, Default)]
pub struct ParkourController {
    pub state: ParkourState,
    pub can_vault: bool,
    pub can_climb: bool,
    pub can_wall_run: bool,
    pub can_slide: bool,
}

// ============================================================================
// DETECTION SYSTEMS
// ============================================================================

/// Multi-ray raycasting system to detect obstacles ahead of player
pub fn detect_obstacles(
    mut player_query: Query<
        (&Transform, &LinearVelocity, &mut ObstacleDetectionResult),
        With<Player>,
    >,
    config: Res<ObstacleDetectionConfig>,
    spatial_query: SpatialQuery,
    mut gizmos: Gizmos,
) {
    for (transform, velocity, mut detection) in player_query.iter_mut() {
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

        // Cast rays
        let center_hit = spatial_query.cast_ray(
            center_origin,
            ray_direction,
            max_distance,
            true,
            &SpatialQueryFilter::default(),
        );

        let upper_hit = spatial_query.cast_ray(
            upper_origin,
            ray_direction,
            max_distance,
            true,
            &SpatialQueryFilter::default(),
        );

        let lower_hit = spatial_query.cast_ray(
            lower_origin,
            ray_direction,
            max_distance,
            true,
            &SpatialQueryFilter::default(),
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

/// PLACEHOLDER: Trigger parkour animations based on input and detection
/// This will be expanded with actual animation integration
pub fn trigger_parkour_actions(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<
        (
            &ObstacleDetectionResult,
            &mut ParkourController,
            &LinearVelocity,
        ),
        With<Player>,
    >,
) {
    for (detection, mut parkour, velocity) in player_query.iter_mut() {
        let speed = velocity.length();

        // Update movement state based on velocity
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

// ============================================================================
// SIMPLIFIED ROOT MOTION - Apply forward movement during parkour
// ============================================================================

/// Applies forward movement during parkour animations (simplified root motion)
/// This moves the character forward during vault/climb/slide animations
pub fn apply_parkour_root_motion(
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

// ============================================================================
// ANIMATION COMPLETION DETECTION
// ============================================================================

/// Component to track parkour animation timing
#[derive(Component)]
pub struct ParkourAnimationState {
    /// The parkour state being animated
    pub current_state: ParkourState,
    /// Time when this animation started
    pub start_time: f32,
}

/// Starts tracking when a parkour animation begins
pub fn start_parkour_animation_tracking(
    mut commands: Commands,
    mut player_query: Query<(Entity, &ParkourController, Option<&mut ParkourAnimationState>), (With<Player>, Changed<ParkourController>)>,
    time: Res<Time>,
) {
    for (entity, parkour, anim_state) in player_query.iter_mut() {
        let is_parkour_action = matches!(
            parkour.state,
            ParkourState::Vaulting
                | ParkourState::Climbing
                | ParkourState::Sliding
                | ParkourState::WallRunning
        );

        if is_parkour_action {
            // Just entered a parkour state
            if let Some(mut state) = anim_state {
                // Update existing state if changed
                if state.current_state != parkour.state {
                    state.current_state = parkour.state;
                    state.start_time = time.elapsed_secs();
                    info!("🎬 Started parkour animation: {:?}", parkour.state);
                }
            } else {
                // Add new tracking component
                commands.entity(entity).insert(ParkourAnimationState {
                    current_state: parkour.state,
                    start_time: time.elapsed_secs(),
                });
                info!("🎬 Started parkour animation: {:?}", parkour.state);
            }
        } else if anim_state.is_some() {
            // Exited parkour, remove tracking
            commands.entity(entity).remove::<ParkourAnimationState>();
        }
    }
}

/// Detects when parkour animations complete and transitions back to locomotion
/// For now uses fixed durations, later will query AnimationPlayer for actual clip length
pub fn detect_parkour_animation_completion(
    mut player_query: Query<(&mut ParkourController, &ParkourAnimationState), With<Player>>,
    time: Res<Time>,
) {
    for (mut parkour, anim_state) in player_query.iter_mut() {
        let elapsed = time.elapsed_secs() - anim_state.start_time;

        // Fixed durations for parkour animations (in seconds)
        // TODO: Get actual clip duration from AnimationPlayer/Assets
        let animation_duration = match parkour.state {
            ParkourState::Vaulting => 1.5,  // Vault takes ~1.5 seconds
            ParkourState::Climbing => 2.0,  // Climb takes ~2 seconds
            ParkourState::Sliding => 1.2,   // Slide takes ~1.2 seconds
            ParkourState::WallRunning => 99999.0, // Wall run is continuous
            _ => 0.0,
        };

        // Check if animation completed
        if elapsed >= animation_duration {
            info!("✅ Parkour animation completed ({}s), returning to locomotion", elapsed);
            parkour.state = ParkourState::Idle;
        }
    }
}

