# Animation Retargeting in Bevy 0.17

## The Problem

Animations from external GLTF files won't play on your character even if bone names match perfectly.

**Root Cause**: Bevy 0.17 uses `AnimationTargetId` based on **entity hierarchy paths**, not just bone names.

## Example

**Character** (`brian_parkour.glb`):
```
brian → mixamorig12:Hips → mixamorig12:Spine → ...
```

**Animation** (`vault.glb` from Mixamo via Blender):
```
Armature → mixamorig12:Hips → mixamorig12:Spine → ...
```

Even though bone names match (`mixamorig12:Hips`), the **paths are different**:
- Character path: `["brian", "mixamorig12:Hips"]` → AnimationTargetId UUID-A
- Animation path: `["Armature", "mixamorig12:Hips"]` → AnimationTargetId UUID-B

**Different UUIDs = No animation!** ❌

## The Solution

Rename the root node in your animation GLTF to match your character's root node.

### In Blender:

1. **Open** `vault.glb` in Blender
2. **Select** the "Armature" object in the outliner
3. **Rename** it to "brian" (or whatever your character's root is named)
4. **Export** → glTF 2.0 → Save as `vault.glb`

Now the paths match:
- Character: `["brian", "mixamorig12:Hips"]` → UUID-A
- Animation: `["brian", "mixamorig12:Hips"]` → UUID-A ✅

## Verification

After fixing, you should see:
- Press 'V' to trigger vault animation
- Character performs vaulting motion
- No T-pose or freeze

## Apply to All Animations

For each Mixamo animation (`climb.glb`, `slide.glb`, etc.):
1. Open in Blender
2. Rename root node to "brian"
3. Re-export

## Credits

Solution discovered through forum discussion with pcwalton (Bevy contributor):
> "I had problems until I made the rig exactly match the Mixamo one, including things like bone roll"

Key insight: Hierarchy paths must match **exactly** for AnimationTargetId to work across GLTFs.
