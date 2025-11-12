# Mixamo Animation Setup Guide

The placeholder files in this directory need to be replaced with actual Mixamo animations.

## Quick Start: Download from Mixamo

1. **Go to Mixamo**: https://www.mixamo.com/
2. **Sign in** with Adobe account (free)
3. **Select Character**: Choose "Brian" or any character
4. **Download these animations** (search by name):

### Required Animations (Phase 1):
- **Breathing Idle** → Save as `Breathing Idle.glb`
- **Standard Run** → Save as `Standard Run.glb`
- **Standing Jumping** → Save as `Standing Jumping.glb`
- **Hard Landing** → Save as `Hard Landing.glb`

### Future Parkour Animations (Phase 2+):
- **Jump To Hang** → Save as `Jump To Hang.glb`
- **Freehang Climb** → Save as `Freehang Climb.glb`
- **Over Obstacle Jumping** → Save as `Over Obstacle Jumping.glb`
- **Running Slide** → Save as `Running Slide.glb`
- **Falling To Roll** → Save as `Falling To Roll.glb`
- **Braced Hang** → Save as `Braced Hang.glb`
- **Braced Hang To Crouch** → Save as `Braced Hang To Crouch.glb`
- **Braced Hang Drop** → Save as `Braced Hang Drop.glb`
- **Free Hang To Braced** → Save as `Free Hang To Braced.glb`
- **Standing Jump To Freehang** → Save as `Stand To Freehang.glb`

## Download Settings

When downloading from Mixamo, use these settings:

### Format:
- **GLB (binary)**

### Skin:
- **With Skin** (if you want the character mesh)
- **Without Skin** (if you only want animation - recommended for this project since we use Brian model separately)

### FPS:
- **30 fps** (standard for games)

### Keyframe Reduction:
- **None** (for best quality)
- Or **Uniform** (for smaller files)

## File Structure

After downloading, your directory should look like:
```
assets/models/animations/
├── Breathing Idle.glb         (10-50 KB)
├── Standard Run.glb            (10-50 KB)
├── Standing Jumping.glb        (10-50 KB)
├── Hard Landing.glb            (10-50 KB)
├── ... other animations ...
└── README.md (this file)
```

## Character Model

The character model (Brian) should also be downloaded from Mixamo:

1. Go to Mixamo → Characters
2. Select "Brian" character
3. Download in **T-pose** or **A-pose**
4. Format: **GLB (binary)**
5. Save to: `assets/models/characters/brian.glb`

## Testing

After replacing the placeholder files:
1. Run `cargo run`
2. Navigate to gameplay screen
3. You should see smooth animations:
   - Standing still → Idle animation
   - WASD movement → Running animation
   - Space bar (grounded) → Jump animation
   - Space bar (airborne) → Fall animation

## Troubleshooting

**Error: "does not contain the labeled asset 'Animation0'"**
- This means the placeholder files haven't been replaced yet
- Download real animations from Mixamo following the steps above

**Character is a T-pose statue:**
- Animations not loaded yet (normal with placeholders)
- Movement still works via Tnua physics controller
- Replace with real Mixamo animations to see smooth transitions

**Animation is choppy:**
- Try downloading at higher FPS (60 instead of 30)
- Disable keyframe reduction for smoother motion

## Alternative: Use Different Character

If you want to use a different Mixamo character:
1. Download your chosen character model
2. Replace `brian.glb` with your character
3. Download the same animations list above
4. All animations should work with any Mixamo character!
