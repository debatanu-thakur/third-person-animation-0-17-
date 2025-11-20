# IK Integration Summary

## Changes Made to Target Matching System

This document summarizes the changes made to integrate the custom FABRIK IK solver with the target matching system for Bevy 0.17.

## Key Fixes Applied

### 1. IK Plugin Registration
**File**: `src/main.rs`
- Added `InverseKinematicsPlugin` to the app
- This activates the custom IK solver defined in `src/ik/`

### 2. System Scheduling Fix
**File**: `src/ik/mod.rs`
- **Problem**: IK solver was running in `Update` schedule, but Bevy animations run in `PostUpdate`
- **Solution**: Moved IK solver to `PostUpdate` schedule to run AFTER animations
- **Result**: IK can now override animation bone rotations instead of being overwritten

```rust
app.add_systems(PostUpdate, solver::inverse_kinematics_system);
```

### 3. Component Fix
**File**: `src/ik/solver.rs`
- **Problem**: Used non-existent `ChildOf` component
- **Solution**: Changed to use Bevy's `Parent` component (available in prelude)
- Updated `parents.get(joints[i])?.get()` to access parent entity

### 4. Request Handling
**File**: `src/game/target_matching/systems.rs`
- **Problem**: `Added<TargetMatchRequest>` only triggered once, but request component is re-inserted every 0.1s
- **Solution**: Changed to `Changed<TargetMatchRequest>` filter
- **Result**: System now detects and processes request updates continuously

### 5. IK Target Updates
**File**: `src/game/target_matching/systems.rs`
- **Problem**: System created new IK constraints every frame
- **Solution**: Added logic to check if IK constraint already exists
  - If exists: Update existing target position
  - If not: Create new constraint
- **Result**: Efficient target updates without recreating entities

### 6. Disabled Conflicting System
**File**: `src/game/target_matching/mod.rs`
- Commented out `update_active_matching` system
- **Reason**: This system directly manipulated bone transforms, conflicting with IK solver
- **Result**: IK solver has exclusive control over masked bones

## Animation Mask Configuration

**File**: `src/game/animations/animation_controller.rs`

Animation clips use mask `0b00001` which means:
- **Group 0** (bit 0 = 1): Body animations ARE applied
- **Groups 1-4** (bits 1-4 = 0): Hands/feet animations are EXCLUDED

Bone group assignments:
- Group 1: Left Foot
- Group 2: Right Foot
- Group 3: Left Hand
- Group 4: Right Hand

This allows IK to control hands/feet while animations control the body.

## System Execution Order

```
Update Schedule:
  1. build_bone_map - Discovers bone entities
  2. retry_bone_map_if_empty - Retries if scene not loaded
  3. handle_target_match_requests - Creates/updates IK targets
  4. debug_visualize_targets - Debug gizmos

PostUpdate Schedule (runs after Update):
  1. Animation systems (Bevy built-in) - Apply animations to group 0
  2. inverse_kinematics_system - Override groups 1-4 with IK
```

## How It Works

1. **Hand placement detection** (`src/game/hand_placement.rs`):
   - Raycasts forward from character
   - If wall detected < 1.5 units away
   - Inserts `TargetMatchRequest` with wall position

2. **Request handling** (`handle_target_match_requests`):
   - Detects changed `TargetMatchRequest` component
   - Checks if IK constraint already exists on bone
   - If yes: Updates existing IK target position
   - If no: Creates new IK target entity and constraint

3. **IK solving** (`inverse_kinematics_system`):
   - Runs in PostUpdate after animations
   - For each enabled `IkConstraint`:
     - Builds bone chain from constraint bone up to root
     - Runs FABRIK algorithm for N iterations
     - Adjusts bone rotations to reach target
   - Respects pole targets for natural bending

## Expected Behavior

- Hands should reach toward walls when within 1.5 units
- Feet should maintain contact with ground during animations
- Body animations continue normally (not affected by IK)
- IK overrides hand/foot positions from animation clips

## Testing Checklist

- [ ] Hands reach toward nearby walls
- [ ] Feet maintain ground contact on slopes
- [ ] Body animations play normally
- [ ] No jittering or popping
- [ ] IK targets update smoothly
- [ ] Performance is acceptable (20 iterations per constraint)

## Git Commits

1. `795c602` - Import Parent from bevy::hierarchy
2. `fc78efc` - Remove redundant hierarchy import for Bevy 0.17
3. `3f1c923` - Push syntax fix
4. `dfdcef1` - Disable update_active_matching to prevent IK conflicts
5. `c642bb4` - Use Changed filter and update IK targets instead of recreating

## Next Steps

If hands still don't reach walls after these changes:
1. Verify animation masks are actually excluding hands (check animation_controller.rs)
2. Add logging to IK solver to confirm it's running
3. Check GlobalTransform propagation
4. Verify IK chain_length matches bone hierarchy
5. Test with different IK iteration counts
