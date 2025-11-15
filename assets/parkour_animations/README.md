# Parkour Animations Directory

Place animation-only GLB files here for runtime sampling.

## Required Files

Download these animations from Mixamo (FBX format, **without character mesh**):

1. **vault.glb** - Vaulting over obstacles
2. **climb.glb** - Climbing up walls
3. **slide.glb** - Sliding under obstacles
4. **wall_run_left.glb** - Wall running (left side)
5. **wall_run_right.glb** - Wall running (right side)
6. **roll.glb** - Rolling/recovery

## File Format

- **Format**: GLB (preferred) or FBX
- **Character**: None (animation-only export)
- **Rig**: Same as your player character (Mixamo Y-Bot recommended)
- **Animation**: Single animation per file (at index 0)

## How to Download from Mixamo

1. Go to [mixamo.com](https://www.mixamo.com)
2. Search for animation (e.g., "Vault")
3. Click "Download"
4. **Settings:**
   - Format: FBX for Unity (.fbx)
   - Skin: **Without Skin** ← IMPORTANT!
   - Frames per second: 30
   - Keyframe reduction: None
5. Download and save as `<animation>.fbx`
6. Convert FBX to GLB using Blender (optional but recommended):
   - Import FBX
   - Export → glTF 2.0
   - Format: GLB
   - Include: Animations only
   - Save as `<animation>.glb`

## How It Works

The system will:
1. Load GLB files from this directory
2. Extract the first animation (index 0): `gltf.animations[0]`
3. Collect bone names for retargeting verification
4. Sample animation curves at specific times
5. Use sampled transforms for IK targets
6. Adapt to obstacle heights in real-time

## Bone Name Matching

The system verifies that bone names in animations match your character:

```
Character (brian_parkour.glb):        Animation (vault.glb):
mixamorig:Hips                    →   mixamorig:Hips          ✅
mixamorig:Spine                   →   mixamorig:Spine         ✅
mixamorig:LeftHand                →   mixamorig:LeftHand      ✅
```

If bone names match → Automatic retargeting works!

Check the console for verification messages when animations load.

## Debug Commands

In-game:
- **Press P** - Sample vault animation at 0.5s and print bone transforms

Console will show:
```
📊 Sampled vault animation at 0.5s:
   mixamorig:Hips → pos: Vec3(0.0, 1.0, 0.0), rot: Quat(...)
   mixamorig:Spine → pos: Vec3(0.0, 1.2, 0.0), rot: Quat(...)
   ...
```

## Troubleshooting

**"Parkour animations not loaded yet!"**
- Check that GLB files are in this directory
- Verify file names match exactly (vault.glb, climb.glb, etc.)
- Check console for loading errors

**"Only X/Y bones matched"**
- Your animation rig doesn't match your character rig
- Download animations for the same character type
- Use Mixamo Y-Bot for both character and animations

**"No animation found in GLB"**
- Make sure you exported **with animation**
- Verify animation is at index 0 in the GLB file

## Next Steps

After adding files:
1. Run the game
2. Check console for "✅ Loaded <animation> animation"
3. Verify bone matching: "✅ vault: X/Y bones matched (100%)"
4. Press P to test animation sampling
5. System will use sampled data for IK targets automatically!
