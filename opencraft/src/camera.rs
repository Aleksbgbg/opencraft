use crate::core::math::angle::Radians;
use crate::core::math::mat4::{self, Mat4x4};
use crate::core::math::rotor3::Rotor3;
use crate::core::math::vec3::Vec3;
use crate::core::math::{HALF_ROTATION, QUARTER_ROTATION, X_AXIS, Y_AXIS, Z_AXIS};

#[derive(Default)]
pub struct Camera {
  position: Vec3,
  rotation_x: Radians,
  rotation_y: Radians,
}

impl Camera {
  pub fn new() -> Self {
    Self::default()
  }

  fn rotor(rotation_x: Radians, rotation_y: Radians) -> Rotor3 {
    let rotor_x = {
      let orientation_x = Z_AXIS.angle_axis_rotate(rotation_x, X_AXIS);
      Rotor3::new(Z_AXIS, orientation_x)
    };
    let rotor_y = if rotation_y == HALF_ROTATION {
      let midpoint = Z_AXIS.angle_axis_rotate(QUARTER_ROTATION, Y_AXIS);
      Rotor3::new(Z_AXIS, midpoint) * Rotor3::new(midpoint, -Z_AXIS)
    } else {
      let orientation_y = Z_AXIS.angle_axis_rotate(rotation_y, Y_AXIS);
      Rotor3::new(Z_AXIS, orientation_y)
    };

    rotor_x * rotor_y
  }

  pub fn translate(&mut self, offset: Vec3) {
    self.position += Self::rotor(self.rotation_x, self.rotation_y).rotate(offset);
  }

  pub fn rotate<X: Into<Radians>, Y: Into<Radians>>(&mut self, x: X, y: Y) {
    self.rotation_x += x.into();
    self.rotation_y += y.into();

    self.rotation_x = self.rotation_x.clamp();
    self.rotation_y = self.rotation_y.clamp();
  }

  /// Returns a transformation to be applied on the world to simulate the
  /// position of the camera. The world transformation will be the inverse of
  /// all movements applied on the camera, as (for example) moving the camera
  /// backwards can be simulated by moving the entire world forwards.
  pub fn world_transform(&self, reverse: bool) -> Mat4x4 {
    let rotation_y = if reverse {
      self.rotation_y + HALF_ROTATION
    } else {
      self.rotation_y
    };
    mat4::rotate(-Self::rotor(self.rotation_x, rotation_y)) * mat4::translate(-self.position)
  }
}
