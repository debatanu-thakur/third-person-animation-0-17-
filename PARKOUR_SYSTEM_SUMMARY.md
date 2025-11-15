# Parkour System Implementation Summary

## Overview

This document summarizes the parkour animation system implementation for Bevy 0.17, including obstacle detection, animation loading, and IK setup.

## System Architecture

### 1. Obstacle Detection (`src/game/obstacle_detection/`)

**Purpose**: Detect obstacles ahead of the player and determine appropriate parkour actions.

**Key Components**:
- `ObstacleDetectionConfig` - Configuration for detection ranges and heights
- `ObstacleDetectionResult` - Stores detected obstacle info (hit_point, height, distance)
- `ParkourController` - Manages parkour state machine (Idle, Vaulting, Climbing, etc.)

**How it Works**:
```rust
// Raycasts forward from player to detect obstacles
// Different heights for different parkour actions:
- Low obstacles (< 1.0m) → Vault
- Mid obstacles (1.0-1.8m) → Climb
- High obstacles → Hang/Pull-up
```

**Files**:
- `detection.rs:1-350` - Main detection logic
- `trigger.rs:1-150` - Parkour action triggering
- `visualization.rs:1-120` - Debug gizmos for detected obstacles

### 2. Animation System (`src/game/parkour_animations/`)

**Purpose**: Load and manage parkour animations using Bevy 0.17's AnimationGraph pattern.

**Key Resources**:
- `ParkourAnimationLibrary` - Stores AnimationGraph handles for each parkour action
- `SampledParkourPoses` - Stores sampled bone transforms at key times
- `AnimationBoneNames` - Tracks bone name matching between character and animations

**Animation Loading Pattern** (Bevy 0.17):
```rust
// 1. Load animation clip from GLB
let vault_clip = asset_server.load(
    GltfAssetLabel::Animation(0).from_asset("models/animations/vault_over_rining.glb")
);

// 2. Create AnimationGraph from clip
let (vault_graph, vault_node) = AnimationGraph::from_clip(vault_clip.clone());

// 3. Store graph in assets
let vault_graph = animation_graphs.add(vault_graph);

// 4. Later: Play animation
player.play(vault_node);
player.seek_to(0.5); // Seek to specific time for sampling
```

**Animation Files** (from Mixamo, animation-only, no character mesh):
- `vault_over_rining.glb` - Vaulting over low obstacles
- `Freehang Climb.glb` - Climbing up walls
- `Running Slide.glb` - Sliding under obstacles
- `Over Obstacle Jumping.glb` - Wall run actions
- `Falling To Roll.glb` - Recovery roll

**Files**:
- `mod.rs:1-375` - Animation loading, bone name collection, sampling setup

### 3. IK System (`src/game/parkour_ik/`)

**Purpose**: Adapt hand/foot placement to actual obstacle positions using Inverse Kinematics.

**Key Components**:
- `ParkourIkTargets` - Stores IK target positions for hands/feet
- `IkConfig` - Configuration (hand spread, height offset, debug viz)
- `LeftHandIkTarget`, `RightHandIkTarget`, etc. - Marker components

**IK Chain Setup**:
```rust
// Hand IK: Hand → Forearm → Arm (chain_length: 2)
IkConstraint {
    chain_length: 2,
    iterations: 20,
    target: left_hand_target_entity,
    pole_target: left_forearm_bone,
    enabled: true,
}
```

**Current Behavior**:
- Vaulting: Places hands on top of obstacle (spread apart)
- Climbing: Places hands on ledge point
- Hanging: Places hands slightly below ledge

**Next Step - Animation Integration**:
```rust
// Instead of fixed positions:
ik_targets.left_hand_target = obstacle_top + hand_spread;

// Use sampled animation pose:
let sampled_pose = sampled_poses.get_vault_hand_pos(progress, "LeftHand");
let adjusted_pos = blend(sampled_pose, obstacle_adjusted_pos, blend_factor);
ik_targets.left_hand_target = adjusted_pos;
```

**Files**:
- `mod.rs:1-411` - IK setup, target updates, visualization

## Data Flow

```
1. ObstacleDetection detects obstacle ahead
   ↓
2. ParkourController transitions to Vaulting state
   ↓
3. Animation system has vault animation ready (AnimationGraph)
   ↓
4. IK system calculates target positions:
   - Gets sampled pose from animation (future)
   - Adjusts to actual obstacle height
   - Sets IK targets for hands/feet
   ↓
5. bevy_mod_inverse_kinematics adjusts bone rotations
   ↓
6. Character's hands/feet reach exact obstacle positions
```

## Configuration Files

### IK Configuration
```rust
IkConfig {
    enabled: true,
    hand_spread: 0.3,      // 30cm between hands
    hand_height_offset: 0.05, // 5cm above obstacle
    debug_visualization: true,
}
```

### Obstacle Detection Configuration
```rust
ObstacleDetectionConfig {
    detection_distance: 2.0,
    detection_height: 1.5,
    vault_height_max: 1.0,
    climb_height_max: 1.8,
    ledge_check_distance: 0.5,
}
```

## Debug Controls

**Keyboard**:
- `P` - Print parkour animation library status
- (From obstacle_detection) Debug gizmos show detected obstacles

**Visual Indicators**:
- Green rays: Obstacle detection rays
- Red spheres: Obstacle hit points
- Cyan/Magenta spheres: IK targets (left/right hands)
- Yellow lines: IK target cross markers

## Implementation Status

### ✅ Completed
1. Obstacle detection with raycasting
2. Parkour state machine (Idle → Vaulting → Climbing, etc.)
3. Animation loading using AnimationGraph pattern
4. IK chain setup on player skeleton
5. IK target positioning based on obstacles
6. Debug visualization for all systems

### ⏳ In Progress
1. Runtime animation sampling (seek_to + read GlobalTransforms)
2. Integration of sampled poses with IK targets
3. Blending between animation poses and obstacle-adjusted positions

### 📋 To Do
1. Implement actual animation playback during parkour actions
2. Add foot IK for climbing/landing
3. Animation blending/transitions
4. Polish obstacle detection (better ledge detection)
5. Add more parkour actions (wall run, slide, roll)

## Key Learnings

### Bevy 0.17 Animation System
- Can't directly sample animation curves (VariableCurve is internal)
- Must use AnimationGraph + AnimationPlayer pattern
- Use `player.seek_to(time)` to sample at specific times
- Read bone `GlobalTransform` components instead of curve data

### Animation File Requirements
- Use Mixamo animations (same rig as character)
- Export "Without Skin" (animation-only)
- GLB format preferred over FBX
- First animation in file (index 0) is used

### IK System Pattern
- bevy_mod_inverse_kinematics uses FABRIK algorithm
- Chain length = number of bones to solve (hand→forearm→arm = 2)
- Pole targets help orient joints correctly
- Enable/disable constraints per parkour state

## File Structure

```
src/game/
├── obstacle_detection/
│   ├── mod.rs              - Plugin setup
│   ├── detection.rs        - Raycasting and state machine
│   ├── trigger.rs          - Action triggering logic
│   └── visualization.rs    - Debug gizmos
├── parkour_animations/
│   └── mod.rs              - Animation loading and sampling
├── parkour_ik/
│   └── mod.rs              - IK setup and target updates
├── parkour_poses/          - Manual pose extraction (deprecated)
└── mod.rs                  - Game plugin composition

assets/models/animations/   - Mixamo animation GLB files
```

## References

**Bevy Examples Used**:
- `examples/animation/animated_mesh.rs` - AnimationGraph pattern
- `examples/animation/eased_motion.rs` - Curve creation and easing
- `examples/3d/irradiance_volumes.rs` - GLTF animation loading

**Documentation**:
- `ANIMATION_SAMPLING_STRATEGY.md` - Runtime sampling approach
- `AGENTS.md` - Bevy 0.17 API patterns
- `assets/parkour_animations/README.md` - Animation file guide (outdated path)

## Next Steps

1. **Implement Runtime Sampling System**
   - Spawn temporary entity with player skeleton
   - Attach AnimationPlayer + AnimationGraphHandle
   - Play vault animation and seek to times: [0.0, 0.25, 0.5, 0.75, 1.0]
   - Read bone GlobalTransforms
   - Store in `SampledParkourPoses` resource

2. **Enhance IK Target Calculation**
   ```rust
   // Get animation sample at current parkour progress
   let progress = parkour.action_progress; // 0.0 to 1.0
   let anim_hand_pos = sampled_poses.get_vault_hand_pos(progress, "LeftHand");

   // Adjust to actual obstacle
   let offset = obstacle_height - anim_obstacle_height;
   let adjusted_pos = anim_hand_pos + Vec3::Y * offset;

   ik_targets.left_hand_target = Some(adjusted_pos);
   ```

3. **Test and Polish**
   - Verify bone transforms are in correct coordinate space
   - Add blend factor for smooth transitions
   - Test with obstacles of varying heights
   - Add foot IK for climbing actions

---

**Last Updated**: 2025-11-15
**Bevy Version**: 0.17
**Dependencies**: avian3d (physics), bevy-tnua (character controller), bevy_mod_inverse_kinematics
