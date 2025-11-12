use crate::screens::Screen;
use avian3d::prelude::*;
use bevy::prelude::*;
/// Marker component for animation test scene entities
#[derive(Component)]
pub struct AnimationTestSceneEntity;

/// Spawn the animation test scene with various obstacles
pub fn spawn_animation_test_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    info!("Spawning animation test scene...");

    // Floor material
    let floor_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.5, 0.3),
        perceptual_roughness: 0.9,
        ..default()
    });

    // Obstacle material - different colors for different types
    let wall_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.7, 0.3, 0.3),
        ..default()
    });

    let platform_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.3, 0.7),
        ..default()
    });

    let ramp_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.7, 0.7, 0.3),
        ..default()
    });

    // Ground floor - 50m x 50m
    let floor_size = 50.0;
    let floor_height = 0.5;
    spawn_obstacle(
        &mut commands,
        &mut meshes,
        floor_material.clone(),
        Vec3::new(0.0, 0., 0.0),
        Vec3::new(floor_size, floor_height, floor_size),
        Vec3::new(floor_size, floor_height / 2., floor_size),
        "Ground Floor",
    );

    // Player height reference: 2m tall
    let player_height = 2.0;

    // ===== WALLS AT DIFFERENT HEIGHTS =====
    let wall_thickness = 0.5;
    let wall_length = 5.0;

    // Low wall (waist-high) - 1m
    spawn_obstacle(
        &mut commands,
        &mut meshes,
        wall_material.clone(),
        Vec3::new(-10.0, 0.5, 0.0),
        Vec3::new(wall_length, 1.0, wall_thickness),
        Vec3::new(wall_length, 1.0, wall_thickness),
        "Low Wall (1m)",
    );

    // Mid wall (shoulder-high) - 1.5m
    spawn_obstacle(
        &mut commands,
        &mut meshes,
        wall_material.clone(),
        Vec3::new(-10.0, 0.75, 5.0),
        Vec3::new(wall_length, 1.5, wall_thickness),
        Vec3::new(wall_length, 1.5, wall_thickness),
        "Mid Wall (1.5m)",
    );

    // Tall wall (head-height) - 2m
    spawn_obstacle(
        &mut commands,
        &mut meshes,
        wall_material.clone(),
        Vec3::new(-10.0, 1.0, 10.0),
        Vec3::new(wall_length, 2.0, wall_thickness),
        Vec3::new(wall_length, 2.0, wall_thickness),
        "Tall Wall (2m)",
    );

    // Very tall wall (climbing) - 3m
    spawn_obstacle(
        &mut commands,
        &mut meshes,
        wall_material.clone(),
        Vec3::new(-10.0, 1.5, 15.0),
        Vec3::new(wall_length, 3.0, wall_thickness),
        Vec3::new(wall_length, 3.0, wall_thickness),
        "Climb Wall (3m)",
    );

    // ===== PLATFORMS AT DIFFERENT HEIGHTS =====
    let platform_size = 3.0;

    // Low platform - 0.5m
    spawn_obstacle(
        &mut commands,
        &mut meshes,
        platform_material.clone(),
        Vec3::new(5.0, 0.5, 0.0),
        Vec3::new(platform_size, 0.5, platform_size),
        Vec3::new(platform_size, 0.5, platform_size),
        "Platform (0.5m)",
    );

    // Mid platform - 1m
    spawn_obstacle(
        &mut commands,
        &mut meshes,
        platform_material.clone(),
        Vec3::new(10.0, 1.0, 0.0),
        Vec3::new(platform_size, 0.5, platform_size),
        Vec3::new(platform_size, 0.5, platform_size),
        "Platform (1m)",
    );

    // High platform - 1.5m
    spawn_obstacle(
        &mut commands,
        &mut meshes,
        platform_material.clone(),
        Vec3::new(15.0, 1.5, 0.0),
        Vec3::new(platform_size, 0.5, platform_size),
        Vec3::new(platform_size, 0.5, platform_size),
        "Platform (1.5m)",
    );

    // Very high platform - 2m
    spawn_obstacle(
        &mut commands,
        &mut meshes,
        platform_material.clone(),
        Vec3::new(20.0, 2.0, 0.0),
        Vec3::new(platform_size, 0.5, platform_size),
        Vec3::new(platform_size, 0.5, platform_size),
        "Platform (2m)",
    );

    // ===== GAPS FOR JUMPING =====
    // Create a series of platforms with gaps between them
    let gap_platform_height = 1.0;
    let gap_platform_size = 2.0;

    // Gap 1m
    spawn_obstacle(
        &mut commands,
        &mut meshes,
        platform_material.clone(),
        Vec3::new(0.0, gap_platform_height, -10.0),
        Vec3::new(gap_platform_size, 0.5, gap_platform_size),
        Vec3::new(gap_platform_size, 0.5, gap_platform_size),
        "Gap Start",
    );
    spawn_obstacle(
        &mut commands,
        &mut meshes,
        platform_material.clone(),
        Vec3::new(0.0 + gap_platform_size + 1.0, gap_platform_height, -10.0),
        Vec3::new(gap_platform_size, 0.5, gap_platform_size),
        Vec3::new(gap_platform_size, 0.5, gap_platform_size),
        "Gap 1m",
    );

    // Gap 2m
    spawn_obstacle(
        &mut commands,
        &mut meshes,
        platform_material.clone(),
        Vec3::new(
            0.0 + gap_platform_size * 2.0 + 3.0,
            gap_platform_height,
            -10.0,
        ),
        Vec3::new(gap_platform_size, 0.5, gap_platform_size),
        Vec3::new(gap_platform_size, 0.5, gap_platform_size),
        "Gap 2m",
    );

    // Gap 3m
    spawn_obstacle(
        &mut commands,
        &mut meshes,
        platform_material.clone(),
        Vec3::new(
            0.0 + gap_platform_size * 3.0 + 6.0,
            gap_platform_height,
            -10.0,
        ),
        Vec3::new(gap_platform_size, 0.5, gap_platform_size),
        Vec3::new(gap_platform_size, 0.5, gap_platform_size),
        "Gap 3m",
    );

    // Gap 4m
    spawn_obstacle(
        &mut commands,
        &mut meshes,
        platform_material.clone(),
        Vec3::new(
            0.0 + gap_platform_size * 4.0 + 10.0,
            gap_platform_height,
            -10.0,
        ),
        Vec3::new(gap_platform_size, 0.5, gap_platform_size),
        Vec3::new(gap_platform_size, 0.5, gap_platform_size),
        "Gap 4m",
    );

    // ===== RAMPS/SLOPES =====
    // Small ramp (15 degrees)
    spawn_ramp(
        &mut commands,
        &mut meshes,
        ramp_material.clone(),
        Vec3::new(-5.0, 0.0, -10.0),
        Vec3::new(5.0, 0.5, 3.0),
        15.0,
        "Ramp (15°)",
    );

    // Medium ramp (30 degrees)
    spawn_ramp(
        &mut commands,
        &mut meshes,
        ramp_material.clone(),
        Vec3::new(-5.0, 0.0, -15.0),
        Vec3::new(5.0, 0.5, 3.0),
        30.0,
        "Ramp (30°)",
    );

    // Steep ramp (45 degrees)
    spawn_ramp(
        &mut commands,
        &mut meshes,
        ramp_material.clone(),
        Vec3::new(-5.0, 0.0, -20.0),
        Vec3::new(5.0, 0.5, 3.0),
        45.0,
        "Ramp (45°)",
    );

    // ===== LIGHTING =====
    // Directional light
    commands.spawn((
        AnimationTestSceneEntity,
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.0, -0.5, 0.0)),
    ));

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 200.0,
        affects_lightmapped_meshes: false,
    });

    info!("Animation test scene spawned successfully!");
}

/// Helper function to spawn an obstacle with physics
fn spawn_obstacle(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    material: Handle<StandardMaterial>,
    position: Vec3,
    size: Vec3,
    collider_size: Vec3,
    label: &str,
) {
    let mesh = Mesh::from(Cuboid::new(size.x, size.y, size.z));

    commands.spawn((
        DespawnOnExit(Screen::Gameplay),
        AnimationTestSceneEntity,
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(material),
        Transform::from_translation(position),
        RigidBody::Static,
        // Collider::cuboid takes half-extents
        Collider::cuboid(collider_size.x, collider_size.y, collider_size.z),
        Name::new(label.to_string()),
    ));

    // TODO: Add text label above the obstacle (optional, requires text rendering setup)
    info!("Spawned obstacle: {} at {}", label, position);
}

/// Helper function to spawn a ramp/slope
fn spawn_ramp(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    material: Handle<StandardMaterial>,
    position: Vec3,
    size: Vec3,
    angle_degrees: f32,
    label: &str,
) {
    let mesh = Mesh::from(Cuboid::new(size.x, size.y, size.z));
    let angle_radians = angle_degrees.to_radians();

    commands.spawn((
        DespawnOnExit(Screen::Gameplay),
        AnimationTestSceneEntity,
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(material),
        Transform::from_translation(position).with_rotation(Quat::from_rotation_z(angle_radians)),
        RigidBody::Static,
        // Collider::cuboid takes half-extents
        Collider::cuboid(size.x / 2.0, size.y / 2.0, size.z / 2.0),
        Name::new(label.to_string()),
    ));

    info!(
        "Spawned ramp: {} at {} with angle {}°",
        label, position, angle_degrees
    );
    let animation_files = vec![
        "animation_models/Jump To Hang.glb",
        "animation_models/Freehang Climb.glb",
        "animation_models/Standard Run.glb",
        "animation_models/Jump To Freehang.glb",
        "animation_models/Running Slide.glb",
        "animation_models/Over Obstacle Jumping.glb",
        "animation_models/Braced Hang To Crouch.glb",
        "animation_models/Braced Hang Drop.glb",
        "animation_models/Breathing Idle.glb",
        "animation_models/Standing Jumping.glb",
        "animation_models/Braced Hang.glb",
        "animation_models/Hard Landing.glb",
        "animation_models/Free Hang To Braced.glb",
        "animation_models/Falling To Roll.glb",
        "animation_models/Stand To Freehang.glb",
    ];
}
