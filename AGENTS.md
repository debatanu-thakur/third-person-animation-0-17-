# Bevy 0.17 API Summary for Third-Person Animation Project

This document summarizes key API changes and patterns in Bevy 0.17 based on the examples, specifically for implementing obstacle detection, raycasting, IK, and animations.

---

## 🎯 Core API Changes

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
