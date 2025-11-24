# Pose Extraction Guide

Complete step-by-step guide for extracting pose data from GLB files.

## 📋 Overview

This tool converts single-frame GLB pose files into optimized RON (Rusty Object Notation) files for the procedural animation system.

**Workflow**:
```
Blender (create poses) → Export GLBs → Run extraction → RON files ready
```

## 🗂️ File Structure

### Input
Place your pose GLB files in:
```
assets/poses/
├── idle.glb
├── walk_left.glb
├── walk_right.glb
├── run_left.glb
├── run_right.glb
├── jump_takeoff.glb
├── jump_air.glb
├── jump_land.glb
├── roll_left.glb
├── roll_right.glb
├── attack_punch.glb
├── attack_kick.glb
└── crouch.glb
```

### Output
Extracted RON files will be saved to:
```
assets/poses_ron/
├── idle.pose.ron
├── walk_left.pose.ron
├── walk_right.pose.ron
... etc
```

## 🎯 Step-by-Step Instructions

### Step 1: Prepare Pose GLB Files

In Blender:

1. Open your character rig
2. Create each of the 13 poses:
   - Delete all keyframes except the one you want
   - Position the character in the desired pose
   - Ensure the armature is selected
   - File → Export → glTF 2.0 (.glb)
   - Name it appropriately (e.g., `idle.glb`, `walk_left.glb`)
   - Export settings:
     - Format: **GLB (Binary)**
     - Include: **Selected Objects** (armature)
     - Transform: **+Y Up** (Bevy default)
     - Geometry: Can disable meshes if only extracting bones
3. Save each pose GLB to `assets/poses/`

### Step 2: Enable Procedural Animation Plugin

Edit `src/main.rs`:

```rust
// Uncomment this line:
app.add_plugins(procedural_animation::ProceduralAnimationPlugin);
```

This is required because the extraction systems are part of the plugin.

### Step 3: Run Extraction

In your terminal:

```bash
# Run with extraction feature enabled
cargo run --features extract_poses

# Or combine with dev features
cargo run --features "dev_native,extract_poses"
```

### Step 4: Monitor Progress

You'll see output like:

```
🎬 Pose extraction mode ENABLED (feature flag active)
🎬 Loading 13 pose GLB files from assets/poses/
  Loading: poses/idle.glb → Idle
  Loading: poses/walk_left.glb → WalkLeftFootForward
  ... etc
Loading poses: 5/13
Loading poses: 10/13
Loading poses: 13/13
✓ All 13 pose GLBs loaded. Starting extraction...
✓ Extracted pose 'Idle' with 67 bones
✓ Saved pose to assets/poses_ron/idle.pose.ron
✓ Extracted pose 'Walk Left' with 67 bones
✓ Saved pose to assets/poses_ron/walk_left.pose.ron
... etc
✅ Pose extraction complete! Check assets/poses_ron
```

### Step 5: Verify RON Files

Check `assets/poses_ron/` for the generated `.pose.ron` files.

Example content:
```ron
(
    name: "Idle",
    bone_transforms: {
        "mixamorig12:Hips": (
            translation: (0.0, 0.9523, 0.0),
            rotation: (0.0, 0.0, 0.0, 1.0),
            scale: (1.0, 1.0, 1.0),
        ),
        // ... all bones
    },
    metadata: (
        source_animation: Some("Idle.glb"),
        notes: Some("Neutral standing pose"),
    ),
)
```

### Step 6: Use Poses (Normal Runtime)

Once extracted, disable the extraction feature and run normally:

```bash
# Regular run (extraction code not compiled)
cargo run
```

The procedural animation system will load the `.pose.ron` files from `assets/poses_ron/`.

## 📝 Customizing Pose Mappings

Edit `src/procedural_animation/extraction.rs` to customize which GLB files map to which poses:

```rust
impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            pose_mapping: vec![
                PoseMapping {
                    glb_name: "idle".to_string(),  // Looks for assets/poses/idle.glb
                    pose_id: PoseId::Idle,         // Maps to this pose type
                    notes: Some("Neutral standing pose".to_string()),
                },
                // Add more mappings...
            ],
        }
    }
}
```

## ⚙️ Configuration Options

In `src/procedural_animation/extraction.rs`:

```rust
impl Default for ExtractionMode {
    fn default() -> Self {
        Self {
            enabled: false,
            poses_input_path: "poses".to_string(),          // Input: assets/poses/
            ron_output_path: "assets/poses_ron".to_string(), // Output: assets/poses_ron/
        }
    }
}
```

Change these paths if you want to organize files differently.

## 🔧 Troubleshooting

### No bones found in pose
```
⚠️  No bones found in pose 'Idle'
```

**Solution**: Make sure:
- GLB includes the armature
- Bones have names (check in Blender)
- Export settings include armature

### GLB file not found
```
Loading poses: 3/13  (stuck here)
```

**Solution**: Check:
- Files are in `assets/poses/` directory
- Filenames match exactly (case-sensitive)
- Files have `.glb` extension

### Wrong transforms extracted
The extracted transforms are local (bone-relative), not world-space. This is correct for pose blending.

### Missing poses
You don't need all 13 poses to test. Start with a few:
- `idle.glb` - Essential
- `walk_left.glb`, `walk_right.glb` - For walk cycle
- `run_left.glb`, `run_right.glb` - For run cycle

The system will warn about missing poses but won't crash.

## 📊 Expected File Sizes

- **Input GLB**: 50-200 KB per file (with mesh), 5-20 KB (bones only)
- **Output RON**: 10-30 KB per file (depends on bone count)
- **Total for 13 poses**: ~150-400 KB (RON files)

## 🎨 Blender Export Tips

**Optimal Export Settings**:
```
Format: glTF Binary (.glb)
Include:
  ✓ Selected Objects
  ✓ Custom Properties

Transform:
  ✓ +Y Up

Geometry:
  ✗ Mesh (optional - not needed for poses)

Animation:
  ✓ Animation (even if just one frame)
  ✓ Shape Keys (if using)
```

**Bone Naming**:
- Keep original Mixamo names (`mixamorig12:BoneName`)
- Or use custom names (system stores whatever names exist)
- Be consistent across all pose files

## 🚀 Quick Start (TL;DR)

```bash
# 1. Place pose GLBs in assets/poses/
# 2. Uncomment plugin in src/main.rs
# 3. Run extraction
cargo run --features extract_poses

# 4. Check output in assets/poses_ron/
# 5. Done! Comment out plugin and run normally
cargo run
```

## ✅ Verification Checklist

- [ ] All 13 GLB files in `assets/poses/`
- [ ] ProceduralAnimationPlugin uncommented in main.rs
- [ ] Ran with `--features extract_poses`
- [ ] Saw "✅ Pose extraction complete!" message
- [ ] All 13 `.pose.ron` files in `assets/poses_ron/`
- [ ] Opened one RON file to verify bone data
- [ ] Commented out plugin for normal use
- [ ] Ready to use procedural animation!

## 🔗 Related Documentation

- [PROCEDURAL_ANIMATION.md](PROCEDURAL_ANIMATION.md) - Full system guide
- [README.md](README.md) - Project overview
- RON format specification: https://github.com/ron-rs/ron
