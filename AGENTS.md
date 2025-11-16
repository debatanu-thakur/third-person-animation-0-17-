# Third-Person Parkour Animation System - Project Status

**Last Updated**: 2025-11-16
**Bevy Version**: 0.17
**Session**: Architecture Cleanup & AnimationClip Direct Access

---

## 🎯 ARCHITECTURE DECISIONS (2025-11-16)

### Decision 1: AnimationClip Direct Access

**Decision**: Use `AnimationClip` curves directly instead of runtime sampling

**Rationale**:
- Animation keyframe data already exists in GLB files
- Bevy's `AnimationClip` provides direct access to curves (rotation, translation, scale)
- Sampling just duplicates data that's already accessible
- Cleaner: Read curves → Calculate transforms → Apply IK
- No need for intermediate RON files or sampling systems

**Status**: ✅ Implemented (sampling code removed)

---

### Decision 2: Hybrid Movement System (Tnua + Custom Parkour)

**Decision**: Keep Bevy Tnua for basic locomotion, use custom movement for parkour actions

**The Problem**:
- Parkour animations (vault, climb, slide) use **root motion** - animation drives character position
- Tnua uses **physics-based movement** - physics calculations drive character position
- These two systems **fight each other** and cannot coexist during parkour actions
- Example: Vault animation moves character forward, Tnua's physics resets position → jittery/broken movement

**The Solution - Two Movement Systems**:

#### 1. Basic Locomotion (Tnua Enabled)
- **States**: Idle, Walk, Run, Jump
- **Tnua handles**: Physics, ground detection, slopes, stairs, collisions
- **IK adds**: Foot placement adjustment for terrain adaptation
- **Why keep it**: Solid physics system, handles complex scenarios (slopes, platforms) for free

#### 2. Parkour Actions (Tnua Disabled)
- **States**: Vaulting, Climbing, Sliding, Ledge Hang
- **Custom controller handles**:
  - Root motion extraction from animations
  - OR procedural path calculation (bezier curves from start → end)
  - IK for precise hand/foot placement on obstacles
  - Collision validation before executing action
- **Why custom**: Full control needed for animation-driven movement and precise positioning

**State Transition Flow**:
```
Normal Movement (Tnua Active)
  ↓
Detect Obstacle + Player Input
  ↓
Enter Parkour State → Disable Tnua
  ↓
Play Parkour Animation + Custom Movement + IK
  ↓
Parkour Complete → Re-enable Tnua
  ↓
Resume Normal Movement
```

**Implementation Phases**:
1. **Phase 1 (Current)**: Keep Tnua, add IK foot adjustment for basic locomotion
2. **Phase 2 (Next)**: Implement parkour state machine with Tnua enable/disable
3. **Phase 3**: Add custom parkour movement (root motion or procedural)
4. **Phase 4**: Layer IK on parkour for obstacle adaptation

**Benefits**:
- ✅ Best of both worlds: solid physics + precise parkour control
- ✅ No system conflicts (Tnua disabled during parkour)
- ✅ Simpler than full custom physics
- ✅ Can implement parkour incrementally

---

## 📋 Project Overview

Building an adaptive parkour system for Bevy 0.17 that combines:
- Real-time obstacle detection via raycasting
- Dynamic animation retargeting across GLTF files
- Inverse Kinematics for procedural hand/foot placement
- Direct AnimationClip curve access for adaptive movement

---

## ✅ COMPLETED

### 1. Animation Retargeting Solution ⭐ CRITICAL FIX
**Problem**: Animations from external GLTF files (vault.glb, climb.glb) wouldn't play on character even with matching bone names.

**Root Cause**: Bevy 0.17's `AnimationTargetId` is based on **entity hierarchy paths**, not just bone names.
- Character: `["brian", "mixamorig12:Hips"]` → UUID-A
- Animation: `["Armature", "mixamorig12:Hips"]` → UUID-B
- Different paths = Different UUIDs = No animation ❌

**Solution**: Rename root node in animation GLTFs to match character root node.

**Blender Workflow**:
1. Open `vault.glb` in Blender
2. Rename "Armature" → "brian" (to match character root)
3. Re-export as GLB
4. Animations now work! ✅

**Documentation**: `ANIMATION_RETARGETING.md`

### 2. Obstacle Detection System
**Status**: Fully implemented in `src/game/obstacle_detection/`

**Features**:
- Multi-ray raycasting (upper, center, lower) for different obstacle types
- Automatic classification: VaultableObstacle, ClimbableWall, SlideableObstacle
- Height-based action selection (< 1.0m = vault, 1.0-1.8m = climb)
- Debug visualization with colored gizmos
- Parkour state machine (Idle → Vaulting → Climbing → Hanging)

**Key Files**:
- `detection.rs` - Raycasting and obstacle classification
- `trigger.rs` - Action triggering based on detection
- `visualization.rs` - Debug gizmos

**Configuration**:
```rust
ObstacleDetectionConfig {
    detection_range: 2.0,
    center_ray_height: 1.0,
    upper_ray_height: 1.8,
    lower_ray_height: 0.3,
    debug_draw_rays: true,
}
```

### 3. Animation Loading System
**Status**: Working with GLTF loader pattern

**Implementation**: `src/game/parkour_animations/`
- `ParkourGltfAssets` - Loads GLTF files
- `ParkourAnimations` - Extracts animation clips from loaded GLTFs
- `ParkourAnimationLibrary` - Stores animation handles for playback
- Integrated into `AnimationGraph` system

**Animations**:
- ✅ All animations retargeted with root node "brian"
- ✅ `vault.glb`, `Freehang Climb.glb`, `Running Slide.glb`
- ✅ `Over Obstacle Jumping.glb`, `Falling To Roll.glb`
- ✅ `Running Jump.glb`, `Standard Run.glb`, `Standing Jumping.glb`
- ✅ Additional animations: Braced Hang, Jump To Freehang, etc.

### 4. Debug Systems
**Keyboard Controls**:
- `V` - Toggle vault animation (sets ParkourState::Vaulting)
- `O` - Dump bone hierarchy to RON files
- `P` - Print animation library info + write status to RON

**RON Debug Files** (in `assets/bones/`):
- `character_bones.ron` - Complete bone hierarchy (Press 'O')
- `vault_animation_bones.ron` - Animation curve data (Press 'O')

**Visual Debug**:
- Green rays: Obstacle detection (no hit)
- Red rays: Obstacle detection (hit)
- Red spheres: Obstacle hit points

---

## ⏳ IN PROGRESS

### 1. Code Cleanup (Current)
- [x] Document architecture decision
- [ ] Remove sampling system code
- [ ] Delete generated RON files
- [ ] Re-enable update_animation_state
- [ ] Verify basic animations (idle, walk, run)

### 2. AnimationClip Curve Reader
**Location**: `src/game/parkour_animations/` (to be refactored)

**Goal**: Direct access to animation keyframe data without sampling

**Approach**:
- Access `AnimationClip` from `Assets<AnimationClip>`
- Read `VariableCurve` for specific bones at specific times
- Calculate world-space transforms using skeleton hierarchy
- Use for IK target positioning

---

## 📋 TO DO

### Phase 1: Basic Locomotion (1-2 hours)
- [ ] Ensure idle, walk, run animations work correctly
- [ ] Verify animation transitions are smooth
- [ ] Test character controller integration

### Phase 2: AnimationClip Curve Utilities (2-3 hours)
- [ ] Create utility to read bone transform from AnimationClip at time t
- [ ] Add helper to convert local → world space using hierarchy
- [ ] Test reading hand positions from vault animation
- [ ] Verify data matches what we saw in sampled_vault.ron

### Phase 3: IK Integration with AnimationClip (3-4 hours)
- [ ] Read hand positions from vault animation curve
- [ ] Calculate IK targets based on obstacle position + animation data
- [ ] Implement progress tracking (0.0 to 1.0) for parkour actions
- [ ] Blend animation curve data with obstacle height adjustment
- [ ] Add pole targets for natural arm bending
- [ ] Test with obstacles at different heights

### Phase 4: Parkour Animation System (2-3 hours)
- [ ] Connect parkour state to animation playback
- [ ] Synchronize IK with animation timeline
- [ ] Add foot IK for climbing/landing
- [ ] Test full parkour flow (detect → animate → IK)

---

## 🏗️ System Architecture

### Data Flow
```
Player Movement
    ↓
Obstacle Detection (raycasts)
    ↓
Classification (vault/climb/slide)
    ↓
State Transition (ParkourController)
    ↓
Animation Trigger (AnimationGraph)
    ↓
Sample Animation Poses (seek_to + read bones)
    ↓
Calculate IK Targets (sampled + obstacle adjusted)
    ↓
IK Solver (bevy_mod_inverse_kinematics)
    ↓
Final Character Pose
```

### File Structure
```
src/game/
├── obstacle_detection/
│   ├── detection.rs         ✅ Raycasting, classification
│   ├── trigger.rs           ✅ Action triggering
│   └── visualization.rs     ✅ Debug gizmos
│
├── parkour_animations/
│   ├── mod.rs              ✅ Animation loading
│   ├── assets.rs           ✅ GLTF loading pattern
│   └── retarget.rs         ✅ Retargeting notes (WIP)
│
├── parkour_ik/
│   └── mod.rs              ⏳ IK setup (needs sampling integration)
│
└── animations/
    └── animation_controller.rs  ✅ Main animation graph

assets/models/animations/
├── vault.glb               ✅ Working (root = "brian")
├── Freehang Climb.glb      ⏳ Needs root rename
├── Running Slide.glb       ⏳ Needs root rename
└── ...                     ⏳ Others pending
```

---

## 🔑 Key Technical Insights

### Bevy 0.17 Animation System
**AnimationTargetId is Path-Based**:
- NOT just bone names
- Full entity hierarchy path matters
- `["brian", "mixamorig12:Hips"]` != `["Armature", "mixamorig12:Hips"]`
- Solution: Match root node names exactly

**Animation Loading Pattern**:
```rust
// Load GLTF → Extract animations → Add to graph
let gltf_handle = asset_server.load("vault.glb");
let gltf = gltf_assets.get(&gltf_handle)?;
let animation_clip = gltf.animations.first()?;
```

**Can't Sample Curves Directly**:
- `VariableCurve` is internal API
- Must play animation and read bone transforms
- Use `player.seek_to(time)` + read `GlobalTransform`

### IK System (bevy_mod_inverse_kinematics)
**IkConstraint Setup**:
```rust
commands.entity(hand_bone).insert(IkConstraint {
    chain_length: 2,        // hand → forearm → arm
    iterations: 20,         // solver iterations
    target: target_entity,  // where to reach
    pole_target: Some(forearm_bone),  // controls bend direction
    enabled: true,
});
```

### Obstacle Detection (Avian3D)
**SpatialQuery API (0.4.x)**:
```rust
let hit = spatial_query.cast_ray(
    origin,
    direction,  // Dir3
    max_distance,
    true,  // solid
    &SpatialQueryFilter::default(),  // ⚠️ Pass by reference!
);

// Access hit data
let hit_point = origin + *direction * hit.distance;  // ⚠️ .distance not .time_of_impact
```

---

## 📚 Documentation Files

- `AGENTS.md` ← You are here (Project status & Bevy 0.17 API)
- `ANIMATION_RETARGETING.md` - How to fix bone path mismatches
- `PARKOUR_SYSTEM_SUMMARY.md` - Detailed implementation summary
- `PARKOUR_IK_SYSTEM.md` - IK system architecture
- `ANIMATION_SAMPLING_STRATEGY.md` - Runtime sampling approach
- `assets/parkour_animations/README.md` - Animation file guide (outdated paths)

---

## 🎯 Current Priority

**Next Immediate Step**: Fix remaining animations in Blender (30 minutes)

Then proceed to: Animation sampling system → IK integration → Polish

---

## 🔧 Quick Reference Commands

**Test Vault Animation**:
```bash
cargo run
# In game: Press 'V' to trigger vault animation
```

**Debug with RON Files** (no log reading needed!):
```bash
# In game controls:
# Press 'O' → Generates character_bones.ron + vault_animation_bones.ron
# Press 'P' → Generates animation_library_status.ron
# Auto → sampled_vault.ron generated when sampling completes

# Check RON files:
ls assets/bones/
cat assets/bones/character_bones.ron          # Bone hierarchy
cat assets/bones/vault_animation_bones.ron    # Animation curves
cat assets/bones/sampled_vault.ron            # Sampled transforms
cat assets/bones/animation_library_status.ron # Current status
```

**Verify Sampling Worked**:
```bash
# Should see 5 keyframes with bone data
cat assets/bones/sampled_vault.ron | grep "time:"
# Expected: time: 0.0, time: 0.25, time: 0.5, time: 0.75, time: 1.0
```

---

## 🐛 Known Issues & Solutions

### Issue: Animation doesn't play
**Symptom**: Character freezes, no movement
**Cause**: Root node name mismatch (Armature vs brian)
**Fix**: Rename root in Blender, re-export

### Issue: AnimationPlayer shows paused: true
**Symptom**: Animation loaded but not playing
**Cause**: AnimationTransitions overrides manual plays
**Fix**: Set ParkourState instead of calling player.play() directly

### Issue: Bone names match but animation still broken
**Symptom**: 65/65 bones match, still no animation
**Cause**: Entity hierarchy paths don't match
**Fix**: Check root node name (must be exactly "brian")

---

## 🎓 Bevy 0.17 API Summary

### Animation System
- `AnimationGraph` + `AnimationPlayer` + `AnimationTransitions`
- Use `SceneInstanceReady` observer for scene loading
- `AnimationTargetId` is **path-based**, not name-based
- Load via `GltfAssetLabel::Animation(0).from_asset()`

### Raycasting (Avian3D 0.4.x)
- `SpatialQuery` system parameter
- `RayHitData.distance` (not .time_of_impact)
- Pass `SpatialQueryFilter` by reference: `&SpatialQueryFilter::default()`
- Use `Dir3` for typed direction vectors

### Component Bundles
- Separate components instead of bundles
- `Mesh3d` + `MeshMaterial3d` + `Transform` (not `PbrBundle`)
- `Color::srgb()` instead of `Color::rgb()`

### Query Patterns
- `Single<T>` for single-entity queries (enforced at compile time)
- `children.iter_descendants()` for hierarchy traversal
- `.observe()` for event-based systems

---

**Ready to build adaptive parkour!** 🏃‍♂️💨

### 1. **Component Bundle Simplification**

**Old (0.14/0.15)**:
```rust
commands.spawn(PbrBundle {
    mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
    material: materials.add(Color::rgb(0.8, 0.7, 0.6)),
    transform: Transform::from_xyz(0.0, 0.5, 0.0),
    ..default()
});
```

**New (0.17)** - Components are separate:
```rust
commands.spawn((
    Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
    MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
    Transform::from_xyz(0.0, 0.5, 0.0),
));
```

**Key Changes**:
- `PbrBundle` → Separate `Mesh3d` + `MeshMaterial3d` + `Transform`
- `Color::rgb()` → `Color::srgb()` (proper color space)
- More explicit, less "magic bundles"

---

## 🔫 Raycasting

### A. **Mesh-Based Raycasting** (for visual meshes, not physics)

**Example**: `examples/3d/mesh_ray_cast.rs`

```rust
use bevy::prelude::*;

fn bouncing_raycast(
    mut ray_cast: MeshRayCast,  // System parameter
    mut gizmos: Gizmos,
) {
    let ray = Ray3d::new(ray_pos, ray_dir);

    // Cast ray and get hits
    let hits = ray_cast.cast_ray(ray, &MeshRayCastSettings::default());

    if let Some((entity, hit)) = hits.first() {
        // hit.point - Vec3 position
        // hit.normal - Vec3 normal
        gizmos.sphere(hit.point, 0.1, Color::RED);
    }
}
```

**When to use**: UI picking, visual mesh interaction (not physics)

---

### B. **Physics-Based Raycasting** (Avian3D `SpatialQuery`)

**Example**: Used in our obstacle detection

```rust
use avian3d::prelude::*;

fn detect_obstacles(
    spatial_query: SpatialQuery,  // System parameter from Avian3D
    mut gizmos: Gizmos,
) {
    let origin = Vec3::new(0.0, 1.0, 0.0);
    let direction = Dir3::NEG_Z;  // Dir3 direction
    let max_distance = 5.0;

    // Cast a single ray
    let hit = spatial_query.cast_ray(
        origin,
        direction,
        max_distance,
        true,  // solid (hit triggers/sensors)
        &SpatialQueryFilter::default(),  // ⚠️ Pass by reference!
    );

    if let Some(ray_hit_data) = hit {
        // ray_hit_data.distance - f32 distance (⚠️ renamed from time_of_impact in 0.4.x)
        // ray_hit_data.entity - Entity hit
        // ray_hit_data.normal - Vec3 surface normal
        let hit_point = origin + *direction * ray_hit_data.distance;
        gizmos.sphere(Isometry3d::from_translation(hit_point), 0.1, Color::GREEN);
    }
}
```

**Key Points (Avian3D 0.4.x)**:
- `SpatialQuery` is a system parameter from `avian3d::prelude::*`
- Returns `Option<RayHitData>`
- **⚠️ API Changes in 0.4.x**:
  - `RayHitData.time_of_impact` → `RayHitData.distance`
  - `SpatialQueryFilter` must be passed by reference: `&SpatialQueryFilter::default()`
- `RayHitData` fields: `distance`, `entity`, `normal`
- Use `Dir3` for directions (typed direction vector)
- Convert `Dir3` to `Vec3` with `*direction` for math operations
- Filter allows excluding specific entities/layers

---

### C. **Tnua's Spatial Abstraction** (for character controllers)

**Example**: `examples/bevy_tnua_demo/src/character_control_systems/spatial_ext_facade.rs`

```rust
use bevy_tnua::spatial_ext::TnuaSpatialExt;
use bevy_tnua_avian3d::TnuaSpatialExtAvian3d;

// Tnua provides physics-backend-agnostic raycasting
fn tnua_raycast(spatial_ext: TnuaSpatialExtAvian3d) {
    let collider_data = spatial_ext.fetch_collider_data(entity)?;

    let result = spatial_ext.cast_ray(
        origin,
        direction,
        max_toi,
        &collider_data,
    );

    if let Some((distance, normal)) = result {
        // Use distance and normal
    }
}
```

**When to use**: Inside Tnua-based character controllers for consistency

---

## 🎬 Animation System

### **AnimationGraph + AnimationPlayer**

**Example**: `examples/animation/animated_mesh.rs`

```rust
use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;

#[derive(Component)]
struct AnimationToPlay {
    graph_handle: Handle<AnimationGraph>,
    index: AnimationNodeIndex,
}

fn setup_animation(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    // Create animation graph from a clip
    let (graph, index) = AnimationGraph::from_clip(
        asset_server.load(GltfAssetLabel::Animation(2).from_asset("model.glb"))
    );

    let graph_handle = graphs.add(graph);

    // Spawn scene with animation data
    commands
        .spawn((
            AnimationToPlay { graph_handle, index },
            SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("model.glb"))),
        ))
        .observe(play_animation_when_ready);  // 👈 Observer pattern!
}

// Triggered when scene is ready
fn play_animation_when_ready(
    scene_ready: On<SceneInstanceReady>,  // 👈 Event trigger
    animations_to_play: Query<&AnimationToPlay>,
    mut players: Query<&mut AnimationPlayer>,
    children: Query<&Children>,
) {
    if let Ok(animation_to_play) = animations_to_play.get(scene_ready.entity) {
        // Find AnimationPlayer in scene hierarchy
        for child in children.iter_descendants(scene_ready.entity) {
            if let Ok(mut player) = players.get_mut(child) {
                player.play(animation_to_play.index).repeat();

                // Connect graph to player
                commands.entity(child)
                    .insert(AnimationGraphHandle(animation_to_play.graph_handle.clone()));
            }
        }
    }
}
```

**Key Patterns**:
- `AnimationGraph` - Container for animation clips
- `AnimationPlayer` - Plays animations (spawned automatically with GLTF)
- `SceneInstanceReady` - Event when scene is loaded
- `.observe()` - New observer pattern for event handling
- `children.iter_descendants()` - Traverse hierarchy

---

## 🦾 Inverse Kinematics (IK)

**Example**: `examples/inverse_kinematics/skin_mesh.rs`

```rust
use bevy_mod_inverse_kinematics::*;

fn main() {
    App::new()
        .add_plugins(InverseKinematicsPlugin)  // 👈 Add IK plugin
        .add_systems(Update, setup_ik)
        .run();
}

fn setup_ik(
    mut commands: Commands,
    added_query: Query<Entity, (Added<AnimationPlayer>, With<ChildOf>)>,
    children: Query<&Children>,
    names: Query<&Name>,
) {
    for entity in added_query.iter() {
        // Find bone by traversing hierarchy
        let right_hand = find_entity(
            &vec!["Pelvis".into(), "Spine1".into(), "Hand.R".into()],
            entity,
            &children,
            &names,
        ).unwrap();

        // Create IK target (red sphere)
        let target = commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.05))),
            MeshMaterial3d(materials.add(Color::from(css::RED))),
            Transform::from_xyz(0.0, 1.0, 0.5),
        )).id();

        // Create pole target (controls elbow/knee direction)
        let pole_target = commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.05))),
            MeshMaterial3d(materials.add(Color::from(css::LIME))),
            Transform::from_xyz(-1.0, 0.4, -0.2),
        )).id();

        // Add IK constraint to bone
        commands.entity(right_hand).insert(IkConstraint {
            chain_length: 2,  // 2 bones (forearm + upper arm)
            iterations: 20,   // Solver iterations
            target,           // Target entity to reach
            pole_target: Some(pole_target),  // Controls bend direction
            pole_angle: -std::f32::consts::FRAC_PI_2,
            enabled: true,
        });
    }
}

// Helper to find bone by name path
fn find_entity(
    path: &Vec<Name>,
    root: Entity,
    children: &Query<&Children>,
    names: &Query<&Name>,
) -> Result<Entity, ()> {
    let mut current_entity = root;

    for part in path.iter() {
        let mut found = false;
        if let Ok(children) = children.get(current_entity) {
            for child in children.iter() {
                if let Ok(name) = names.get(child) {
                    if name == part {
                        current_entity = child;
                        found = true;
                        break;
                    }
                }
            }
        }
        if !found {
            return Err(());
        }
    }
    Ok(current_entity)
}
```

**Key Concepts**:
- `IkConstraint` - Component added to end bone (hand, foot)
- `chain_length` - Number of bones in IK chain
- `target` - Entity to reach toward
- `pole_target` - Controls which way elbow/knee bends
- `iterations` - Higher = more accurate but slower

**For Parkour**:
```rust
// During vault animation
let vault_top = Vec3::new(0.0, 1.5, 2.0);  // From raycast hit

let left_hand_target = commands.spawn((
    Transform::from_translation(vault_top + Vec3::new(-0.3, 0.0, 0.0)),
)).id();

let right_hand_target = commands.spawn((
    Transform::from_translation(vault_top + Vec3::new(0.3, 0.0, 0.0)),
)).id();

commands.entity(left_hand_bone).insert(IkConstraint {
    chain_length: 2,
    target: left_hand_target,
    // ... other fields
});
```

---

## 🎮 Tnua Character Controller

**Example**: `examples/bevy_tnua_simple_examples/example.rs`

```rust
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            TnuaControllerPlugin::new(FixedUpdate),  // 👈 Required
            TnuaAvian3dPlugin::new(FixedUpdate),     // 👈 Physics backend
        ))
        .add_systems(FixedUpdate, apply_controls.in_set(TnuaUserControlsSystems))
        .run();
}

fn setup_player(mut commands: Commands) {
    commands.spawn((
        RigidBody::Dynamic,
        Collider::capsule(0.5, 1.0),
        TnuaController::default(),  // 👈 Main controller
        TnuaAvian3dSensorShape(Collider::cylinder(0.49, 0.0)),  // 👈 Ground sensor
        LockedAxes::ROTATION_LOCKED,  // Prevent tipping
    ));
}

fn apply_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut TnuaController>,
) {
    let Ok(mut controller) = query.single_mut() else { return };

    let mut direction = Vec3::ZERO;
    if keyboard.pressed(KeyCode::ArrowUp) {
        direction -= Vec3::Z;
    }

    // ALWAYS feed basis every frame (even if not moving)
    controller.basis(TnuaBuiltinWalk {
        desired_velocity: direction.normalize_or_zero() * 10.0,
        float_height: 1.5,  // Must be > (capsule_height / 2)
        turning_angvel: 12.0,
        desired_forward: Dir3::new(direction.normalize_or_zero()).ok(),
        ..Default::default()
    });

    // Feed jump when key pressed
    if keyboard.pressed(KeyCode::Space) {
        controller.action(TnuaBuiltinJump {
            height: 4.0,
            input_buffer_time: 0.5,  // Coyote time
            ..Default::default()
        });
    }
}
```

**Key Points**:
- `TnuaController` - Must receive basis EVERY frame
- `TnuaAvian3dSensorShape` - Detects ground (slightly smaller than main collider)
- `float_height` - Character hovers above ground
- `input_buffer_time` - Jump buffering (press jump slightly before landing)

---

## 🔍 Query Patterns

### **Single<T>** - For queries that should return exactly one result

**Old**:
```rust
fn system(query: Query<&Transform, With<Player>>) {
    let transform = query.single();  // Can panic
}
```

**New**:
```rust
fn system(player: Single<&Transform, With<Player>>) {
    let transform = *player;  // Type-safe, enforced at query time
}
```

**Multiple components**:
```rust
fn system(camera: Single<(&Camera, &GlobalTransform)>) {
    let (camera, transform) = camera.into_inner();
}
```

---

## 🎨 Gizmos (Debug Visualization)

**Example**: Used throughout examples

```rust
fn debug_system(mut gizmos: Gizmos) {
    // Line
    gizmos.line(Vec3::ZERO, Vec3::Y, Color::srgb(1.0, 0.0, 0.0));

    // Sphere
    gizmos.sphere(Vec3::new(1.0, 1.0, 1.0), 0.5, Color::GREEN);

    // Circle with transform
    gizmos.circle(
        Isometry3d::new(
            Vec3::ZERO,
            Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
        ),
        1.0,
        Color::BLUE,
    );

    // Gradient linestrip
    gizmos.linestrip_gradient(vec![
        (Vec3::ZERO, Color::RED),
        (Vec3::X, Color::GREEN),
        (Vec3::Y, Color::BLUE),
    ]);
}
```

**For Raycasting Debug**:
```rust
// Ray line
gizmos.line(ray_origin, ray_origin + ray_dir * max_distance, Color::YELLOW);

// Hit point
if let Some(hit) = raycast_hit {
    gizmos.sphere(hit_point, 0.1, Color::RED);
}
```

---

## 📦 Scene Loading

**Example**: `examples/animation/animated_mesh.rs`

```rust
use bevy::scene::SceneInstanceReady;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        SceneRoot(asset_server.load(
            GltfAssetLabel::Scene(0).from_asset("model.glb")
        )),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ))
    .observe(on_scene_ready);  // 👈 Observer
}

fn on_scene_ready(
    trigger: On<SceneInstanceReady>,
    // ... queries
) {
    let entity = trigger.entity;  // The entity that triggered
    // Do stuff when scene loads
}
```

**GLTF Asset Labels**:
```rust
// Scene #0 from GLTF
GltfAssetLabel::Scene(0).from_asset("model.glb")

// Animation #2 from GLTF
GltfAssetLabel::Animation(2).from_asset("model.glb")

// Mesh from GLTF
GltfAssetLabel::Mesh(0).from_asset("model.glb")
```

---

## 🧩 Camera & Viewport

**Example**: `examples/3d/3d_viewport_to_world.rs`

```rust
fn system(
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
) {
    let (camera, camera_transform) = *camera;

    // Get cursor ray
    if let Some(cursor_pos) = window.cursor_position() {
        if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
            // ray: Ray3d with origin and direction

            // Intersect with plane
            let plane_normal = Vec3::Y;
            let plane_origin = Vec3::ZERO;

            if let Some(distance) = ray.intersect_plane(
                plane_origin,
                InfinitePlane3d::new(plane_normal)
            ) {
                let hit_point = ray.get_point(distance);
            }
        }
    }
}
```

---

## 🚀 Recommended API for Obstacle Detection

Based on examples, here's the recommended approach:

### **Use Avian3D's `SpatialQuery`**

```rust
use avian3d::prelude::*;
use bevy::prelude::*;

fn detect_obstacles(
    player: Single<&Transform, With<Player>>,
    spatial_query: SpatialQuery,
    mut gizmos: Gizmos,
) {
    let player_pos = player.translation;
    let forward = player.forward();

    // Multi-ray setup
    let rays = [
        (player_pos + Vec3::Y * 1.8, Color::BLUE),   // Upper
        (player_pos + Vec3::Y * 1.0, Color::YELLOW), // Center
        (player_pos + Vec3::Y * 0.3, Color::GREEN),  // Lower
    ];

    for (origin, color) in rays {
        let direction = Dir3::new(forward).unwrap();
        let max_dist = 2.0;

        // Debug ray
        gizmos.line(origin, origin + *direction * max_dist, color);

        // Cast ray
        if let Some(hit) = spatial_query.cast_ray(
            origin,
            direction,
            max_dist,
            true,
            SpatialQueryFilter::default(),
        ) {
            let hit_point = origin + *direction * hit.time_of_impact;
            gizmos.sphere(hit_point, 0.1, Color::RED);

            // Use hit.entity to query obstacle components
        }
    }
}
```

---

## 📋 Summary Checklist

### ✅ **Updated Patterns**
- [x] Use `Mesh3d` + `MeshMaterial3d` instead of `PbrBundle`
- [x] Use `Color::srgb()` instead of `Color::rgb()`
- [x] Use `Single<T>` for single-entity queries
- [x] Use `SpatialQuery` from `avian3d::prelude::*`
- [x] Use `Dir3` for typed direction vectors
- [x] Use `.observe()` for scene loading events
- [x] Use `AnimationGraph` + `AnimationPlayer` for animations
- [x] Use `IkConstraint` for target matching
- [x] Use `Gizmos` for debug visualization

### 🔄 **Code to Update in `obstacle_detection.rs`**

1. **Ray direction**: Already using `Dir3::new()` ✅
2. **Spatial query**: Already using `SpatialQuery` ✅
3. **Gizmos**: Need to update sphere API (add `Isometry3d`)
4. **Hit data access**: Already using `time_of_impact` ✅

### 🆕 **New Additions Needed**

1. **IK System**: Add `bevy_mod_inverse_kinematics` integration
2. **Animation Graph**: Setup for parkour animations
3. **Observer Pattern**: Use for animation state changes
4. **Sensor Shapes**: Add Avian3D sensor shapes for proximity detection

---

## 🚨 Avian3D 0.4.x Migration Notes

### **Breaking Changes**

1. **`RayHitData.time_of_impact` → `RayHitData.distance`**
   ```rust
   // ❌ Old (0.3.x)
   let hit_point = origin + direction * hit.time_of_impact;

   // ✅ New (0.4.x)
   let hit_point = origin + direction * hit.distance;
   ```

2. **`SpatialQueryFilter` must be passed by reference**
   ```rust
   // ❌ Old (0.3.x)
   spatial_query.cast_ray(origin, dir, max_dist, true, SpatialQueryFilter::default())

   // ✅ New (0.4.x)
   spatial_query.cast_ray(origin, dir, max_dist, true, &SpatialQueryFilter::default())
   ```

3. **`Dir3` handling**
   - `Transform::forward()` returns `Dir3`
   - Dereference with `*direction` to convert to `Vec3` for math operations
   ```rust
   let forward = transform.forward();  // Returns Dir3
   let forward_vec = *forward;  // Convert to Vec3
   let result = origin + forward_vec * distance;  // Vec3 math
   ```

---

## 🎯 Next Steps

1. ✅ **Update `obstacle_detection.rs`** - Fixed Avian3D 0.4.x API
2. ✅ **Fix gizmo sphere API** - Updated to use `Isometry3d`
3. **Add IK setup system** - Create hand/foot targets
4. **Integrate AnimationGraph** - Load and play parkour animations
5. **Test raycasting** - Verify detection works with debug gizmos
6. **Add sensors** - Proximity triggers for automatic actions

This summary should serve as a reference for migrating code to Bevy 0.17 and Avian3D 0.4.x patterns! 🚀
