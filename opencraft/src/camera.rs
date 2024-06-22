use crate::core::math::angle::{Angle, HALF_ROTATION, QUARTER_ROTATION};
use crate::core::math::mat4::{self, Mat4x4};
use crate::core::math::rotor3::Rotor3;
use crate::core::math::vec3::Vec3;
use crate::core::math::{YZ_PLANE, ZX_PLANE};

fn rotor(yaw: Angle, pitch: Angle) -> Rotor3 {
  let rotor_yaw = Rotor3::angle_plane(yaw, ZX_PLANE);
  let rotor_pitch = Rotor3::angle_plane(pitch, YZ_PLANE);

  rotor_yaw * rotor_pitch
}

pub enum Direction {
  Forward,
  Backward,
}

#[derive(Default)]
pub struct Camera {
  position: Vec3,
  yaw: Angle,
  pitch: Angle,
}

impl Camera {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn translate(&mut self, offset: Vec3) {
    self.position += rotor(self.yaw, self.pitch).rotate(offset);
  }

  pub fn rotate(&mut self, yaw: Angle, pitch: Angle) {
    self.yaw += yaw;
    self.pitch += pitch;

    self.yaw = self.yaw.wrap();
    self.pitch = self.pitch.clamp(QUARTER_ROTATION);
  }

  /// Returns a transformation to be applied on the world to simulate the
  /// position of the camera.
  ///
  /// The world transformation will be the inverse of
  /// all movements applied on the camera, as (for example) moving the camera
  /// backwards can be simulated by moving the entire world forwards.
  ///
  /// The camera can be flipped backwards by passing in [`Direction::Backward`]
  /// for the `facing` parameter.
  pub fn world_transform(&self, facing: Direction) -> Mat4x4 {
    let world_rotor = -rotor(self.yaw, self.pitch);
    let world_rotor = match facing {
      Direction::Forward => world_rotor,
      Direction::Backward => Rotor3::angle_plane(HALF_ROTATION, ZX_PLANE) * world_rotor,
    };

    mat4::rotate(world_rotor) * mat4::translate(-self.position)
  }
}
