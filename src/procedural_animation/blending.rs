//! Velocity-based pose blending logic

use bevy::prelude::*;
use avian3d::prelude::*;
use super::{ProceduralAnimationController, PoseBlendState, PoseId, ContactState};

/// Update blend weights based on character velocity and state
pub fn update_blend_weights(
    mut controllers: Query<(&mut ProceduralAnimationController, &LinearVelocity, &Transform)>,
    time: Res<Time>,
) {
    for (mut controller, velocity, transform) in controllers.iter_mut() {
        if !controller.enabled {
            continue;
        }

        let vel = velocity.0;
        let speed = vel.xz().length(); // Horizontal speed
        let acceleration = Vec3::ZERO; // TODO: Calculate from previous frame

        // Update velocity and acceleration
        controller.blend_state.velocity = speed;
        controller.blend_state.acceleration = acceleration;

        // Calculate contact state (simplified - TODO: use raycast)
        controller.blend_state.contact_state = if transform.translation.y < 0.1 {
            ContactState::Grounded
        } else {
            ContactState::Airborne
        };

        // Calculate blend weights
        controller.blend_state.active_poses = calculate_pose_weights(
            speed,
            acceleration,
            controller.blend_state.contact_state,
            &mut controller.blend_state.foot_phase,
            time.delta_secs(),
        );
    }
}

/// Calculate which poses to blend and their weights
fn calculate_pose_weights(
    speed: f32,
    _acceleration: Vec3,
    contact_state: ContactState,
    foot_phase: &mut f32,
    delta_time: f32,
) -> Vec<(PoseId, f32)> {
    use PoseId::*;

    match contact_state {
        ContactState::Grounded => {
            // Standing still (< 0.5 m/s)
            if speed < 0.5 {
                vec![(Idle, 1.0)]
            }
            // Walking (0.5 - 3.0 m/s)
            else if speed < 3.0 {
                blend_walk_cycle(speed, foot_phase, delta_time)
            }
            // Running (> 3.0 m/s)
            else {
                blend_run_cycle(speed, foot_phase, delta_time)
            }
        }

        ContactState::Airborne => {
            // TODO: Distinguish between jump/fall based on velocity.y
            vec![(JumpAirborne, 1.0)]
        }

        ContactState::Landing => {
            vec![(JumpLanding, 1.0)]
        }
    }
}

/// Blend between walk left and walk right poses based on cycle phase
fn blend_walk_cycle(speed: f32, foot_phase: &mut f32, delta_time: f32) -> Vec<(PoseId, f32)> {
    use PoseId::*;

    // Walk cycle frequency (steps per second)
    // Slower walking = slower cycle
    let cycle_frequency = 1.0 + (speed - 0.5) * 0.5; // 1.0-2.25 Hz

    // Update phase
    *foot_phase += cycle_frequency * delta_time;
    *foot_phase %= 1.0; // Keep in 0.0-1.0 range

    // Blend between left and right foot forward
    if *foot_phase < 0.5 {
        // First half: transition from left to right
        let t = *foot_phase * 2.0; // 0.0-1.0
        vec![
            (WalkLeftFootForward, 1.0 - t),
            (WalkRightFootForward, t),
        ]
    } else {
        // Second half: transition from right to left
        let t = (*foot_phase - 0.5) * 2.0; // 0.0-1.0
        vec![
            (WalkRightFootForward, 1.0 - t),
            (WalkLeftFootForward, t),
        ]
    }
}

/// Blend between run left and run right poses based on cycle phase
fn blend_run_cycle(speed: f32, foot_phase: &mut f32, delta_time: f32) -> Vec<(PoseId, f32)> {
    use PoseId::*;

    // Run cycle frequency (steps per second)
    // Faster running = faster cycle
    let cycle_frequency = 2.0 + (speed - 3.0) * 0.3; // 2.0-3.5 Hz

    // Update phase
    *foot_phase += cycle_frequency * delta_time;
    *foot_phase %= 1.0;

    // Blend between left and right foot forward
    if *foot_phase < 0.5 {
        let t = *foot_phase * 2.0;
        vec![
            (RunLeftFootForward, 1.0 - t),
            (RunRightFootForward, t),
        ]
    } else {
        let t = (*foot_phase - 0.5) * 2.0;
        vec![
            (RunRightFootForward, 1.0 - t),
            (RunLeftFootForward, t),
        ]
    }
}

/// Apply the blended pose to character bones
pub fn apply_pose_blending(
    controllers: Query<(&ProceduralAnimationController, &Children)>,
    mut bone_transforms: Query<(&mut Transform, &Name)>,
    // TODO: Add pose library and assets here
) {
    for (controller, children) in controllers.iter() {
        if !controller.enabled {
            continue;
        }

        // TODO: Get actual pose data from PoseLibrary
        // TODO: Blend poses according to active_poses weights
        // TODO: Apply blended transforms to bones

        // For now, just log the blend state
        if controller.blend_state.active_poses.len() > 0 {
            trace!(
                "Blending {} poses at speed {:.2} m/s, phase {:.2}",
                controller.blend_state.active_poses.len(),
                controller.blend_state.velocity,
                controller.blend_state.foot_phase
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_walk_cycle_blending() {
        let mut phase = 0.0;
        let poses = blend_walk_cycle(1.5, &mut phase, 0.1);

        // Should return 2 poses
        assert_eq!(poses.len(), 2);

        // Weights should sum to ~1.0
        let total_weight: f32 = poses.iter().map(|(_, w)| w).sum();
        assert!((total_weight - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_idle_below_threshold() {
        let mut phase = 0.0;
        let poses = calculate_pose_weights(0.1, Vec3::ZERO, ContactState::Grounded, &mut phase, 0.016);

        assert_eq!(poses.len(), 1);
        assert_eq!(poses[0].0, PoseId::Idle);
        assert_eq!(poses[0].1, 1.0);
    }
}
