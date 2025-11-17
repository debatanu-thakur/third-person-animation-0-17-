# Target Matching Plugin - Usage Guide

## Overview

The Target Matching plugin is now integrated into your project! It provides Unity-style animation target matching for precise character movement control, especially useful for parkour, climbing, and dynamic foot placement.

## Plugin Status: ✅ Implemented

All core components are complete and ready to use:

- ✅ Animation masking system
- ✅ Procedural curve generation
- ✅ IK integration support
- ✅ Mixamo rig support with `mixamorig12:` prefix handling
- ✅ Debug visualization framework
- ✅ Bone mapping system

## Quick Start

### 1. Plugin is Already Enabled

The plugin has been added to `src/game/mod.rs` and is automatically initialized with your game.

### 2. Enable Target Matching on Your Player

Add the `TargetMatchEnabled` component to your player entity:

```rust
use crate::game::target_matching::{TargetMatchEnabled, BoneMap};

// In your player spawn system:
commands.spawn((
    Player,
    TargetMatchEnabled,  // Enable target matching
    BoneMap::default(),  // Will be populated automatically
    // ... other components
));
```

### 3. Request Target Matching

Use the convenience trait or component to request target matching:

```rust
use crate::game::target_matching::{TargetMatchRequest, TargetBone};

// Method 1: Using the convenience trait
commands.entity(player_entity)
    .match_target(
        TargetBone::LeftFoot,
        target_position,  // Vec3 world position
        1.2,              // Animation duration in seconds
    );

// Method 2: Using the component directly
commands.entity(player_entity).insert(
    TargetMatchRequest::new(
        TargetBone::RightFoot,
        ledge_position,
        jump_duration,
    ).with_window(0.0, 0.8)  // Custom match window
);
```

## Real-World Example: Jump to Ledge

Here's how to integrate target matching with your existing jump system:

```rust
use bevy::prelude::*;
use crate::game::{
    player::Player,
    animations::models::AnimationState,
    target_matching::{TargetMatchRequest, TargetBone},
};

fn handle_parkour_jump(
    mut commands: Commands,
    player_query: Query<Entity, With<Player>>,
    // Detect ledge with raycast or triggers
    ledge_position: Vec3,  // From your ledge detection system
) {
    let player_entity = player_query.single();

    // Request left foot to land on ledge
    commands.entity(player_entity).insert(
        TargetMatchRequest::new(
            TargetBone::LeftFoot,
            ledge_position,
            1.5,  // Jump animation duration
        )
        .with_window(0.1, 0.8),  // Start matching at 10%, finish at 80%
    );

    // Trigger your jump animation
    // The target matching will run alongside it
}
```

## Available Target Bones

```rust
pub enum TargetBone {
    LeftFoot,
    RightFoot,
    LeftHand,
    RightHand,
    Head,
    Hips,
}
```

Each bone knows its:
- Mixamo rig name (handles `mixamorig12:` prefix automatically)
- IK chain (for natural limb bending)
- Mask group (for selective animation control)

## Debug Visualization

Enable debug visualization to see targets and bones:

```rust
use crate::game::target_matching::TargetMatchDebugSettings;

// In your setup or debug system:
app.insert_resource(TargetMatchDebugSettings {
    show_targets: true,    // Red spheres at target positions
    show_bones: true,      // Green spheres at bone positions
    show_trajectories: false,
    target_color: Color::srgb(1.0, 0.0, 0.0),
    gizmo_size: 0.1,
});
```

## How It Works

### 1. Masking Strategy

The plugin uses animation masks to selectively control which bones are affected by which animations:

- **Body Group (0)**: Torso, spine, shoulders, head
- **Left Leg Group (1)**: Left leg chain (hip → knee → ankle → foot)
- **Right Leg Group (2)**: Right leg chain
- **Left Arm Group (3)**: Left arm chain
- **Right Arm Group (4)**: Right arm chain
- **Head Group (5)**: Head only

When you request target matching on a foot:
- The main animation continues on the body
- The matched leg can be controlled separately

### 2. Automatic Bone Discovery

The `build_bone_map` system automatically discovers bones in your character:

```rust
// Runs automatically when TargetMatchEnabled is added
// Looks for bones like:
// - "mixamorig12:LeftFoot"
// - "mixamorig12:RightHand"
// etc.

// Populates the BoneMap component for quick lookups
```

### 3. Target Matching Lifecycle

```
1. Insert TargetMatchRequest component
   ↓
2. handle_target_match_requests detects new request
   ↓
3. State changes to TargetMatchingState::Matching
   ↓
4. Custom curve or IK applied to bone
   ↓
5. update_active_matching monitors progress
   ↓
6. State changes to Complete after duration
   ↓
7. Request component removed
```

## Integration with Your Animation System

The target matching plugin works alongside your existing animation controller:

```rust
// Your animation system plays the jump animation
transitions.play(
    &mut animation_player,
    animation_nodes.jumping,
    Duration::from_millis(50)
);

// Target matching plugin handles the foot placement
// Both run simultaneously without conflicts due to masking!
```

## Advanced Usage

### Custom Easing Functions

```rust
use crate::game::target_matching::curve_generator::{
    generate_target_curve_with_easing,
    EasingFunction,
};

// For custom curve generation (advanced):
let curve = generate_target_curve_with_easing(
    &request,
    bone_target_id,
    current_position,
    EasingFunction::EaseInOut,  // Smooth acceleration and deceleration
);
```

### Matching Multiple Bones

```rust
// Match both feet simultaneously
commands.entity(player).insert((
    TargetMatchRequest::new(TargetBone::LeftFoot, left_target, duration),
    TargetMatchRequest::new(TargetBone::RightFoot, right_target, duration),
));
// Note: Current implementation handles one at a time
// Multi-bone support is planned for future enhancement
```

### Root Offset Calculation

```rust
use crate::game::target_matching::curve_generator::calculate_root_offset;

// Alternative approach: move character root instead of bone
let offset = calculate_root_offset(
    bone_world_pos,
    target_pos,
    character_root_pos,
);

character_transform.translation += offset;
```

## Next Steps

### Immediate Testing

1. **Add TargetMatchEnabled** to your player in `src/game/player/mod.rs`
2. **Create a test trigger** that requests a foot target match
3. **Enable debug visualization** to see it in action

### Future Enhancements

- [ ] Pre-sample animation curve data from GLTF
- [ ] Multi-bone simultaneous matching
- [ ] Physics override integration with Tnua
- [ ] Animation events for match timing
- [ ] Editor tools for visual target placement

## Troubleshooting

### Bones Not Found

**Problem**: BoneMap is empty or missing bones

**Solution**:
- Check that your character has `AnimationTargetId` components on bones
- Verify bone names match Mixamo convention (with or without `mixamorig12:` prefix)
- Enable debug logging to see bone discovery

### Target Matching Not Visible

**Problem**: No visible effect when requesting target match

**Solution**:
- Enable debug visualization to see targets
- Check that animation is playing
- Verify TargetMatchEnabled component is present
- Look for warnings/errors in console

### IK Not Working

**Problem**: IK constraint not applied

**Solution**:
- Ensure `bevy_mod_inverse_kinematics` plugin is active
- Check IK chain length matches bone hierarchy
- Verify bone entities exist in BoneMap

## API Reference

See [`target_matching.md`](./target_matching.md) for full technical documentation.

## Example: Complete Parkour System

```rust
use bevy::prelude::*;
use crate::game::target_matching::*;

#[derive(Component)]
struct LedgeDetector;

fn detect_and_jump_to_ledge(
    mut commands: Commands,
    player: Query<(Entity, &Transform), With<Player>>,
    ledges: Query<&GlobalTransform, With<Ledge>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if !keyboard.just_pressed(KeyCode::Space) {
        return;
    }

    let (player_entity, player_transform) = player.single();

    // Find closest ledge in front of player
    if let Some(ledge_pos) = find_closest_ledge_ahead(
        player_transform,
        &ledges,
    ) {
        // Request target matching for jump
        commands.entity(player_entity)
            .match_target(
                TargetBone::LeftFoot,
                ledge_pos,
                1.2,  // Jump duration
            );

        info!("Jumping to ledge at {:?}", ledge_pos);
    }
}

fn find_closest_ledge_ahead(
    player_transform: &Transform,
    ledges: &Query<&GlobalTransform, With<Ledge>>,
) -> Option<Vec3> {
    // Your ledge detection logic here
    // Return the landing position
    None
}
```

## Credits

This plugin implements concepts from:
- Unity's Animator.MatchTarget API
- Bevy's animation mask system (animation_masks.rs example)
- IK integration via bevy_mod_inverse_kinematics

Built specifically for Mixamo character rigs in Bevy 0.17.
