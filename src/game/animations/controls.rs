use bevy::prelude::*;
use bevy_tnua::prelude::*;
use bevy_hotpatching_experiments::hot;
use crate::{game::player::{MovementController, Player}};


const FLOAT_HEIGHT: f32 = 0.9;
const ROTATION_SPEED: f32 = 10.0;

#[hot]
pub fn apply_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut TnuaController>,
    mut movement_query: Query<(&MovementController, &mut Transform), With<Player>>,
    camera_query: Query<&Transform, (With<Camera3d>, Without<Player>)>,
    time: Res<Time>,
) {
    let Ok(mut controller) = query.single_mut() else {
        return;
    };

    let Ok((movement_controller, mut player_transform)) = movement_query.single_mut() else {
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


    // Feed the basis every frame. Even if the player doesn't move - just use `desired_velocity:
    // Vec3::ZERO`. `TnuaController` starts without a basis, which will make the character collider
    // just fall.
    controller.basis(TnuaBuiltinWalk {
        // The `desired_velocity` determines how the character will move.
        desired_velocity: direction.normalize_or_zero() * 2.0,
        // The `float_height` must be greater (even if by little) from the distance between the
        // character's center and the lowest point of its collider.
        float_height: FLOAT_HEIGHT,
        turning_angvel: 12.0,  // Increased for more responsive turning.
        desired_forward: Dir3::new(direction.normalize_or_zero()).ok(),
        // `TnuaBuiltinWalk` has many other fields for customizing the movement - but they have
        // sensible defaults. Refer to the `TnuaBuiltinWalk`'s documentation to learn what they do.
        ..Default::default()
    });

    // Feed the jump action every frame as long as the player holds the jump button. If the player
    // stops holding the jump button, simply stop feeding the action.
    if keyboard.pressed(KeyCode::Space) {
        controller.action(TnuaBuiltinJump {
            // The height is the only mandatory field of the jump button.
            height: movement_controller.jump_height,
            input_buffer_time: 0.5,
            // `TnuaBuiltinJump` also has customization fields with sensible defaults.
            ..Default::default()
        });
    }

    // if direction.length_squared() > 0.0 {
    //     // player_transform.rotate_y(angle);.slerp(direction, ROTATION_SPEED * time.delta_secs());
    //     let target_rotation = Quat::from_rotation_arc(Vec3::NEG_Z, direction.normalize_or_zero());
    //     player_transform.rotation = player_transform
    //         .rotation
    //         .slerp(target_rotation, ROTATION_SPEED * time.delta_secs());
    // }
}
