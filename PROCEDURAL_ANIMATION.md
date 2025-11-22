# Procedural Animation System

## Overview

This system implements Overgrowth-style procedural animation using only **13 keyframe poses** that are blended dynamically based on character physics state (velocity, acceleration, terrain angle).

## Architecture

### Key Concepts

**Traditional Animation**:
- Play full animation clips (100+ frames)
- Time-based playback
- Fixed transitions

**Procedural Animation (Overgrowth approach)**:
- Use only 13 static poses
- Blend based on velocity/physics
- Infinite variation from minimal data

### The 13 Keyframe Poses

1. **Idle** - Neutral standing pose
2. **Walk Left Foot Forward** - Left foot extended, right foot back
3. **Walk Right Foot Forward** - Right foot extended, left foot back
4. **Run Left Foot Forward** - Running pose, left foot forward
5. **Run Right Foot Forward** - Running pose, right foot forward
6. **Jump Takeoff** - Crouch before jump
7. **Jump Airborne** - Mid-air pose
8. **Jump Landing** - Landing crouch
9. **Roll Left** - Left roll pose
10. **Roll Right** - Right roll pose
11. **Attack Punch** - Punch pose
12. **Attack Kick** - Kick pose
13. **Crouch** - Crouching pose

## RON Format

Poses are stored as `.pose.ron` files in `assets/poses/` directory.

### Example: `idle.pose.ron`

```ron
(
    name: "Idle",
    bone_transforms: {
        "mixamorig12:Hips": (
            translation: (0.0, 0.95, 0.0),
            rotation: (0.0, 0.0, 0.0, 1.0),  // Quat(x, y, z, w)
            scale: (1.0, 1.0, 1.0),
        ),
        "mixamorig12:Spine": (
            translation: (0.0, 0.1, 0.0),
            rotation: (0.0, 0.0, 0.0, 1.0),
            scale: (1.0, 1.0, 1.0),
        ),
        "mixamorig12:LeftFoot": (
            translation: (-0.15, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0, 1.0),
            scale: (1.0, 1.0, 1.0),
        ),
        "mixamorig12:RightFoot": (
            translation: (0.15, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0, 1.0),
            scale: (1.0, 1.0, 1.0),
        ),
        // ... all bones in the rig
    },
    metadata: (
        source_animation: Some("idle"),
        source_frame: Some(0.0),
        source_time: Some(0.5),
        notes: Some("Neutral standing pose"),
    ),
)
```

## Usage

### Step 1: Extract Poses from Existing Animations

Run the game with the extraction feature enabled:

```bash
cargo run --features extract_poses
```

Or combine with other features:

```bash
cargo run --features "dev_native,extract_poses"
```

This will:
1. Load the character GLB file (`assets/models/characters/brian_parkour.glb`)
2. Sample specific frames from animations (configured in `extraction.rs`)
3. Save 13 pose files to `assets/poses/`

**Note**: The extraction code is only compiled when the `extract_poses` feature is enabled, so there's zero overhead in normal builds.

### Step 2: Configure Extraction Times

Edit `src/procedural_animation/extraction.rs` to adjust which frames are extracted:

```rust
ExtractionEntry {
    animation_name: "walk".to_string(),
    time_seconds: 0.25,  // Extract at 0.25 seconds into walk animation
    pose_id: PoseId::WalkLeftFootForward,
    notes: Some("Left foot forward, right foot back".to_string()),
},
```

### Step 3: Enable Procedural Animation

In `src/main.rs`, uncomment:

```rust
app.add_plugins(procedural_animation::ProceduralAnimationPlugin);
```

### Step 4: Add Component to Character

```rust
commands.spawn((
    // ... character components
    ProceduralAnimationController {
        enabled: true,
        blend_state: PoseBlendState::default(),
    },
));
```

## Blending Logic

### Velocity-Based Blending

The system calculates which poses to blend based on character speed:

- **< 0.5 m/s**: 100% Idle
- **0.5 - 3.0 m/s**: Blend between WalkLeft ↔ WalkRight (walk cycle)
- **> 3.0 m/s**: Blend between RunLeft ↔ RunRight (run cycle)
- **Airborne**: JumpAirborne or JumpLanding based on vertical velocity

### Foot Phase Calculation

The walk/run cycle is controlled by a `foot_phase` value (0.0 - 1.0):

- **0.0 - 0.5**: Transition from Left foot forward → Right foot forward
- **0.5 - 1.0**: Transition from Right foot forward → Left foot forward

Phase advances based on velocity:
- **Walk**: 1.0 - 2.25 Hz (cycles per second)
- **Run**: 2.0 - 3.5 Hz

### Stride Length

Stride length is calculated based on:
1. **Velocity**: Faster = longer strides
2. **Terrain angle**: Uphill = shorter (70%), downhill = slightly longer (110%)
3. **Base values**:
   - Walk: 0.6m
   - Run: 1.2m

## Implementation Status

### ✅ Completed
- [x] Plugin architecture
- [x] Pose data structures (RON serialization)
- [x] Pose asset loader
- [x] Velocity-based blend weight calculation
- [x] Foot phase calculation
- [x] Stride length calculation
- [x] Extraction tool framework

### 🚧 TODO
- [ ] Complete curve sampling in extraction tool (currently placeholder)
- [ ] Apply blended poses to character bones
- [ ] Test with real extracted poses
- [ ] Add terrain normal detection for slope adjustment
- [ ] Integrate with IK system for precise foot placement
- [ ] Add acceleration-based blending
- [ ] Implement roll and attack poses

## Integration with IK

The procedural animation system works with the IK system:

1. **Procedural animation** calculates rough limb positions based on velocity
2. **IK system** refines foot/hand placement for precise contact
3. **Animation masks** allow IK to override specific bones

Example: Character running uphill:
- Procedural animation: Blends RunLeft/RunRight based on speed + foot phase
- Stride calculator: Shortens stride due to uphill slope
- IK system: Places feet on actual terrain surface

## Debugging

Enable debug visualizations:

```rust
app.add_systems(Update, stride::debug_visualize_stride);
```

This draws:
- Yellow line: Current stride length
- Cyan sphere: Foot phase indicator (rotates around character)

## Performance

**Memory**:
- 13 poses × ~100 bones × 40 bytes = ~52 KB total
- vs traditional: 5 animations × 100 frames × 100 bones × 40 bytes = ~2 MB

**CPU**:
- Blending 2-3 poses per frame
- No animation playback overhead
- IK solving is the main cost

## References

- [Wolfire GDC 2014 - Procedural Animation](https://www.wolfire.com/blog/2014/05/GDC-2014-Procedural-Animation-Video/)
- [Overgrowth Physics Engine](https://www.wolfire.com/blog/2009/05/creating-the-overgrowth-physics-engine/)
- [Overgrowth Open Source](https://www.wolfire.com/blog/2022/04/Overgrowth-Open-Source-Announcement/)
