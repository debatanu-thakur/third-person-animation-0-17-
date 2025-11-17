# Quick Start: Foot Placement on Slopes

## What You Need to Do

Just **3 simple steps** to enable dynamic foot placement on slopes!

### Step 1: Enable on Your Player

Add these components to your player spawn:

```rust
// In src/game/player/mod.rs, in the spawn_player function:

use crate::game::{
    target_matching::{TargetMatchEnabled, BoneMap},
    foot_placement::FootPlacementEnabled,
};

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

            // ✅ ADD THESE THREE LINES:
            TargetMatchEnabled,              // Enable target matching system
            BoneMap::default(),              // Will auto-populate with foot bones
            FootPlacementEnabled::default(), // Enable slope foot placement

            // ... rest of your existing components
            ThirdPersonCameraTarget,
            DespawnOnExit(Screen::Gameplay),
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

### Step 2: (Optional) Enable Debug Visualization

See it in action with debug gizmos:

```rust
// In your app setup (main.rs or wherever you configure plugins):

use crate::game::target_matching::TargetMatchDebugSettings;

app.insert_resource(TargetMatchDebugSettings {
    show_targets: true,   // Red spheres = where feet are trying to reach
    show_bones: true,     // Green spheres = current foot positions
    ..default()
});
```

### Step 3: Test It!

1. Run your game
2. Walk your character on a sloped surface
3. Watch the feet automatically adjust to the terrain!

## That's It! 🎉

The system is now:
- ✅ Automatically detecting ground beneath feet
- ✅ Requesting target matching for each foot
- ✅ Smoothly adjusting foot positions on slopes

## Configuration Options

Want to tune the behavior? Customize FootPlacementEnabled:

```rust
// Default (balanced):
FootPlacementEnabled::default()

// For gentle slopes (smoother, more updates):
FootPlacementEnabled::for_gentle_slopes()

// For steep terrain (performance-focused):
FootPlacementEnabled::for_steep_terrain()

// Or fully custom:
FootPlacementEnabled::new(
    2.0,   // raycast_distance: how far down to look
    0.05,  // foot_offset: height above ground
    0.1,   // update_interval: how often to update (seconds)
)
```

## How It Works

```
Every 0.1 seconds:
1. Raycast down from each foot
2. Find ground position
3. Request target matching to that position
4. Target matching plugin adjusts the foot

Result: Feet automatically plant on slopes!
```

## Troubleshooting

**Problem**: Feet jitter on flat ground
**Solution**: Increase `min_slope_angle` to only activate on slopes:
```rust
FootPlacementEnabled {
    min_slope_angle: 10.0, // Only activate on slopes > 10 degrees
    ..default()
}
```

**Problem**: Feet don't reach ground
**Solution**: Increase `raycast_distance`:
```rust
FootPlacementEnabled {
    raycast_distance: 3.0, // Look further down
    ..default()
}
```

**Problem**: Performance impact
**Solution**: Reduce update frequency:
```rust
FootPlacementEnabled {
    update_interval: 0.2, // Update 5 times per second instead of 10
    ..default()
}
```

## What's Happening Under the Hood

1. **FootPlacementPlugin** runs every frame
2. Checks if it's time to update (based on `update_interval`)
3. Raycasts down from foot bones using Avian3D physics
4. Creates **TargetMatchRequest** components for each foot
5. **TargetMatchingPlugin** handles the requests
6. Feet smoothly move to target positions

## Next Steps

- Combine with IK for even better leg bending (see FOOT_PLACEMENT_EXAMPLE.md)
- Add foot rotation to align with surface normal
- Customize for different movement states (walking, running, jumping)

## Full Documentation

- **FOOT_PLACEMENT_EXAMPLE.md** - Complete implementation guide with advanced features
- **TARGETMATCHING_USAGE.md** - Full target matching API reference
- **target_matching.md** - Technical architecture documentation
