# Parkour Animation Bone Extraction Workflow

This guide explains how to extract bone poses from Mixamo animations to create procedural parkour animations.

## 🎯 Goal

Extract key bone poses from reference animations and save them as RON files. These poses will be used to:
1. Apply IK (Inverse Kinematics) to hands/feet during parkour
2. Create procedural animations that adapt to obstacle heights
3. Blend between poses for smooth parkour movements

## 📋 Workflow

### Step 1: Download Mixamo Animations

1. Go to [Mixamo](https://www.mixamo.com)
2. Download parkour animations (suggested list below)
3. Export as **FBX for Unity** (this works best with Bevy)
4. Use the same character rig as your `brian_parkour.glb`

**Suggested Animations:**
- Slot 1: Standing Vault
- Slot 2: Running Vault / Hurdling
- Slot 3: Climbing Up Wall
- Slot 4: Wall Run Left
- Slot 5: Wall Run Right
- Slot 6: Sliding
- Slot 7: Rolling / Recovery Fall
- Slots 8-9: Reserved for future

### Step 2: Add Animations to GLTF

**Option A: Append to existing GLTF (Blender)**
1. Open `brian_parkour.glb` in Blender
2. Import the Mixamo FBX files
3. Rename animations to match slot names:
   - `debug_1` or `standing_vault`
   - `debug_2` or `running_vault`
   - `debug_3` or `climb_up`
   - etc.
4. Export as GLB, overwriting `assets/models/characters/brian_parkour.glb`

**Option B: Temporary separate GLTF**
1. Create a new Blender file with your character
2. Import all parkour animations
3. Name them as above
4. Export as `brian_parkour.glb` (temporarily replace the original)

### Step 3: Extract Bone Poses

1. **Run the game**
   ```bash
   cargo run
   ```

2. **Check console for debug animations**
   You should see:
   ```
   DEBUG: Found parkour debug animations for bone extraction!
     - Slot 1 (press '1')
     - Slot 2 (press '2')
     ...
   Press F12 to dump current bone transforms to RON file
   ```

3. **Play an animation**
   - Press `1` to play the animation in slot 1
   - The animation will start playing on your character

4. **Pause at key moments and extract**
   - Let the animation play to an important pose (e.g., hands on obstacle)
   - Press **F12** to dump current bone transforms
   - A RON file will be created in `assets/parkour_poses/`

5. **Repeat for multiple keyframes**
   For each animation, extract 3-5 key poses:
   - **Start pose** (beginning of movement)
   - **Contact pose** (hands/feet touch obstacle)
   - **Mid-vault pose** (body over obstacle)
   - **Landing pose** (feet landing)
   - **Recovery pose** (return to neutral)

### Step 4: Review Generated RON Files

After pressing F12, check:
```
assets/parkour_poses/debug_1_0.50s.ron
assets/parkour_poses/debug_1_1.20s.ron
assets/parkour_poses/debug_1_2.00s.ron
```

Each file contains:
```ron
(
    time: 1.20,
    description: Some("Extracted at 1.20s"),
    bones: [
        (
            bone_name: "mixamorig:LeftHand",
            position: (0.5, 1.2, 0.3),
            rotation: (0.0, 0.707, 0.0, 0.707),
        ),
        // ... more bones
    ]
)
```

### Step 5: Organize Poses

1. Rename files descriptively:
   ```
   standing_vault_start.ron
   standing_vault_hands_on.ron
   standing_vault_mid.ron
   standing_vault_landing.ron
   ```

2. Create a master animation file combining multiple poses:
   ```ron
   // assets/parkour_poses/standing_vault.ron
   (
       name: "standing_vault",
       duration: 1.5,
       key_poses: [
           // Paste content from individual pose files here
       ],
       root_motion: None,
   )
   ```

## 🎮 Controls

| Key | Action |
|-----|--------|
| `1-9, 0` | Play debug animation in corresponding slot |
| `F12` | Extract current bone poses to RON file |
| `Space` | (Parkour actions when implemented) |

## 🦴 Bones Extracted

The system extracts these critical bones:

**Arms (IK targets):**
- mixamorig:LeftHand, RightHand
- mixamorig:LeftForeArm, RightForeArm
- mixamorig:LeftArm, RightArm

**Legs (IK targets):**
- mixamorig:LeftFoot, RightFoot
- mixamorig:LeftLeg, RightLeg
- mixamorig:LeftUpLeg, RightUpLeg

**Core (body orientation):**
- mixamorig:Spine, Spine1, Spine2
- mixamorig:Hips

**Head (look direction):**
- mixamorig:Head, Neck

## 📝 Tips

- **Extract at key contact points:** When hands/feet touch obstacles
- **Multiple angles:** Get start, middle, and end poses
- **Use animation scrubbing:** Pause/play to find perfect frames
- **Take notes:** Document what each pose represents
- **Verify bone names:** Check console to ensure bones are found

## 🔄 Next Steps

After extraction:
1. Claude will implement the pose interpolation system
2. Connect poses to obstacle detection
3. Set up IK to reach detected obstacle points
4. Add easing curves for smooth transitions
5. Test and refine

## 🚨 Troubleshooting

**"No debug animation playing!"**
- Make sure you pressed 1-9 first before F12

**"No bones found!"**
- Check that the animation is actually playing
- Verify bone names match Mixamo rig (`mixamorig:*`)

**No file created**
- Check write permissions in `assets/parkour_poses/`
- Look for errors in console

**Animation not found**
- Verify animation names in GLTF match `debug_1`, `debug_2`, etc.
- Check console for available animation names

## 💡 Alternative: Hand-Author Poses

If you don't have animations yet, you can manually create pose files:

```ron
(
    time: 0.0,
    description: Some("Hand-authored vault start"),
    bones: [
        (
            bone_name: "mixamorig:LeftHand",
            position: (0.3, 1.0, 0.2),  // Relative positions
            rotation: (0.0, 0.0, 0.0, 1.0),  // Quaternion
        ),
        // Add more bones...
    ]
)
```

---

**Ready to start?** Download your Mixamo animations and let's extract some poses! 🚀
