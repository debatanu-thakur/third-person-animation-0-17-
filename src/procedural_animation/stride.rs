//! Stride length and foot placement calculation

use bevy::prelude::*;

/// Calculate stride length based on velocity and terrain
pub struct StrideCalculator {
    /// Base stride length at normal walk speed (meters)
    pub base_walk_stride: f32,
    /// Base stride length at normal run speed (meters)
    pub base_run_stride: f32,
    /// Stride length scaling based on velocity
    pub velocity_scale: f32,
    /// Stride length adjustment for slopes
    pub slope_factor: f32,
}

impl Default for StrideCalculator {
    fn default() -> Self {
        Self {
            base_walk_stride: 0.6,  // Average human walk stride
            base_run_stride: 1.2,   // Average human run stride
            velocity_scale: 1.0,
            slope_factor: 1.0,
        }
    }
}

impl StrideCalculator {
    /// Calculate stride length based on current velocity
    ///
    /// # Arguments
    /// * `velocity` - Current horizontal velocity (m/s)
    /// * `terrain_normal` - Normal vector of terrain under character
    ///
    /// # Returns
    /// Stride length in meters
    pub fn calculate_stride_length(
        &self,
        velocity: f32,
        terrain_normal: Vec3,
    ) -> f32 {
        // Base stride depends on speed range
        let base_stride = if velocity < 3.0 {
            // Walking range: interpolate from 0 to base_walk_stride
            self.base_walk_stride * (velocity / 3.0).min(1.0)
        } else {
            // Running range: interpolate from walk to run stride
            let run_factor = ((velocity - 3.0) / 5.0).min(1.0);
            self.base_walk_stride + (self.base_run_stride - self.base_walk_stride) * run_factor
        };

        // Adjust for terrain angle
        let slope_adjustment = calculate_slope_adjustment(terrain_normal);

        base_stride * self.velocity_scale * slope_adjustment
    }

    /// Calculate foot placement target for current stride
    ///
    /// # Arguments
    /// * `character_pos` - Current character position
    /// * `velocity` - Current velocity vector
    /// * `stride_length` - Calculated stride length
    /// * `foot_phase` - Current foot phase (0.0-1.0)
    /// * `is_left_foot` - Whether calculating for left foot
    ///
    /// # Returns
    /// Target position for foot placement
    pub fn calculate_foot_target(
        &self,
        character_pos: Vec3,
        velocity: Vec3,
        stride_length: f32,
        foot_phase: f32,
        is_left_foot: bool,
    ) -> Vec3 {
        // Get forward direction from velocity
        let forward = velocity.normalize_or_zero();
        let right = Vec3::Y.cross(forward).normalize_or_zero();

        // Stride offset along movement direction
        let stride_offset = if is_left_foot {
            // Left foot: -0.5 to 0.5 of stride
            (foot_phase - 0.5) * stride_length
        } else {
            // Right foot: 0.0 to 1.0 of stride (offset by half cycle)
            ((foot_phase + 0.5) % 1.0 - 0.5) * stride_length
        };

        // Lateral offset for left/right foot
        let lateral_offset = if is_left_foot {
            0.15 // Left foot 15cm to the left
        } else {
            -0.15 // Right foot 15cm to the right
        };

        // Calculate target position
        character_pos
            + forward * stride_offset
            + right * lateral_offset
    }
}

/// Calculate stride length adjustment based on terrain slope
///
/// # Arguments
/// * `terrain_normal` - Normal vector of terrain surface
///
/// # Returns
/// Multiplier for stride length (1.0 = flat, <1.0 = uphill, >1.0 = downhill)
fn calculate_slope_adjustment(terrain_normal: Vec3) -> f32 {
    // Calculate angle from vertical (0 = flat, 1 = vertical)
    let slope_factor = 1.0 - terrain_normal.dot(Vec3::Y);

    // Adjust stride based on slope
    if slope_factor < 0.01 {
        // Flat ground
        1.0
    } else if slope_factor > 0.0 {
        // Uphill: shorter strides (down to 70% on steep slopes)
        (1.0 - slope_factor * 0.3).max(0.7)
    } else {
        // Downhill: slightly longer strides (up to 110%)
        (1.0 + slope_factor.abs() * 0.1).min(1.1)
    }
}

/// System to visualize stride calculations (debug)
#[allow(dead_code)]
pub fn debug_visualize_stride(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &crate::procedural_animation::ProceduralAnimationController)>,
) {
    for (transform, controller) in query.iter() {
        if !controller.enabled {
            continue;
        }

        let stride_length = controller.blend_state.stride_length;
        let foot_phase = controller.blend_state.foot_phase;

        // Draw stride length indicator
        gizmos.line(
            transform.translation,
            transform.translation + transform.forward() * stride_length,
            Color::srgb(1.0, 1.0, 0.0),
        );

        // Draw foot phase indicator (circle that rotates)
        let phase_angle = foot_phase * std::f32::consts::TAU;
        let phase_pos = transform.translation
            + Vec3::new(phase_angle.cos(), 0.5, phase_angle.sin()) * 0.2;
        gizmos.sphere(
            Isometry3d::from_translation(phase_pos),
            0.05,
            Color::srgb(0.0, 1.0, 1.0),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stride_length_walk() {
        let calc = StrideCalculator::default();
        let stride = calc.calculate_stride_length(1.5, Vec3::Y);

        // Walking speed should give partial walk stride
        assert!(stride > 0.0);
        assert!(stride < calc.base_walk_stride);
    }

    #[test]
    fn test_stride_length_run() {
        let calc = StrideCalculator::default();
        let stride = calc.calculate_stride_length(6.0, Vec3::Y);

        // Running speed should give stride between walk and run
        assert!(stride > calc.base_walk_stride);
        assert!(stride <= calc.base_run_stride);
    }

    #[test]
    fn test_slope_adjustment_uphill() {
        // 45 degree slope
        let normal = Vec3::new(0.0, 0.707, 0.707).normalize();
        let adjustment = calculate_slope_adjustment(normal);

        // Uphill should reduce stride
        assert!(adjustment < 1.0);
        assert!(adjustment >= 0.7);
    }
}
