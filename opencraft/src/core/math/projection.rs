use crate::core::math::angle::Angle;
use crate::core::math::vec2::Vec2;

pub struct Perspective {
  projection_distance: f32,
  inverse_aspect_ratio: f32,
  z_near: f32,
  z_far: f32,
}

impl Perspective {
  pub fn new(screen_size: Vec2, horizontal_fov: Angle, z_near: f32, z_far: f32) -> Self {
    Self {
      projection_distance: 1.0 / (horizontal_fov / 2.0).tan(),
      inverse_aspect_ratio: screen_size.x() / screen_size.y(),
      z_near,
      z_far,
    }
  }

  pub fn projection_distance(&self) -> f32 {
    self.projection_distance
  }

  pub fn inverse_aspect_ratio(&self) -> f32 {
    self.inverse_aspect_ratio
  }

  pub fn z_near(&self) -> f32 {
    self.z_near
  }

  pub fn z_far(&self) -> f32 {
    self.z_far
  }
}
