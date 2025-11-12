use avian3d::prelude::*;
use bevy::prelude::*;

use super::{MovementController, Player};

const ROTATION_SPEED: f32 = 10.0;

pub fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&MovementController, &mut LinearVelocity, &mut Transform), With<Player>>,
    camera_query: Query<&Transform, (With<Camera3d>, Without<Player>)>,
    time: Res<Time>,
) {
    for (controller, mut velocity, mut player_transform) in query.iter_mut() {
        let mut direction = Vec3::ZERO;

        // Get camera forward/right for relative movement
        let (cam_forward, cam_right) = if let Ok(camera_transform) = camera_query.single() {
            let forward = camera_transform.forward();
            let right = camera_transform.right();
            // Flatten to horizontal plane (ignore Y)
            let forward_flat = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
            let right_flat = Vec3::new(right.x, 0.0, right.z).normalize_or_zero();
            (forward_flat, right_flat)
        } else {
            // Fallback to world axes if no camera
            (Vec3::NEG_Z, Vec3::X)
        };

        // WASD input relative to camera
        if keyboard.pressed(KeyCode::KeyW) {
            direction += cam_forward;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            direction -= cam_forward;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            direction -= cam_right;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            direction += cam_right;
        }

        // Normalize to prevent faster diagonal movement
        if direction.length() > 0.0 {
            direction = direction.normalize();
        }

        // Sprint multiplier
        let speed = if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight)
        {
            controller.speed * controller.sprint_multiplier
        } else {
            controller.speed
        };

        // Apply horizontal velocity (preserve vertical for jumping/gravity)
        velocity.x = direction.x * speed;
        velocity.z = direction.z * speed;
        // rotate player to face direction he is currently moving
        if direction.length_squared() > 0.0 {
            // player_transform.rotate_y(angle);.slerp(direction, ROTATION_SPEED * time.delta_secs());
            let target_rotation = Quat::from_rotation_arc(Vec3::NEG_Z, direction);
            player_transform.rotation = player_transform
                .rotation
                .slerp(target_rotation, ROTATION_SPEED * time.delta_secs());
        }
    }
}
