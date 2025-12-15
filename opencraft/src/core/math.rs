pub mod aligned_box3;
pub mod angle;
pub mod bivec3;
pub mod mat4;
pub mod rotor3;
pub mod segment3;
pub mod vec3;

use crate::core::math::bivec3::BiVec3;
use crate::core::math::vec3::Vec3;

pub const X_AXIS: Vec3 = Vec3::new(1.0, 0.0, 0.0);
pub const Y_AXIS: Vec3 = Vec3::new(0.0, 1.0, 0.0);
pub const Z_AXIS: Vec3 = Vec3::new(0.0, 0.0, 1.0);

pub const XY_PLANE: BiVec3 = BiVec3::new(1.0, 0.0, 0.0);
pub const YZ_PLANE: BiVec3 = BiVec3::new(0.0, 1.0, 0.0);
pub const ZX_PLANE: BiVec3 = BiVec3::new(0.0, 0.0, 1.0);

pub fn nearly_eq(lhs: f32, rhs: f32) -> bool {
  nearly_eq_tolerance(lhs, rhs, 1.0)
}

pub fn nearly_eq_tolerance(lhs: f32, rhs: f32, tolerance_multiplier: f32) -> bool {
  (lhs - rhs).abs() <= (tolerance_multiplier * f32::EPSILON)
}

/// Returns true if value ∈ [min, max].
pub fn in_range(value: f32, min: f32, max: f32) -> bool {
  assert!(min <= max);

  (min <= value) && (value <= max)
}

pub fn min_max(a: f32, b: f32) -> (f32, f32) {
  (a.min(b), a.max(b))
}

pub fn align(value: usize, alignment: usize) -> usize {
  let misalignment = value % alignment;
  let padding = (alignment - misalignment) % alignment;

  value + padding
}

/// Split a value into two halves, one rounded up, the other rounded down.
/// Useful when you need to split an integer into two halves, whether it is odd
/// or even.
pub fn split(value: f32) -> (f32, f32) {
  let half = value / 2.0;
  (half.ceil(), half.floor())
}
