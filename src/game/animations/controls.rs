use bevy::prelude::*;
use bevy_tnua::{builtins::TnuaBuiltinDash, prelude::*};
use bevy_hotpatching_experiments::hot;
use crate::game::{
    player::{MovementController, Player},
    animations::models::MovementTimer,
};
use std::time::Duration;


const FLOAT_HEIGHT: f32 = 0.9;
const ROTATION_SPEED: f32 = 10.0;
const WALK_TO_RUN_DURATION: f32 = 1.0; // Seconds to transition from walk to run

#[hot]
pub fn apply_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut TnuaController>,
    mut movement_query: Query<(&mut MovementController, &MovementTimer, &mut Transform), With<Player>>,
    camera_query: Query<&Transform, (With<Camera3d>, Without<Player>)>,
    time: Res<Time>,
) {
    let Ok(mut controller) = query.single_mut() else {
        return;
    };

    let Ok((mut movement_controller, movement_timer, mut player_transform)) = movement_query.single_mut() else {
        return;
    };

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

    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::ArrowUp) || keyboard.pressed(KeyCode::KeyW) {
        direction += cam_forward;
    }
    if keyboard.pressed(KeyCode::ArrowDown)  || keyboard.pressed(KeyCode::KeyS){
        direction -= cam_forward;
    }
    if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA){
        direction -= cam_right;
    }
    if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD){
        direction += cam_right;
    }

    // Calculate target speed based on movement timer
    let is_moving = direction.length_squared() > 0.0;
    let target_speed = if !is_moving {
        // Not moving - target is zero
        0.0
    } else {
        // Moving - determine target based on how long we've been moving
        let time_moving = movement_timer.time_in_state.as_secs_f32();

        if time_moving < WALK_TO_RUN_DURATION {
            // During the first second, accelerate from 0 to walk_speed, then walk_speed to run_speed
            // Use a simple linear interpolation
            let t = time_moving / WALK_TO_RUN_DURATION;
            movement_controller.walk_speed + t * (movement_controller.run_speed - movement_controller.walk_speed)
        } else {
            // After 1 second, target is run speed
            movement_controller.run_speed
        }
    };

    // Smoothly interpolate current speed towards target speed
    let acceleration = 15.0; // How fast we accelerate/decelerate (units per second per second)
    let speed_delta = acceleration * time.delta_secs();

    if target_speed > movement_controller.current_speed {
        // Accelerating
        movement_controller.current_speed = (movement_controller.current_speed + speed_delta).min(target_speed);
    } else {
        // Decelerating
        movement_controller.current_speed = (movement_controller.current_speed - speed_delta).max(target_speed);
    }

    // Feed the basis every frame. Even if the player doesn't move - just use `desired_velocity:
    // Vec3::ZERO`. `TnuaController` starts without a basis, which will make the character collider
    // just fall.
    controller.basis(TnuaBuiltinWalk {
        // The `desired_velocity` determines how the character will move.
        desired_velocity: direction.normalize_or_zero() * movement_controller.current_speed,
        // The `float_height` must be greater (even if by little) from the distance between the
        // character's center and the lowest point of its collider.
        float_height: FLOAT_HEIGHT,
        turning_angvel: 12.0,  // Increased for more responsive turning.
        desired_forward: Dir3::new(direction.normalize_or_zero()).ok(),
        // `TnuaBuiltinWalk` has many other fields for customizing the movement - but they have
        // sensible defaults. Refer to the `TnuaBuiltinWalk`'s documentation to learn what they do.
        ..Default::default()
    });



}
