# Parkour IK System Documentation

This document explains the Inverse Kinematics (IK) system for procedural parkour animations.

## 🎯 System Overview

The parkour IK system allows hands and feet to dynamically reach for obstacles during parkour actions. It combines:

1. **Obstacle Detection** - Raycasting to find obstacles
2. **IK Targets** - Dynamic target positions for hands/feet
3. **IK Chains** - bevy_mod_inverse_kinematics solving for bone rotations
4. **Pose Interpolation** - Blending between keyframe poses from RON files
5. **Visualization** - Debug gizmos showing IK targets

## 🏗️ Architecture

### Components

**ParkourIkTargets** - Attached to player
```rust
pub struct ParkourIkTargets {
    pub left_hand_target: Option<Vec3>,
    pub right_hand_target: Option<Vec3>,
    pub left_foot_target: Option<Vec3>,
    pub right_foot_target: Option<Vec3>,
    pub active: bool,
}
```

**IkConstraint** - Attached to hand/foot bones
```rust
pub struct IkConstraint {
    pub chain_length: usize,  // 2 for hand->forearm->arm
    pub iterations: usize,    // 20 for smooth solving
    pub target: Entity,       // IK target entity
    pub pole_target: Option<Entity>,  // For elbow/knee direction
    pub enabled: bool,
}
```

**ActivePoseAnimation** - For procedural animations
```rust
pub struct ActivePoseAnimation {
    pub animation: Handle<ParkourPoseAnimation>,
    pub start_time: f32,
    pub looping: bool,
}
```

### Resources

**IkConfig**
```rust
pub struct IkConfig {
    pub enabled: bool,              // Master IK toggle
    pub hand_spread: f32,           // 0.3m - distance between hands
    pub hand_height_offset: f32,    // 0.05m - above obstacle
    pub debug_visualization: bool,  // Show gizmos
}
```

## 🔄 System Flow

### 1. IK Setup (Once)
`setup_ik_chains()` runs after player model loads:
- Finds hand/foot bones by name (`mixamorig:LeftHand`, etc.)
- Spawns IK target entities
- Attaches `IkConstraint` components to bones
- Links bones to their targets

### 2. Obstacle Detection (Every Frame)
`detect_obstacles()` in `obstacle_detection.rs`:
- Casts 3 rays (upper, center, lower) from player
- Detects obstacle type and distance
- Stores hit point and ledge point in `ObstacleDetectionResult`

### 3. IK Target Update (Every Frame)
`update_ik_targets_from_obstacles()`:
- Checks parkour state (Vaulting, Climbing, Hanging)
- Calculates hand positions based on obstacle hit points
- Applies hand spread (left/right offset)
- Updates IK target entity transforms

### 4. IK Solving (Every Frame)
`bevy_mod_inverse_kinematics` plugin:
- Reads IkConstraint components
- Solves for bone rotations to reach targets
- Applies rotations to arm/leg bones
- Iterative solver (20 iterations for smoothness)

### 5. Visualization (Every Frame)
`visualize_ik_targets()`:
- Draws cyan sphere for left hand target
- Draws magenta sphere for right hand target
- Shows crosshairs for better depth perception

## 🎮 Usage Examples

### Basic IK During Vault

```rust
// Obstacle detection finds a 1m wall
detection.obstacle_type = ObstacleType::MediumObstacle;
detection.hit_point = Some(Vec3::new(2.0, 1.0, 0.0));

// User presses E to vault
parkour.state = ParkourState::Vaulting;

// IK system automatically:
// 1. Calculates hand positions on obstacle
left_hand_target = hit_point + Vec3::new(0.3, 0.05, 0.0);  // Spread right
right_hand_target = hit_point + Vec3::new(-0.3, 0.05, 0.0); // Spread left

// 2. IK solver moves arms to reach targets
// 3. Hands appear to grab the obstacle realistically
```

### Procedural Animation with Poses

```rust
// Load a vault animation from RON file
let vault_poses = asset_server.load("parkour_poses/standing_vault.ron");

commands.entity(player).insert(ActivePoseAnimation {
    animation: vault_poses,
    start_time: time.elapsed_secs(),
    looping: false,
});

// Pose interpolation system:
// - Reads keyframe poses from RON
// - Interpolates bone positions/rotations
// - Applies to skeleton every frame
// - Blends with IK corrections
```

## 🎨 Visualization Colors

When `IkConfig::debug_visualization = true`:

- **Cyan spheres** - Left hand IK targets
- **Magenta spheres** - Right hand IK targets
- **Green rays** - Obstacle detection (no hit)
- **Red rays** - Obstacle detection (hit)
- **Red spheres** - Obstacle hit points

## ⚙️ Configuration

### Adjust IK Behavior

```rust
// In your code or via inspector
app.insert_resource(IkConfig {
    enabled: true,
    hand_spread: 0.4,  // Wider hand placement
    hand_height_offset: 0.1,  // Higher above obstacle
    debug_visualization: true,
});
```

### Enable/Disable IK

```rust
// Toggle IK globally
ik_config.enabled = false;

// Or per-bone via constraint
left_hand_constraint.enabled = false;
```

## 🔧 Bone Names Reference

Critical bones tracked by the system:

### Arms (IK Chains)
- `mixamorig:LeftHand` - End effector
- `mixamorig:LeftForeArm` - Middle joint
- `mixamorig:LeftArm` - Root joint
- Mirror for right side

### Legs (IK Chains)
- `mixamorig:LeftFoot` - End effector
- `mixamorig:LeftLeg` - Middle joint
- `mixamorig:LeftUpLeg` - Root joint
- Mirror for right side

### Body (Pose Data)
- `mixamorig:Spine`, `Spine1`, `Spine2`
- `mixamorig:Hips`
- `mixamorig:Head`, `Neck`

## 🚀 Workflow Integration

### Full Pipeline

1. **Download Mixamo animations** → Add to GLTF as debug slots
2. **Press 1-9** → Play debug animation
3. **Press F12** → Extract bone poses to RON
4. **Organize RON files** → Create complete animations
5. **IK system** → Automatically adjusts for obstacle heights
6. **Result** → Adaptive parkour that works on any obstacle size

### When to Use IK vs Pure Poses

**Use IK for:**
- ✅ Hand placement on obstacles (always slightly different)
- ✅ Foot placement on uneven terrain
- ✅ Grabbing ledges at varying heights
- ✅ Wall run adjustments

**Use Poses for:**
- ✅ Body lean and spine bends
- ✅ Overall movement flow
- ✅ Timing and rhythm
- ✅ Secondary animations (head look, etc.)

**Best: Hybrid Approach**
- Base animation from poses
- IK corrections for end effectors (hands/feet)
- Procedural adjustments for obstacles

## 🐛 Troubleshooting

### IK targets not visible
- Check `IkConfig::debug_visualization = true`
- Ensure parkour action is active (Vaulting/Climbing)
- Verify obstacle was detected

### Arms not reaching targets
- Increase `IkConstraint::iterations` (try 30-50)
- Check `IkConstraint::enabled = true`
- Verify target position is reachable (not too far)

### Weird elbow/knee bending
- Set `pole_target` to guide joint direction
- Adjust pole_angle in IkConstraint
- May need to tune in Blender first

### Performance issues
- Reduce iterations (try 10-15)
- Disable IK when not needed
- Only enable for active parkour actions

## 📚 Code References

**IK Setup:** `src/game/parkour_ik.rs:78-179`
**Target Update:** `src/game/parkour_ik.rs:185-260`
**Visualization:** `src/game/parkour_ik.rs:290-345`
**Pose Interpolation:** `src/game/parkour_poses.rs:222-325`
**Bone Extraction:** `src/game/parkour_poses.rs:143-220`

## 🎓 Next Steps

After extracting poses:
1. Uncomment `apply_pose_animation` in `parkour_poses.rs:374`
2. Test procedural animations with Press P
3. Tune IK parameters for your character
4. Add easing curves (ease-in/ease-out)
5. Implement root motion if needed

---

**Ready to create adaptive parkour!** 🏃‍♂️💨
