pub mod aligned_box3;
pub mod angle;
pub mod bivec3;
pub mod intersect;
pub mod mat4;
pub mod rotor3;
pub mod segment3;
pub mod vec2;
pub mod vec3;

use crate::core::math::bivec3::BiVec3;
use crate::core::math::vec3::Vec3;
use crate::core::type_conversions::CoerceLossyRound;
use std::ops::RangeInclusive;
use strum::EnumIter;

pub const X_AXIS: Vec3 = Vec3::new(1.0, 0.0, 0.0);
pub const Y_AXIS: Vec3 = Vec3::new(0.0, 1.0, 0.0);
pub const Z_AXIS: Vec3 = Vec3::new(0.0, 0.0, 1.0);

pub const XY_PLANE: BiVec3 = BiVec3::new(1.0, 0.0, 0.0);
pub const YZ_PLANE: BiVec3 = BiVec3::new(0.0, 1.0, 0.0);
pub const ZX_PLANE: BiVec3 = BiVec3::new(0.0, 0.0, 1.0);

#[derive(Debug, Clone, Copy, EnumIter)]
pub enum Direction {
  XPos,
  XNeg,
  YPos,
  YNeg,
  ZPos,
  ZNeg,
}

impl Direction {
  pub fn normal(self) -> Vec3 {
    match self {
      Direction::XPos => X_AXIS,
      Direction::XNeg => -X_AXIS,
      Direction::YPos => Y_AXIS,
      Direction::YNeg => -Y_AXIS,
      Direction::ZPos => Z_AXIS,
      Direction::ZNeg => -Z_AXIS,
    }
  }
}

pub fn nearly_eq(lhs: f32, rhs: f32) -> bool {
  nearly_eq_tolerance(lhs, rhs, 1.0)
}

pub fn nearly_eq_tolerance(lhs: f32, rhs: f32, tolerance_multiplier: f32) -> bool {
  (lhs - rhs).abs() <= (tolerance_multiplier * f32::EPSILON)
}

/// Returns true if value ∈ [min, max].
pub fn in_range<T>(value: T, min: T, max: T) -> bool
where
  T: PartialOrd,
{
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

pub fn affine_transform(
  value: f32,
  input: RangeInclusive<f32>,
  output: RangeInclusive<f32>,
) -> f32 {
  let input_range = input.end() - input.start();
  let output_range = output.end() - output.start();

  let scale_factor = output_range / input_range;

  ((value - input.start()) * scale_factor) + output.start()
}

pub fn normalized_f32_to_u8(value: f32) -> u8 {
  (value * (f32::powf(2.0, 8.0) - 1.0)).coerce_lossy_round()
}
