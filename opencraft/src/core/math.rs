pub mod angle;
pub mod clamp;
pub mod mat4;
pub mod rotor3;
pub mod vec3;

use crate::core::math::angle::Radians;
use crate::core::math::vec3::Vec3;
use std::f32::consts::PI;

pub const X_AXIS: Vec3 = Vec3::new(1.0, 0.0, 0.0);
pub const Y_AXIS: Vec3 = Vec3::new(0.0, 1.0, 0.0);
pub const Z_AXIS: Vec3 = Vec3::new(0.0, 0.0, 1.0);

pub const QUARTER_ROTATION: Radians = Radians::new(PI / 2.0);
pub const HALF_ROTATION: Radians = Radians::new(PI);
pub const FULL_ROTATION: Radians = Radians::new(2.0 * PI);

pub fn nearly_eq(lhs: f32, rhs: f32) -> bool {
  nearly_eq_tolerance(lhs, rhs, 1.0)
}

pub fn nearly_eq_tolerance(lhs: f32, rhs: f32, tolerance_multiplier: f32) -> bool {
  (lhs - rhs).abs() <= (tolerance_multiplier * f32::EPSILON)
}
