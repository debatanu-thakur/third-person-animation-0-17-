use crate::{
    game::{
        player::Player,
        third_person_camera::{
            ThirdPersonCamera, ThirdPersonCameraPlugin,
            third_person_plugin::{Offset, Zoom},
        },
    },
    screens::Screen,
};
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(ThirdPersonCameraPlugin);
    app.add_systems(Update, attach_camera_to_player);

    // Cursor lock management based on screen state
    app.add_systems(OnEnter(Screen::Gameplay), enable_cursor_lock);
    app.add_systems(OnExit(Screen::Gameplay), disable_cursor_lock);
}

/// Attach third-person camera component to the main camera when player exists
fn attach_camera_to_player(
    mut commands: Commands,
    player_query: Query<Entity, With<Player>>,
    camera_query: Query<Entity, (With<Camera3d>, Without<ThirdPersonCamera>)>,
) {
    // Only attach if player exists and camera doesn't have third-person component yet
    if player_query.iter().count() > 0 {
        let fov: f32 = 60.0;
        if let Ok(camera_entity) = camera_query.single() {
            commands.entity(camera_entity).insert((
                ThirdPersonCamera {
                    sensitivity: Vec2::new(0.5, 0.4), // Reduced from default 1.0

                    // Zoom configuration (AC-style: closer to player)
                    zoom_enabled: false,
                    zoom: Zoom::new(2.5, 8.0), // Much closer than 1.5-30
                    zoom_sensitivity: 0.8,

                    // Camera offset over shoulder
                    offset_enabled: true,
                    offset: Offset::new(0.5, 0.3), // Slight offset for over-shoulder view
                    offset_toggle_enabled: true,
                    offset_toggle_speed: 5.0,

                    cursor_lock_key: KeyCode::KeyL,
                    cursor_lock_active: true, // Start with cursor locked
                    ..default()
                },
                Projection::from(PerspectiveProjection {
                    fov: fov.to_radians(),
                    ..Default::default()
                }),
                // RigidBody::Kinematic,
                // Collider::sphere(1.0),
            ));
        }
    }
}

/// Enable cursor lock when entering gameplay
fn enable_cursor_lock(mut camera_query: Query<&mut ThirdPersonCamera>) {
    for mut camera in camera_query.iter_mut() {
        camera.cursor_lock_active = true;
        info!("🔒 Cursor lock enabled for gameplay");
    }
}

/// Disable cursor lock when exiting gameplay
fn disable_cursor_lock(mut camera_query: Query<&mut ThirdPersonCamera>) {
    for mut camera in camera_query.iter_mut() {
        camera.cursor_lock_active = false;
        info!("🔓 Cursor lock disabled outside gameplay");
    }
}
