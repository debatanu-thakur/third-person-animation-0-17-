# Animation Target Matching in Bevy 0.17

## Overview

This document outlines our approach to implementing Unity-style animation target matching in Bevy 0.17. Target matching allows precise control over where specific body parts (hands, feet) land during animations, crucial for parkour systems, climbing, and dynamic interactions.

## What is Target Matching?

**Unity's MatchTarget** adjusts the character's root transform during animation playback to ensure a specific body part reaches a target position at a specific time in the animation.

### Key Differences from IK

- **Target Matching**: Moves the entire character root to place a limb at the target
- **Inverse Kinematics (IK)**: Bends the limbs to reach the target while keeping the root stationary

### Common Use Cases

- Jumping to specific ledges
- Stepping on precise footholds
- Grabbing overhead objects
- Landing on uneven terrain
- Parkour movement systems

## Bevy 0.17 Animation System Capabilities

### Available Features

✅ **Animation Graphs** - Blending and layering animations
✅ **Animation Masks** - Selective bone group control
✅ **Custom Curves** - Programmatic animation creation via `AnimatableCurve`
✅ **Animation Transitions** - Smooth blending between states
✅ **Target-based Animation** - `AnimationTargetId` for bone mapping

### Missing Features

❌ **Native Target Matching API** - No built-in Unity-style MatchTarget
❌ **Root Motion** - No automatic root motion extraction
❌ **Curve Sampling API** - No public API to sample animation curves (PR #16395 pending)

## Our Implementation Strategy

We're implementing a **hybrid masking + IK approach** that combines the best of three techniques:

### 1. Animation Masking (Core)

Use Bevy's animation mask system to selectively control which bones are affected by which animations.

```rust
// Mask groups
const MASK_GROUP_BODY: u32 = 0;      // Upper body, torso
const MASK_GROUP_LEFT_LEG: u32 = 1;  // Left leg chain
const MASK_GROUP_RIGHT_LEG: u32 = 2; // Right leg chain

// Main animation affects only body
animation_graph.add_clip_with_mask(
    jump_animation,
    0x01,  // Only MASK_GROUP_BODY
    1.0,
    blend_node
);
```

### 2. Custom Animation Curves (Override)

Generate procedural animation curves for masked bones to reach specific targets.

```rust
// Create custom foot curve
let mut clip = AnimationClip::default();
clip.add_curve_to_target(
    foot_target_id,
    AnimatableCurve::new(
        animated_field!(Transform::translation),
        UnevenSampleAutoCurve::new(
            times.into_iter().zip(positions)
        )
    )
);
```

### 3. Inverse Kinematics (Refinement)

Use `bevy_mod_inverse_kinematics` for natural leg bending and final placement.

```rust
// IK constraint for foot placement
IkConstraint {
    chain_length: 3,  // Hip -> Knee -> Ankle
    target: target_entity,
    pole_target: Some(knee_direction),
}
```

## Architecture: Target Matching Plugin

### Plugin Structure

```
src/game/target_matching/
├── mod.rs              # Plugin definition and public API
├── mask_setup.rs       # Animation mask configuration
├── curve_generator.rs  # Custom curve creation
├── ik_integration.rs   # IK system integration
├── components.rs       # Target matching components
└── systems.rs          # Update systems
```

### Core Components

**TargetMatchRequest**: Component to trigger target matching
```rust
pub struct TargetMatchRequest {
    pub bone: TargetBone,
    pub target_position: Vec3,
    pub match_window: (f32, f32), // Normalized time (0.0-1.0)
    pub animation_duration: f32,
}
```

**MaskGroupConfig**: Resource defining bone groups
```rust
pub struct MaskGroupConfig {
    pub groups: HashMap<String, u32>,
    pub bone_assignments: HashMap<AnimationTargetId, u32>,
}
```

**TargetMatchingState**: Tracks active matching operations
```rust
pub enum TargetMatchingState {
    Idle,
    Matching { start_time: f32, request: TargetMatchRequest },
    Complete,
}
```

### Public API

```rust
// Add to your app
app.add_plugins(TargetMatchingPlugin);

// Configure for your character rig
app.insert_resource(MaskGroupConfig::for_mixamo());

// Request target matching
commands.entity(player).insert(TargetMatchRequest {
    bone: TargetBone::LeftFoot,
    target_position: ledge_position,
    match_window: (0.0, 0.8), // Match from start to 80% through animation
    animation_duration: 1.2,
});
```

## Implementation Phases

### Phase 1: Foundation
- Plugin structure and components
- Mask group configuration system
- Character rig analysis (Mixamo bone mapping)

### Phase 2: Masking System
- Automatic mask group assignment
- Animation graph setup with masks
- Integration with existing animation controller

### Phase 3: Curve Generation
- Procedural curve creation for target positions
- Timing and blending calculations
- Dynamic clip injection

### Phase 4: IK Integration
- Setup IK chains for legs/arms
- Coordinate IK with masked animations
- Pole target calculation

### Phase 5: Physics Integration
- Coordinate with Tnua physics controller
- Root position adjustment
- Smooth transitions

### Phase 6: Polish & API
- Error handling and validation
- Debug visualization
- Documentation and examples

## Technical Challenges & Solutions

### Challenge 1: Local vs World Space
**Problem**: Animation curves work in local bone space, targets are in world space
**Solution**: Calculate target position relative to character root or parent bone

### Challenge 2: Timing Synchronization
**Problem**: Custom curves must match main animation timing
**Solution**: Sample main animation duration, use normalized time ranges

### Challenge 3: Natural Movement
**Problem**: Only overriding foot can cause unnatural leg stretching
**Solution**: Use IK for the entire leg chain, not just end effector

### Challenge 4: Physics Conflicts
**Problem**: Physics controller may fight with target matching
**Solution**: Temporarily override physics during matching window, blend back after

## Mixamo Character Bone Structure

Common Mixamo rig hierarchy (to be confirmed with actual model):
```
Hips (root)
├── LeftUpLeg
│   ├── LeftLeg (knee)
│   │   └── LeftFoot (ankle)
│   │       └── LeftToeBase
├── RightUpLeg
│   ├── RightLeg
│   │   └── RightFoot
│   │       └── RightToeBase
└── Spine
    └── (upper body bones)
```

### Mask Group Assignments
- **Body Group**: Spine, chest, arms, head
- **Left Leg Group**: LeftUpLeg through LeftToeBase
- **Right Leg Group**: RightUpLeg through RightToeBase

## Testing Strategy

1. **Unit Tests**: Curve generation, mask assignment
2. **Integration Tests**: Full target matching cycle
3. **Visual Tests**: Debug gizmos for targets and trajectories
4. **Real-world Tests**: Jump to ledge, step on stones

## Future Enhancements

- [ ] Pre-baked animation data extraction from GLTF
- [ ] Multi-bone target matching (hands + feet simultaneously)
- [ ] Root motion support when available
- [ ] Animation curve sampling when PR #16395 merges
- [ ] Editor tools for visual target placement
- [ ] Animation events integration for match timing

## References

- [Unity Target Matching Docs](https://docs.unity3d.com/Manual/TargetMatching.html)
- [Bevy Animation Module](https://docs.rs/bevy/latest/bevy/animation/)
- [bevy_mod_inverse_kinematics](https://docs.rs/bevy_mod_inverse_kinematics/)
- Bevy Example: `animation_masks.rs`
- Bevy Example: `animated_transform.rs`

## Notes

- This system is designed to be modular and reusable across projects
- Configuration should be rig-agnostic where possible
- Performance considerations: curve generation should be cached when possible
- IK is optional but recommended for natural results
