pub mod angle;
pub mod bivec3;
pub mod mat4;
pub mod rotor3;
pub mod vec3;

use crate::core::math::bivec3::BiVec3;
use crate::core::math::vec3::Vec3;

pub const X_AXIS: Vec3 = Vec3::new(1.0, 0.0, 0.0);
pub const Y_AXIS: Vec3 = Vec3::new(0.0, 1.0, 0.0);
pub const Z_AXIS: Vec3 = Vec3::new(0.0, 0.0, 1.0);

#[allow(dead_code)]
pub const XY_PLANE: BiVec3 = BiVec3::new(1.0, 0.0, 0.0);
pub const YZ_PLANE: BiVec3 = BiVec3::new(0.0, 1.0, 0.0);
pub const ZX_PLANE: BiVec3 = BiVec3::new(0.0, 0.0, 1.0);

pub fn nearly_eq(lhs: f32, rhs: f32) -> bool {
  nearly_eq_tolerance(lhs, rhs, 1.0)
}

pub fn nearly_eq_tolerance(lhs: f32, rhs: f32, tolerance_multiplier: f32) -> bool {
  (lhs - rhs).abs() <= (tolerance_multiplier * f32::EPSILON)
}
