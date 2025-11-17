//! Procedural animation curve generation for target matching

use bevy::{
    animation::{animated_field, AnimationTargetId},
    prelude::*,
};

use super::{TargetBone, TargetMatchRequest};

/// Generate a custom animation curve to move a bone to a target position
pub fn generate_target_curve(
    request: &TargetMatchRequest,
    bone_target_id: AnimationTargetId,
    current_position: Vec3,
) -> AnimationClip {
    let mut clip = AnimationClip::default();

    // Get the time range for matching
    let (start_time, end_time) = request.time_range();
    let duration = request.match_duration();

    // Create keyframes for the bone's translation
    // We'll use a smooth curve from current position to target
    let keyframes = generate_keyframes(
        current_position,
        request.target_position,
        start_time,
        end_time,
    );

    // Create the curve
    let times: Vec<f32> = keyframes.iter().map(|(t, _)| *t).collect();
    let positions: Vec<Vec3> = keyframes.iter().map(|(_, p)| *p).collect();

    clip.add_curve_to_target(
        bone_target_id,
        AnimatableCurve::new(
            animated_field!(Transform::translation),
            UnevenSampleAutoCurve::new(times.into_iter().zip(positions))
                .expect("Failed to create target matching curve"),
        ),
    );

    clip.set_duration(request.animation_duration);

    clip
}

/// Generate smooth keyframes from start to end position
fn generate_keyframes(
    start_pos: Vec3,
    end_pos: Vec3,
    start_time: f32,
    end_time: f32,
) -> Vec<(f32, Vec3)> {
    // For now, use simple linear interpolation with multiple keyframes
    // This could be enhanced with easing functions
    let num_keyframes = 5;
    let mut keyframes = Vec::new();

    for i in 0..=num_keyframes {
        let t = i as f32 / num_keyframes as f32;
        let time = start_time + t * (end_time - start_time);
        let position = start_pos.lerp(end_pos, t);
        keyframes.push((time, position));
    }

    keyframes
}

/// Generate a curve with custom easing
pub fn generate_target_curve_with_easing(
    request: &TargetMatchRequest,
    bone_target_id: AnimationTargetId,
    current_position: Vec3,
    easing: EasingFunction,
) -> AnimationClip {
    let mut clip = AnimationClip::default();

    let (start_time, end_time) = request.time_range();
    let keyframes = generate_keyframes_with_easing(
        current_position,
        request.target_position,
        start_time,
        end_time,
        easing,
    );

    let times: Vec<f32> = keyframes.iter().map(|(t, _)| *t).collect();
    let positions: Vec<Vec3> = keyframes.iter().map(|(_, p)| *p).collect();

    clip.add_curve_to_target(
        bone_target_id,
        AnimatableCurve::new(
            animated_field!(Transform::translation),
            UnevenSampleAutoCurve::new(times.into_iter().zip(positions))
                .expect("Failed to create eased curve"),
        ),
    );

    clip.set_duration(request.animation_duration);

    clip
}

/// Easing function type
#[derive(Debug, Clone, Copy)]
pub enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl EasingFunction {
    fn apply(&self, t: f32) -> f32 {
        match self {
            EasingFunction::Linear => t,
            EasingFunction::EaseIn => t * t,
            EasingFunction::EaseOut => {
                let t = 1.0 - t;
                1.0 - t * t
            }
            EasingFunction::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    let t = 2.0 * t - 1.0;
                    -0.5 * (t * (t - 2.0) - 1.0)
                }
            }
        }
    }
}

fn generate_keyframes_with_easing(
    start_pos: Vec3,
    end_pos: Vec3,
    start_time: f32,
    end_time: f32,
    easing: EasingFunction,
) -> Vec<(f32, Vec3)> {
    let num_keyframes = 8; // More keyframes for smooth easing
    let mut keyframes = Vec::new();

    for i in 0..=num_keyframes {
        let t = i as f32 / num_keyframes as f32;
        let eased_t = easing.apply(t);
        let time = start_time + t * (end_time - start_time);
        let position = start_pos.lerp(end_pos, eased_t);
        keyframes.push((time, position));
    }

    keyframes
}

/// Calculate the required root offset to achieve target matching
///
/// This is an alternative approach that moves the character root instead of
/// generating a custom curve for the bone
pub fn calculate_root_offset(
    bone_world_pos: Vec3,
    target_pos: Vec3,
    character_root: Vec3,
) -> Vec3 {
    let bone_offset_from_root = bone_world_pos - character_root;
    let required_root = target_pos - bone_offset_from_root;
    required_root - character_root
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyframe_generation() {
        let start = Vec3::ZERO;
        let end = Vec3::new(1.0, 2.0, 3.0);
        let keyframes = generate_keyframes(start, end, 0.0, 1.0);

        assert!(!keyframes.is_empty());
        assert_eq!(keyframes[0].1, start);
        assert_eq!(keyframes.last().unwrap().1, end);
    }

    #[test]
    fn test_easing_functions() {
        assert_eq!(EasingFunction::Linear.apply(0.5), 0.5);
        assert!(EasingFunction::EaseIn.apply(0.5) < 0.5);
        assert!(EasingFunction::EaseOut.apply(0.5) > 0.5);
    }

    #[test]
    fn test_root_offset_calculation() {
        let bone_pos = Vec3::new(1.0, 0.5, 0.0);
        let target_pos = Vec3::new(2.0, 0.5, 0.0);
        let root_pos = Vec3::ZERO;

        let offset = calculate_root_offset(bone_pos, target_pos, root_pos);

        // Should move root 1 unit in X to make bone reach target
        assert_eq!(offset, Vec3::new(1.0, 0.0, 0.0));
    }
}
