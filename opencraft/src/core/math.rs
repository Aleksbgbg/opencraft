pub mod angle;
pub mod clamp;
pub mod mat4;
pub mod rotor3;
pub mod vec3;

use crate::core::math::vec3::Vec3;

pub const X_AXIS: Vec3 = Vec3::new(1.0, 0.0, 0.0);
pub const Y_AXIS: Vec3 = Vec3::new(0.0, 1.0, 0.0);
pub const Z_AXIS: Vec3 = Vec3::new(0.0, 0.0, 1.0);
