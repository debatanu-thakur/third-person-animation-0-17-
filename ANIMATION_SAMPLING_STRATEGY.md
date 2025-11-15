# Runtime Animation Sampling Strategy

## Problem with Direct Curve Sampling

Bevy 0.17's `VariableCurve` API is not publicly exposed for direct sampling. The curves are internal to the animation system.

## Better Approach: Sample Playing Animations

Instead of reading curve data, we:
1. Play the parkour animation
2. Let Bevy's animation system interpolate curves
3. Read the resulting bone transforms at specific times
4. Store those transforms for IK use

## Implementation

```rust
// 1. Load animation into AnimationGraph
let (graph, vault_index) = AnimationGraph::from_clip(vault_clip);

// 2. Play animation
player.play(vault_index);

// 3. At specific times, pause and sample bone transforms
player.seek_to(0.5); // Seek to 0.5 seconds
player.pause();

// 4. Read bone transforms (GlobalTransform components)
for (name, transform) in bone_query.iter() {
    if name == "mixamorig:LeftHand" {
        left_hand_pos = transform.translation();
    }
}

// 5. Store for IK
ik_targets.left_hand = left_hand_pos;
```

## Benefits

- ✅ No need to understand VariableCurve internals
- ✅ Bevy handles all curve interpolation
- ✅ Works with any animation format
- ✅ Can sample at any time point
- ✅ Gets exact result that would be rendered

## Implementation Status

### ✅ Completed

1. **AnimationGraph Setup** - `src/game/parkour_animations/mod.rs`
   - Loads animation clips using `GltfAssetLabel::Animation(0).from_asset(path)`
   - Creates AnimationGraph from each clip using `AnimationGraph::from_clip()`
   - Stores graph handles and node indices for each parkour animation

2. **Resource Structure** - `SampledParkourPoses`
   - Stores sampled bone transforms at key times
   - Provides helper methods like `get_vault_hand_pos(time, hand)`
   - Ready for IK system integration

### ⏳ Next Steps

1. **Implement Runtime Sampling System**
   - Spawn temporary entity with skeleton
   - Attach AnimationPlayer + AnimationGraphHandle
   - Play animation and seek to specific times
   - Read bone GlobalTransforms
   - Store in SampledParkourPoses resource

2. **Integrate with IK System**
   - Use sampled hand/foot positions as IK targets
   - Adapt targets based on obstacle height
   - Blend between sampled poses during parkour actions

3. **Test with Actual Character**
   - Verify bone name matching
   - Ensure transforms are in correct space
   - Debug visualization of sampled poses
