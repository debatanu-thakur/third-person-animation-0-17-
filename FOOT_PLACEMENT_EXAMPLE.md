# Foot Placement on Slopes - Implementation Example

## Overview

This guide shows how to use the Target Matching plugin for dynamic foot placement on uneven terrain and slopes.

## Approach

1. **Detect ground beneath each foot** using raycasts
2. **Calculate target positions** based on slope angle and foot offset
3. **Request target matching** to adjust foot position
4. **Continuously update** as character moves

## Implementation

### Step 1: Create Foot Placement System

```rust
use bevy::prelude::*;
use avian3d::prelude::*;
use crate::game::{
    player::Player,
    target_matching::{TargetMatchRequest, TargetBone, TargetMatchEnabled, BoneMap},
};

/// Component to enable dynamic foot placement
#[derive(Component)]
pub struct FootPlacementEnabled {
    /// How far to raycast down from foot position
    pub raycast_distance: f32,
    /// Offset from ground surface
    pub foot_offset: f32,
    /// How often to update foot placement (in seconds)
    pub update_interval: f32,
    /// Timer for updates
    pub timer: Timer,
}

impl Default for FootPlacementEnabled {
    fn default() -> Self {
        Self {
            raycast_distance: 2.0,
            foot_offset: 0.05, // 5cm above ground
            update_interval: 0.1, // Update 10 times per second
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
        }
    }
}

/// System to detect ground and request foot target matching
pub fn update_foot_placement(
    mut commands: Commands,
    time: Res<Time>,
    spatial_query: SpatialQuery,
    mut players: Query<(
        Entity,
        &GlobalTransform,
        &BoneMap,
        &mut FootPlacementEnabled,
    ), With<Player>>,
    foot_transforms: Query<&GlobalTransform>,
) {
    for (player_entity, player_transform, bone_map, mut foot_placement) in players.iter_mut() {
        // Update timer
        foot_placement.timer.tick(time.delta());

        if !foot_placement.timer.just_finished() {
            continue;
        }

        // Get foot bone entities
        let Some(left_foot_entity) = bone_map.get(TargetBone::LeftFoot) else {
            continue;
        };
        let Some(right_foot_entity) = bone_map.get(TargetBone::RightFoot) else {
            continue;
        };

        // Get current foot positions
        let Ok(left_foot_transform) = foot_transforms.get(left_foot_entity) else {
            continue;
        };
        let Ok(right_foot_transform) = foot_transforms.get(right_foot_entity) else {
            continue;
        };

        // Raycast down from each foot to find ground
        if let Some(left_target) = raycast_for_ground(
            &spatial_query,
            left_foot_transform.translation(),
            foot_placement.raycast_distance,
            foot_placement.foot_offset,
        ) {
            // Request left foot target matching
            commands.entity(player_entity).insert(
                TargetMatchRequest::new(
                    TargetBone::LeftFoot,
                    left_target,
                    foot_placement.update_interval, // Match over one update cycle
                )
                .with_window(0.0, 1.0), // Immediate match
            );
        }

        if let Some(right_target) = raycast_for_ground(
            &spatial_query,
            right_foot_transform.translation(),
            foot_placement.raycast_distance,
            foot_placement.foot_offset,
        ) {
            // Request right foot target matching
            commands.entity(player_entity).insert(
                TargetMatchRequest::new(
                    TargetBone::RightFoot,
                    right_target,
                    foot_placement.update_interval,
                )
                .with_window(0.0, 1.0),
            );
        }
    }
}

/// Helper function to raycast and find ground position
fn raycast_for_ground(
    spatial_query: &SpatialQuery,
    foot_position: Vec3,
    max_distance: f32,
    offset: f32,
) -> Option<Vec3> {
    // Raycast downward from foot position
    let ray_origin = foot_position;
    let ray_direction = Vec3::NEG_Y;

    if let Some(hit) = spatial_query.cast_ray(
        ray_origin,
        ray_direction,
        max_distance,
        true, // Should hit triggers
        SpatialQueryFilter::default(),
    ) {
        // Return hit position with offset
        Some(hit.point + Vec3::Y * offset)
    } else {
        None
    }
}
```

### Step 2: Add to Your Player

```rust
// In your player spawn system (src/game/player/mod.rs)

use crate::game::target_matching::{TargetMatchEnabled, BoneMap};

fn spawn_player(
    In(spawn_config): In<SpawnPlayer>,
    mut commands: Commands,
    player_assets: Res<PlayerAssets>,
) {
    commands
        .spawn((
            Name::new("Player"),
            Player,
            MovementController::default(),
            ThirdPersonCameraTarget,

            // Enable target matching
            TargetMatchEnabled,
            BoneMap::default(), // Will be populated automatically
            FootPlacementEnabled::default(), // Enable slope foot placement

            // ... rest of your components
            Transform::from_translation(spawn_config.position),
            Visibility::Visible,
            RigidBody::Dynamic,
            Collider::capsule(PLAYER_HEIGHT / 2., PLAYER_RADIUS),
            TnuaController::default(),
            LockedAxes::ROTATION_LOCKED.unlock_rotation_y(),
            TnuaAvian3dSensorShape(Collider::cylinder(PLAYER_HEIGHT / 2., 0.0)),
            TnuaAnimatingState::<AnimationState>::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                SceneRoot(player_assets.character_scene.clone()),
                Transform::from_translation(Vec3::new(0., -0.8, 0.))
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::PI))
            ));
        });
}
```

### Step 3: Add System to Your Plugin

```rust
// In your animations plugin or create a new foot_placement module

use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        update_foot_placement
            .run_if(in_state(Screen::Gameplay)),
    );
}
```

## Advanced: Slope-Aware Foot Placement

For better results on steep slopes, also adjust foot rotation:

```rust
/// Enhanced version with foot rotation
fn raycast_for_ground_with_normal(
    spatial_query: &SpatialQuery,
    foot_position: Vec3,
    max_distance: f32,
    offset: f32,
) -> Option<(Vec3, Vec3)> {
    let ray_origin = foot_position;
    let ray_direction = Vec3::NEG_Y;

    if let Some(hit) = spatial_query.cast_ray(
        ray_origin,
        ray_direction,
        max_distance,
        true,
        SpatialQueryFilter::default(),
    ) {
        let target_position = hit.point + Vec3::Y * offset;
        let surface_normal = hit.normal;

        Some((target_position, surface_normal))
    } else {
        None
    }
}

/// Create a custom curve that includes rotation
fn create_foot_placement_curve_with_rotation(
    target_id: AnimationTargetId,
    target_position: Vec3,
    target_rotation: Quat,
    duration: f32,
) -> AnimationClip {
    use bevy::animation::{animated_field, AnimatableCurve};

    let mut clip = AnimationClip::default();

    // Position curve
    clip.add_curve_to_target(
        target_id,
        AnimatableCurve::new(
            animated_field!(Transform::translation),
            UnevenSampleAutoCurve::new(
                [0.0, duration].into_iter().zip([
                    Vec3::ZERO, // Relative to current position
                    target_position,
                ])
            ).expect("Failed to create position curve"),
        ),
    );

    // Rotation curve (to align foot with slope)
    clip.add_curve_to_target(
        target_id,
        AnimatableCurve::new(
            animated_field!(Transform::rotation),
            UnevenSampleAutoCurve::new(
                [0.0, duration].into_iter().zip([
                    Quat::IDENTITY,
                    target_rotation,
                ])
            ).expect("Failed to create rotation curve"),
        ),
    );

    clip.set_duration(duration);
    clip
}

/// Calculate foot rotation from surface normal
fn calculate_foot_rotation_from_normal(surface_normal: Vec3) -> Quat {
    // Align foot up vector with surface normal
    Quat::from_rotation_arc(Vec3::Y, surface_normal)
}
```

## Configuration Options

### Tuning Parameters

```rust
// Adjust these values based on your needs:

FootPlacementEnabled {
    raycast_distance: 2.0,   // How far down to look for ground
    foot_offset: 0.05,       // Height above ground (prevents clipping)
    update_interval: 0.1,    // How often to update (affects smoothness)
    timer: Timer::from_seconds(0.1, TimerMode::Repeating),
}

// For smoother results on gentle slopes:
FootPlacementEnabled {
    update_interval: 0.05,   // Update more frequently
    foot_offset: 0.02,       // Less offset for closer contact
    ..default()
}

// For performance on flat terrain:
FootPlacementEnabled {
    update_interval: 0.2,    // Update less frequently
    raycast_distance: 1.0,   // Shorter raycast
    ..default()
}
```

### Conditional Activation

Only enable on slopes:

```rust
fn update_foot_placement_conditional(
    // ... same parameters ...
) {
    for (player_entity, player_transform, bone_map, mut foot_placement) in players.iter_mut() {
        // Check if player is on a slope
        let ground_normal = detect_ground_normal(&spatial_query, player_transform.translation());

        let slope_angle = ground_normal.angle_between(Vec3::Y).to_degrees();

        // Only activate foot placement on slopes > 5 degrees
        if slope_angle < 5.0 {
            continue;
        }

        // ... rest of foot placement logic
    }
}
```

## Performance Considerations

### Option 1: Raycast Pooling
```rust
// Reuse raycasts from physics system
#[derive(Resource)]
struct GroundDetectionCache {
    left_foot_hit: Option<Vec3>,
    right_foot_hit: Option<Vec3>,
    frame: u64,
}
```

### Option 2: Distance-Based Updates
```rust
// Only update when player moves
#[derive(Component)]
struct LastFootPlacementPosition(Vec3);

// In system:
let moved_distance = (player_transform.translation() - last_pos.0).length();
if moved_distance < 0.5 {
    continue; // Don't update if player hasn't moved much
}
```

### Option 3: Use IK Instead of Curves

For better performance and more natural results:

```rust
use bevy_mod_inverse_kinematics::{IkConstraint, IkPoleTarget};

fn setup_foot_ik(
    mut commands: Commands,
    player: Entity,
    bone_map: &BoneMap,
    left_target_pos: Vec3,
    right_target_pos: Vec3,
) {
    // Create target entities
    let left_target = commands.spawn((
        Name::new("LeftFootTarget"),
        Transform::from_translation(left_target_pos),
        Visibility::default(),
    )).id();

    let right_target = commands.spawn((
        Name::new("RightFootTarget"),
        Transform::from_translation(right_target_pos),
        Visibility::default(),
    )).id();

    // Apply IK to feet
    if let Some(left_foot) = bone_map.get(TargetBone::LeftFoot) {
        commands.entity(left_foot).insert(IkConstraint {
            chain_length: 3, // Hip -> Knee -> Ankle
            iterations: 20,
            target: left_target,
            pole_target: None,
            pole_angle: 0.0,
            enabled: true,
        });
    }

    // Same for right foot...
}
```

## Complete Example Module

```rust
// src/game/foot_placement/mod.rs

use bevy::prelude::*;
use avian3d::prelude::*;
use crate::{
    game::{
        player::Player,
        target_matching::{TargetMatchRequest, TargetBone, BoneMap},
    },
    screens::Screen,
};

pub struct FootPlacementPlugin;

impl Plugin for FootPlacementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_foot_placement.run_if(in_state(Screen::Gameplay)),
        );
    }
}

#[derive(Component)]
pub struct FootPlacementEnabled {
    pub raycast_distance: f32,
    pub foot_offset: f32,
    pub update_interval: f32,
    pub timer: Timer,
}

impl Default for FootPlacementEnabled {
    fn default() -> Self {
        Self {
            raycast_distance: 2.0,
            foot_offset: 0.05,
            update_interval: 0.1,
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
        }
    }
}

fn update_foot_placement(
    mut commands: Commands,
    time: Res<Time>,
    spatial_query: SpatialQuery,
    mut players: Query<(
        Entity,
        &GlobalTransform,
        &BoneMap,
        &mut FootPlacementEnabled,
    ), With<Player>>,
    foot_transforms: Query<&GlobalTransform>,
) {
    // Implementation from above...
}

fn raycast_for_ground(
    spatial_query: &SpatialQuery,
    foot_position: Vec3,
    max_distance: f32,
    offset: f32,
) -> Option<Vec3> {
    // Implementation from above...
}
```

## Testing

1. **Create a sloped test level**:
   - Add some angled planes in your scene
   - Vary the slope angles (10°, 30°, 45°)

2. **Enable debug visualization**:
```rust
app.insert_resource(TargetMatchDebugSettings {
    show_targets: true,  // See where feet are targeting
    show_bones: true,    // See current foot positions
    ..default()
});
```

3. **Walk your character up/down slopes** and observe:
   - Red spheres = target positions (on ground)
   - Green spheres = current bone positions
   - Feet should smoothly adjust to terrain

## Common Issues & Solutions

### Issue: Feet jitter on flat ground
**Solution**: Increase `update_interval` or add a deadzone for small height differences

### Issue: Feet don't reach ground on steep slopes
**Solution**: Increase `raycast_distance` or adjust `foot_offset`

### Issue: Performance impact
**Solution**: Use IK instead of curves, or reduce `update_interval`

### Issue: Feet slide on slopes
**Solution**: Combine with character rotation to align body with slope

## Next Steps

- Combine with IK for more natural leg bending
- Add foot rotation to align with surface normal
- Implement during different movement states (walking, running, jumping)
- Add ground detection caching for performance
