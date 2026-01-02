use crate::core::math::projection::Perspective;
use crate::core::math::rotor3::Rotor3;
use crate::core::math::vec3::Vec3;
use crate::core::math::{X_AXIS, Y_AXIS, Z_AXIS};

pub struct Frustum3 {
  origin: Vec3,

  axis_x: Vec3,
  axis_y: Vec3,
  axis_z: Vec3,

  extent_x_near: f32,
  extent_y_near: f32,
  extent_z_near: f32,

  depth_ratio: f32,
}

impl Frustum3 {
  pub fn new(origin: Vec3, orientation: Rotor3, projection: &Perspective) -> Self {
    Self {
      origin,
      axis_x: orientation.rotate(X_AXIS),
      axis_y: orientation.rotate(Y_AXIS),
      axis_z: orientation.rotate(Z_AXIS),
      extent_x_near: projection.x_near(),
      extent_y_near: projection.y_near(),
      extent_z_near: projection.z_near(),
      depth_ratio: projection.depth_ratio(),
    }
  }

  pub const fn origin(&self) -> Vec3 {
    self.origin
  }

  pub const fn axis_x(&self) -> Vec3 {
    self.axis_x
  }

  pub const fn axis_y(&self) -> Vec3 {
    self.axis_y
  }

  pub const fn axis_z(&self) -> Vec3 {
    self.axis_z
  }

  pub const fn extent_x_near(&self) -> f32 {
    self.extent_x_near
  }

  pub const fn extent_y_near(&self) -> f32 {
    self.extent_y_near
  }

  pub const fn extent_z_near(&self) -> f32 {
    self.extent_z_near
  }

  pub const fn depth_ratio(&self) -> f32 {
    self.depth_ratio
  }
}
