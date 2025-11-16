use bevy::prelude::*;
use avian3d::prelude::*;
use crate::{
    game::{
        obstacle_detection::detection::{ObstacleDetectionResult, ParkourController, ParkourState},
        player::Player,
    }, ik::*, screens::Screen
};

// ============================================================================
// IK TARGET COMPONENTS
// ============================================================================

/// Marker component for the left hand IK target
#[derive(Component)]
pub struct LeftHandIkTarget;

/// Marker component for the right hand IK target
#[derive(Component)]
pub struct RightHandIkTarget;

/// Marker component for the left foot IK target
#[derive(Component)]
pub struct LeftFootIkTarget;

/// Marker component for the right foot IK target
#[derive(Component)]
pub struct RightFootIkTarget;

/// Component that stores IK target positions for parkour actions
#[derive(Component, Default)]
pub struct ParkourIkTargets {
    pub left_hand_target: Option<Vec3>,
    pub right_hand_target: Option<Vec3>,
    pub left_foot_target: Option<Vec3>,
    pub right_foot_target: Option<Vec3>,
    pub active: bool,
}

// ============================================================================
// IK CONFIGURATION
// ============================================================================

/// Configuration for IK system
#[derive(Resource)]
pub struct IkConfig {
    /// Enable IK during parkour
    pub enabled: bool,
    /// Hand offset from obstacle hit point (spread hands apart)
    pub hand_spread: f32,
    /// How high above obstacle to place hands
    pub hand_height_offset: f32,
    /// Enable debug visualization of IK targets
    pub debug_visualization: bool,
}

impl Default for IkConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            hand_spread: 0.3, // 30cm apart
            hand_height_offset: 0.05, // 5cm above obstacle
            debug_visualization: true,
        }
    }
}

/// Configuration for locomotion foot IK
#[derive(Resource)]
pub struct LocomotionIkConfig {
    /// Enable foot IK during locomotion (walk, run)
    pub enabled: bool,
    /// Maximum distance to raycast down for ground
    pub max_ground_distance: f32,
    /// How high to lift foot above ground (prevents clipping)
    pub foot_height_offset: f32,
    /// How much to adjust foot vertically (0.0 = no adjustment, 1.0 = full adjustment)
    pub adjustment_strength: f32,
    /// Enable debug visualization
    pub debug_visualization: bool,
}

impl Default for LocomotionIkConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_ground_distance: 2.0, // Raycast 2m down
            foot_height_offset: 0.05, // 5cm above ground
            adjustment_strength: 1.0, // Full adjustment
            debug_visualization: true,
        }
    }
}

// ============================================================================
// IK SETUP SYSTEM
// ============================================================================

/// System to find and setup IK chains on the player skeleton
/// This runs once after the player model is spawned
pub fn setup_ik_chains(
    mut commands: Commands,
    player_query: Query<Entity, (With<Player>, Without<ParkourIkTargets>)>,
    bone_query: Query<(Entity, &Name)>,
) {
    let Ok(player_entity) = player_query.single() else {
        return;
    };

    // Find the bone entities
    let mut left_hand_bone = None;
    let mut right_hand_bone = None;
    let mut left_foot_bone = None;
    let mut right_foot_bone = None;

    // Find pole targets (for IK joint orientation)
    let mut left_forearm_bone = None;
    let mut right_forearm_bone = None;
    let mut left_leg_bone = None;
    let mut right_leg_bone = None;

    for (entity, name) in bone_query.iter() {
        match name.as_str() {
            "mixamorig12:LeftHand" => left_hand_bone = Some(entity),
            "mixamorig12:RightHand" => right_hand_bone = Some(entity),
            "mixamorig12:LeftFoot" => left_foot_bone = Some(entity),
            "mixamorig12:RightFoot" => right_foot_bone = Some(entity),
            "mixamorig12:LeftForeArm" => left_forearm_bone = Some(entity),
            "mixamorig12:RightForeArm" => right_forearm_bone = Some(entity),
            "mixamorig12:LeftLeg" => left_leg_bone = Some(entity),
            "mixamorig12:RightLeg" => right_leg_bone = Some(entity),
            _ => {}
        }
    }

    // Spawn IK target entities
    let left_hand_target = commands.spawn((
        Name::new("LeftHandIKTarget"),
        LeftHandIkTarget,
        Transform::default(),
        Visibility::Visible,
    )).id();

    let right_hand_target = commands.spawn((
        Name::new("RightHandIKTarget"),
        RightHandIkTarget,
        Transform::default(),
        Visibility::Visible,
    )).id();

    let left_foot_target = commands.spawn((
        Name::new("LeftFootIKTarget"),
        LeftFootIkTarget,
        Transform::default(),
        Visibility::Visible,
    )).id();

    let right_foot_target = commands.spawn((
        Name::new("RightFootIKTarget"),
        RightFootIkTarget,
        Transform::default(),
        Visibility::Visible,
    )).id();

    // Setup IK chains if bones were found
    if let Some(left_hand) = left_hand_bone {
        commands.entity(left_hand).insert(IkConstraint {
            chain_length: 2, // Hand -> Forearm -> Arm
            iterations: 20,
            target: left_hand_target,
            pole_target: left_forearm_bone,
            pole_angle: 0.0,
            enabled: true,
        });
        info!("✓ Set up left hand IK chain");
    }

    if let Some(right_hand) = right_hand_bone {
        commands.entity(right_hand).insert(IkConstraint {
            chain_length: 2,
            iterations: 20,
            target: right_hand_target,
            pole_target: right_forearm_bone,
            pole_angle: 0.0,
            enabled: true,
        });
        info!("✓ Set up right hand IK chain");
    }

    if let Some(left_foot) = left_foot_bone {
        commands.entity(left_foot).insert(IkConstraint {
            chain_length: 2, // Foot -> Leg -> UpLeg
            iterations: 20,
            target: left_foot_target,
            pole_target: left_leg_bone,
            pole_angle: 0.0,
            enabled: false, // Start disabled, enable during specific parkour actions
        });
        info!("✓ Set up left foot IK chain");
    }

    if let Some(right_foot) = right_foot_bone {
        commands.entity(right_foot).insert(IkConstraint {
            chain_length: 2,
            iterations: 20,
            target: right_foot_target,
            pole_target: right_leg_bone,
            pole_angle: 0.0,
            enabled: false,
        });
        info!("✓ Set up right foot IK chain");
    }

    // Add IK targets component to player
    commands.entity(player_entity).insert(ParkourIkTargets::default());

    // Write setup status to debug file
    let mut setup_info = String::new();
    setup_info.push_str("(\n  ik_setup_complete: true,\n");
    setup_info.push_str(&format!("  left_hand_found: {},\n", left_hand_bone.is_some()));
    setup_info.push_str(&format!("  right_hand_found: {},\n", right_hand_bone.is_some()));
    setup_info.push_str(&format!("  left_foot_found: {},\n", left_foot_bone.is_some()));
    setup_info.push_str(&format!("  right_foot_found: {},\n", right_foot_bone.is_some()));
    setup_info.push_str(")\n");
    let _ = std::fs::write("assets/debug/ik_setup.ron", setup_info);

    info!("✅ IK chains setup complete!");
}

// ============================================================================
// IK TARGET UPDATE SYSTEM
// ============================================================================

/// Updates IK target positions based on obstacle detection and parkour state
pub fn update_ik_targets_from_obstacles(
    mut player_query: Query<
        (
            &Transform,
            &ObstacleDetectionResult,
            &ParkourController,
            &mut ParkourIkTargets,
        ),
        With<Player>,
    >,
    mut left_hand_query: Query<&mut Transform, (With<LeftHandIkTarget>, Without<Player>)>,
    mut right_hand_query: Query<&mut Transform, (With<RightHandIkTarget>, Without<Player>, Without<LeftHandIkTarget>)>,
    config: Res<IkConfig>,
) {
    if !config.enabled {
        return;
    }

    let Ok((player_transform, detection, parkour, mut ik_targets)) = player_query.single_mut() else {
        return;
    };

    // Determine if IK should be active based on parkour state
    let should_use_ik = matches!(
        parkour.state,
        ParkourState::Vaulting | ParkourState::Climbing | ParkourState::Hanging
    );

    ik_targets.active = should_use_ik;

    if !should_use_ik {
        return;
    }

    // Calculate IK target positions based on parkour action
    match parkour.state {
        ParkourState::Vaulting => {
            // For vaulting, place hands on top of obstacle
            if let Some(hit_point) = detection.hit_point {
                let obstacle_height = hit_point.y + config.hand_height_offset;
                let player_forward = player_transform.forward();

                // Spread hands to left and right
                let hand_right = player_transform.right();

                ik_targets.left_hand_target = Some(
                    hit_point + *hand_right * config.hand_spread + Vec3::Y * config.hand_height_offset
                );
                ik_targets.right_hand_target = Some(
                    hit_point - *hand_right * config.hand_spread + Vec3::Y * config.hand_height_offset
                );
            }
        }
        ParkourState::Climbing => {
            // For climbing, use ledge point if available
            if let Some(ledge_point) = detection.ledge_point {
                let hand_right = player_transform.right();

                ik_targets.left_hand_target = Some(
                    ledge_point + *hand_right * config.hand_spread
                );
                ik_targets.right_hand_target = Some(
                    ledge_point - *hand_right * config.hand_spread
                );
            }
        }
        ParkourState::Hanging => {
            // Similar to climbing but might be lower
            if let Some(ledge_point) = detection.ledge_point {
                let hand_right = player_transform.right();

                ik_targets.left_hand_target = Some(
                    ledge_point + *hand_right * config.hand_spread - Vec3::Y * 0.2
                );
                ik_targets.right_hand_target = Some(
                    ledge_point - *hand_right * config.hand_spread - Vec3::Y * 0.2
                );
            }
        }
        _ => {}
    }

    // Apply target positions to IK target entities
    if let Some(target_pos) = ik_targets.left_hand_target {
        if let Ok(mut transform) = left_hand_query.single_mut() {
            transform.translation = target_pos;
        }
    }

    if let Some(target_pos) = ik_targets.right_hand_target {
        if let Ok(mut transform) = right_hand_query.single_mut() {
            transform.translation = target_pos;
        }
    }
}

// ============================================================================
// IK ENABLE/DISABLE SYSTEM
// ============================================================================

/// Enable/disable IK constraints based on parkour state
pub fn toggle_ik_constraints(
    player_query: Query<&ParkourIkTargets, (With<Player>, Changed<ParkourIkTargets>)>,
    mut left_hand_constraint: Query<&mut IkConstraint, With<LeftHandIkTarget>>,
    mut right_hand_constraint: Query<&mut IkConstraint, (With<RightHandIkTarget>, Without<LeftHandIkTarget>)>,
) {
    let Ok(ik_targets) = player_query.single() else {
        return;
    };

    // Enable/disable hand IK based on whether we have active targets
    for mut constraint in left_hand_constraint.iter_mut() {
        constraint.enabled = ik_targets.active;
    }

    for mut constraint in right_hand_constraint.iter_mut() {
        constraint.enabled = ik_targets.active;
    }
}

// ============================================================================
// VISUALIZATION SYSTEM
// ============================================================================

/// Debug visualization of IK targets
pub fn visualize_ik_targets(
    ik_targets_query: Query<&ParkourIkTargets, With<Player>>,
    left_hand_query: Query<&Transform, With<LeftHandIkTarget>>,
    right_hand_query: Query<&Transform, With<RightHandIkTarget>>,
    config: Res<IkConfig>,
    mut gizmos: Gizmos,
) {
    if !config.debug_visualization {
        return;
    }

    let Ok(ik_targets) = ik_targets_query.single() else {
        return;
    };

    if !ik_targets.active {
        return;
    }

    // Visualize left hand target
    if let Ok(transform) = left_hand_query.single() {
        gizmos.sphere(
            Isometry3d::from_translation(transform.translation),
            0.08,
            Color::srgb(0.0, 1.0, 1.0), // Cyan
        );

        // Draw cross for better visibility
        let size = 0.1;
        gizmos.line(
            transform.translation + Vec3::X * size,
            transform.translation - Vec3::X * size,
            Color::srgb(0.0, 1.0, 1.0),
        );
        gizmos.line(
            transform.translation + Vec3::Y * size,
            transform.translation - Vec3::Y * size,
            Color::srgb(0.0, 1.0, 1.0),
        );
    }

    // Visualize right hand target
    if let Ok(transform) = right_hand_query.single() {
        gizmos.sphere(
            Isometry3d::from_translation(transform.translation),
            0.08,
            Color::srgb(1.0, 0.0, 1.0), // Magenta
        );

        let size = 0.1;
        gizmos.line(
            transform.translation + Vec3::X * size,
            transform.translation - Vec3::X * size,
            Color::srgb(1.0, 0.0, 1.0),
        );
        gizmos.line(
            transform.translation + Vec3::Y * size,
            transform.translation - Vec3::Y * size,
            Color::srgb(1.0, 0.0, 1.0),
        );
    }
}

// ============================================================================
// LOCOMOTION FOOT IK SYSTEM
// ============================================================================

/// Updates foot IK targets based on ground raycasting during locomotion
/// This runs during normal movement (not parkour) to adapt feet to terrain
pub fn update_locomotion_foot_ik(
    spatial_query: SpatialQuery,
    config: Res<LocomotionIkConfig>,
    parkour_query: Query<&ParkourController, With<Player>>,
    bone_query: Query<(Entity, &GlobalTransform, &Name)>,
    mut left_foot_target_query: Query<&mut Transform, (With<LeftFootIkTarget>, Without<RightFootIkTarget>)>,
    mut right_foot_target_query: Query<&mut Transform, With<RightFootIkTarget>>,
    mut ik_constraint_query: Query<(&Name, &mut IkConstraint)>,
) {
    if !config.enabled {
        return;
    }

    // Only apply foot IK during normal locomotion, not during parkour
    let Ok(parkour) = parkour_query.single() else {
        return;
    };

    // Enable for all states except parkour actions
    let is_normal_locomotion = !matches!(
        parkour.state,
        ParkourState::Vaulting | ParkourState::Climbing |
        ParkourState::Sliding | ParkourState::Hanging
    );

    // Find the foot bone entities
    let mut left_foot_data = None;
    let mut right_foot_data = None;

    for (entity, transform, name) in bone_query.iter() {
        match name.as_str() {
            "mixamorig12:LeftFoot" => left_foot_data = Some((entity, transform)),
            "mixamorig12:RightFoot" => right_foot_data = Some((entity, transform)),
            _ => {}
        }
    }

    // Enable/disable foot IK constraints based on state
    for (name, mut constraint) in ik_constraint_query.iter_mut() {
        match name.as_str() {
            "mixamorig12:LeftFoot" | "mixamorig12:RightFoot" => {
                constraint.enabled = is_normal_locomotion;
            }
            _ => {}
        }
    }

    if !is_normal_locomotion {
        return;
    }

    // Raycast from each foot to find ground
    if let Some((_entity, foot_transform)) = left_foot_data {
        let foot_pos = foot_transform.translation();

        // Raycast downward from foot position
        if let Some(hit) = spatial_query.cast_ray(
            foot_pos,
            Dir3::NEG_Y,
            config.max_ground_distance,
            true,
            &SpatialQueryFilter::default(),
        ) {
            // Adjust foot target to ground position
            if let Ok(mut target_transform) = left_foot_target_query.single_mut() {
                let ground_pos = foot_pos + Vec3::NEG_Y * hit.distance;
                let adjusted_pos = ground_pos + Vec3::Y * config.foot_height_offset;

                // Blend between current and target position
                target_transform.translation = target_transform.translation.lerp(
                    adjusted_pos,
                    config.adjustment_strength
                );
            }
        }
    }

    if let Some((_entity, foot_transform)) = right_foot_data {
        let foot_pos = foot_transform.translation();

        if let Some(hit) = spatial_query.cast_ray(
            foot_pos,
            Dir3::NEG_Y,
            config.max_ground_distance,
            true,
            &SpatialQueryFilter::default(),
        ) {
            if let Ok(mut target_transform) = right_foot_target_query.single_mut() {
                let ground_pos = foot_pos + Vec3::NEG_Y * hit.distance;
                let adjusted_pos = ground_pos + Vec3::Y * config.foot_height_offset;

                target_transform.translation = target_transform.translation.lerp(
                    adjusted_pos,
                    config.adjustment_strength
                );
            }
        }
    }
}

/// Debug visualization for locomotion foot IK
pub fn visualize_locomotion_foot_ik(
    config: Res<LocomotionIkConfig>,
    left_foot_query: Query<&Transform, With<LeftFootIkTarget>>,
    right_foot_query: Query<&Transform, (With<RightFootIkTarget>, Without<LeftFootIkTarget>)>,
    mut gizmos: Gizmos,
) {
    if !config.debug_visualization || !config.enabled {
        return;
    }

    // Visualize left foot target
    if let Ok(transform) = left_foot_query.single() {
        gizmos.sphere(
            Isometry3d::from_translation(transform.translation),
            0.06,
            Color::srgb(0.0, 1.0, 0.0), // Green
        );
    }

    // Visualize right foot target
    if let Ok(transform) = right_foot_query.single() {
        gizmos.sphere(
            Isometry3d::from_translation(transform.translation),
            0.06,
            Color::srgb(1.0, 1.0, 0.0), // Yellow
        );
    }
}

// ============================================================================
// DEBUG LOGGING SYSTEM
// ============================================================================

/// Writes IK debug information to RON file for troubleshooting
pub fn write_ik_debug_info(
    parkour_query: Query<&ParkourController, With<Player>>,
    bone_query: Query<(Entity, &GlobalTransform, &Name)>,
    ik_constraint_query: Query<(Entity, &Name, &IkConstraint)>,
    left_foot_target_query: Query<&Transform, (With<LeftFootIkTarget>, Without<RightFootIkTarget>)>,
    right_foot_target_query: Query<&Transform, With<RightFootIkTarget>>,
    config: Res<LocomotionIkConfig>,
    time: Res<Time>,
) {
    // Only write once per second to avoid spam
    static mut LAST_WRITE: f32 = 0.0;
    let current_time = time.elapsed_secs();

    unsafe {
        if current_time - LAST_WRITE < 1.0 {
            return;
        }
        LAST_WRITE = current_time;
    }

    let mut debug_info = String::new();
    debug_info.push_str("(\n");
    debug_info.push_str(&format!("  timestamp: {},\n", current_time));

    // Config status
    debug_info.push_str("  config: (\n");
    debug_info.push_str(&format!("    enabled: {},\n", config.enabled));
    debug_info.push_str(&format!("    max_ground_distance: {},\n", config.max_ground_distance));
    debug_info.push_str(&format!("    foot_height_offset: {},\n", config.foot_height_offset));
    debug_info.push_str(&format!("    adjustment_strength: {},\n", config.adjustment_strength));
    debug_info.push_str("  ),\n");

    // Parkour state
    if let Ok(parkour) = parkour_query.single() {
        debug_info.push_str(&format!("  parkour_state: \"{:?}\",\n", parkour.state));
        let is_normal = !matches!(
            parkour.state,
            ParkourState::Vaulting | ParkourState::Climbing |
            ParkourState::Sliding | ParkourState::Hanging
        );
        debug_info.push_str(&format!("  ik_should_be_active: {},\n", is_normal));
    } else {
        debug_info.push_str("  parkour_state: \"Not Found\",\n");
        debug_info.push_str("  ik_should_be_active: false,\n");
    }

    // Bone entities
    debug_info.push_str("  bones_found: (\n");
    let mut found_left_foot = false;
    let mut found_right_foot = false;
    let mut left_foot_pos = Vec3::ZERO;
    let mut right_foot_pos = Vec3::ZERO;

    for (_entity, transform, name) in bone_query.iter() {
        match name.as_str() {
            "mixamorig12:LeftFoot" => {
                found_left_foot = true;
                left_foot_pos = transform.translation();
                debug_info.push_str(&format!("    left_foot: \"Found\",\n"));
                debug_info.push_str(&format!("    left_foot_pos: ({}, {}, {}),\n",
                    left_foot_pos.x, left_foot_pos.y, left_foot_pos.z));
            }
            "mixamorig12:RightFoot" => {
                found_right_foot = true;
                right_foot_pos = transform.translation();
                debug_info.push_str(&format!("    right_foot: \"Found\",\n"));
                debug_info.push_str(&format!("    right_foot_pos: ({}, {}, {}),\n",
                    right_foot_pos.x, right_foot_pos.y, right_foot_pos.z));
            }
            _ => {}
        }
    }

    if !found_left_foot {
        debug_info.push_str("    left_foot: \"Not Found\",\n");
    }
    if !found_right_foot {
        debug_info.push_str("    right_foot: \"Not Found\",\n");
    }
    debug_info.push_str("  ),\n");

    // IK Constraints
    debug_info.push_str("  ik_constraints: [\n");
    for (_entity, name, constraint) in ik_constraint_query.iter() {
        if name.as_str().contains("Foot") {
            debug_info.push_str("    (\n");
            debug_info.push_str(&format!("      bone: \"{}\",\n", name.as_str()));
            debug_info.push_str(&format!("      enabled: {},\n", constraint.enabled));
            debug_info.push_str(&format!("      chain_length: {},\n", constraint.chain_length));
            debug_info.push_str(&format!("      iterations: {},\n", constraint.iterations));
            debug_info.push_str("    ),\n");
        }
    }
    debug_info.push_str("  ],\n");

    // IK Targets
    debug_info.push_str("  ik_targets: (\n");
    if let Ok(transform) = left_foot_target_query.single() {
        debug_info.push_str(&format!("    left_foot_target: ({}, {}, {}),\n",
            transform.translation.x, transform.translation.y, transform.translation.z));
    } else {
        debug_info.push_str("    left_foot_target: \"Not Found\",\n");
    }

    if let Ok(transform) = right_foot_target_query.single() {
        debug_info.push_str(&format!("    right_foot_target: ({}, {}, {}),\n",
            transform.translation.x, transform.translation.y, transform.translation.z));
    } else {
        debug_info.push_str("    right_foot_target: \"Not Found\",\n");
    }
    debug_info.push_str("  ),\n");

    debug_info.push_str(")\n");

    // Write to file
    let _ = std::fs::write("assets/debug/ik_debug.ron", debug_info);
}

// ============================================================================
// PLUGIN
// ============================================================================

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<IkConfig>();
    app.init_resource::<LocomotionIkConfig>();
    app.add_plugins(InverseKinematicsPlugin);

    // IK setup happens once after player model loads
    app.add_systems(
        Update,
        setup_ik_chains.run_if(in_state(Screen::Gameplay)),
    );

    // Parkour IK update systems run every frame during gameplay
    app.add_systems(
        Update,
        (
            update_ik_targets_from_obstacles,
            toggle_ik_constraints,
            visualize_ik_targets,
        )
            .chain()
            .run_if(in_state(Screen::Gameplay)),
    );

    // Locomotion foot IK systems (for basic movement)
    app.add_systems(
        Update,
        (
            update_locomotion_foot_ik,
            visualize_locomotion_foot_ik,
            write_ik_debug_info,  // Debug logging to RON file
        )
            .run_if(in_state(Screen::Gameplay)),
    );
}
