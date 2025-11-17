# Foot Placement Troubleshooting Guide

## The Issue

You have foot placement enabled but don't see any changes. Let's diagnose what's wrong!

## Diagnostic Tool

I've added an automatic diagnostic system that runs every 3 seconds. When you run your game, look for these log messages:

```
=== Foot Placement Diagnostics ===
Player entity: Entity { index: 123, generation: 1 }
Raycast distance: 2.0
Foot offset: 0.05
Update interval: 0.1
Min slope angle: 5.0
Bone map size: 2
✓ BoneMap populated with bones:
  - LeftFoot -> Entity { index: 456, generation: 1 }
  - RightFoot -> Entity { index: 789, generation: 1 }
```

## Common Issues & Solutions

### Issue 1: BoneMap is Empty ⚠️

**Symptoms:**
```
⚠️  BoneMap is EMPTY - bones not discovered!
```

**Cause:** The `build_bone_map` system couldn't find your foot bones.

**Solutions:**

1. **Check bone naming**: Your Mixamo rig should have bones named:
   - `mixamorig12:LeftFoot`
   - `mixamorig12:RightFoot`

2. **Verify bones are in the scene**: Run this query to see all named entities:
   ```rust
   fn list_all_bones(query: Query<(Entity, &Name)>) {
       for (entity, name) in query.iter() {
           if name.as_str().contains("Foot") {
               info!("Found foot bone: {} -> {:?}", name, entity);
           }
       }
   }
   ```

3. **Timing issue**: BoneMap might build before the character scene loads. The system should retry, but you can manually trigger:
   ```rust
   // Remove and re-add BoneMap to force rebuild
   commands.entity(player).remove::<BoneMap>();
   ```

### Issue 2: Only Works on Slopes ⚠️

**Symptoms:**
```
⚠️  min_slope_angle is 5.0 - will only activate on slopes
```

**Cause:** Default configuration only activates on slopes > 5 degrees.

**Solution:** Use the testing preset on flat ground:

```rust
// In player spawn:
FootPlacementEnabled::for_testing()  // Instead of ::default()
```

Or customize:
```rust
FootPlacementEnabled {
    min_slope_angle: 0.0,  // Always active
    ..Default::default()
}
```

### Issue 3: Raycasts Not Hitting Ground ⚠️

**Symptoms:**
```
Left foot raycast missed ground
Right foot raycast missed ground
```

**Causes & Solutions:**

**A) Ground has no collider**
```rust
// Make sure your ground has a collider:
commands.spawn((
    Mesh3d(...),
    Transform::default(),
    Collider::cuboid(10.0, 0.1, 10.0),  // ← ADD THIS
));
```

**B) Raycast distance too short**
```rust
FootPlacementEnabled {
    raycast_distance: 5.0,  // Increase from default 2.0
    ..Default::default()
}
```

**C) Feet are above/below ground**
- Check that character is actually standing on ground
- Try adjusting player spawn height

### Issue 4: Capsule Collider Interfering

**Status:** ✅ **FIXED** - We now exclude the player entity from raycasts.

The capsule collider on your player is fine. The raycasts now use:
```rust
let filter = SpatialQueryFilter::from_excluded_entities([player_entity]);
```

This ensures we only hit the ground, not the player's own collider.

## Debug Logging Levels

To see detailed foot placement activity, check these log messages:

**TRACE level** (very verbose):
```
Foot placement update - bone map has 2 bones
Found left foot entity: Entity { ... }
Left foot position: Vec3(0.5, 1.0, 0.0)
Raycasting from Vec3(0.5, 1.0, 0.0) down for 2.0 units
Raycast hit at distance 1.5, final position: Vec3(0.5, -0.45, 0.0)
```

**INFO level** (important events):
```
Left foot raycast hit ground at: Vec3(0.5, -0.45, 0.0)
Right foot raycast hit ground at: Vec3(-0.5, -0.45, 0.0)
```

**WARN level** (problems):
```
⚠️  BoneMap is empty for player Entity { ... }
⚠️  Left foot not found in bone map
⚠️  Left foot entity has no GlobalTransform
```

## Testing Checklist

Run through this checklist:

- [ ] Player has `FootPlacementEnabled` component
- [ ] Player has `TargetMatchEnabled` component
- [ ] Player has `BoneMap` component (starts empty, auto-populates)
- [ ] Diagnostic logs show "BoneMap populated with bones"
- [ ] Ground has collider component
- [ ] Player is standing on ground (not floating/falling)
- [ ] Using `FootPlacementEnabled::for_testing()` for flat ground testing
- [ ] Console shows "raycast hit ground" messages

## Quick Fix Configurations

### For Flat Ground Testing
```rust
FootPlacementEnabled {
    raycast_distance: 5.0,
    foot_offset: 0.05,
    update_interval: 0.1,
    min_slope_angle: 0.0,  // ← KEY: Always active
    timer: Timer::from_seconds(0.1, TimerMode::Repeating),
}

// Or use the preset:
FootPlacementEnabled::for_testing()
```

### For Slopes Only (Default)
```rust
FootPlacementEnabled::default()  // min_slope_angle: 5.0
```

### For Aggressive Terrain
```rust
FootPlacementEnabled {
    raycast_distance: 10.0,  // Very long raycast
    foot_offset: 0.0,        // Directly on surface
    min_slope_angle: 0.0,    // Always active
    update_interval: 0.05,   // Very frequent
    timer: Timer::from_seconds(0.05, TimerMode::Repeating),
}
```

## What to Look For When It Works

When foot placement is working correctly, you should see:

1. **Console logs every 0.1 seconds:**
   ```
   Left foot raycast hit ground at: Vec3(...)
   Right foot raycast hit ground at: Vec3(...)
   ```

2. **Feet visibly adjusting** to terrain height

3. **No warnings** about empty BoneMap or missing bones

4. **Diagnostic shows:**
   ```
   ✓ BoneMap populated with bones:
     - LeftFoot -> Entity { ... }
     - RightFoot -> Entity { ... }
   ```

## Next Steps

If you're still having issues:

1. **Share the diagnostic output** - Copy the console logs
2. **Check bone names** - List all entities with "Foot" in the name
3. **Verify ground collider** - Make sure ground has physics collider
4. **Try the testing preset** - Use `FootPlacementEnabled::for_testing()`

## Advanced Debugging

Add this temporary system to see everything:

```rust
fn debug_everything(
    players: Query<(Entity, &GlobalTransform), With<Player>>,
    bones: Query<(Entity, &Name, &GlobalTransform)>,
    spatial_query: SpatialQuery,
) {
    for (player_entity, player_transform) in players.iter() {
        info!("Player at: {:?}", player_transform.translation());

        // List all bones
        for (bone_entity, bone_name, bone_transform) in bones.iter() {
            if bone_name.as_str().contains("Foot") {
                info!("  Bone '{}' at {:?}", bone_name, bone_transform.translation());

                // Test raycast
                let filter = SpatialQueryFilter::from_excluded_entities([player_entity]);
                if let Some(hit) = spatial_query.cast_ray(
                    bone_transform.translation(),
                    Dir3::NEG_Y,
                    5.0,
                    true,
                    &filter,
                ) {
                    info!("    → Raycast hit at distance: {}", hit.distance);
                } else {
                    warn!("    → Raycast MISSED");
                }
            }
        }
    }
}
```

This will show exactly what's happening with each bone and raycast.
