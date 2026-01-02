use crate::core::math::angle::Angle;
use crate::core::math::vec2::Vec2;

pub struct Perspective {
  projection_distance: f32,
  inverse_aspect_ratio: f32,
  x_near: f32,
  y_near: f32,
  z_near: f32,
  z_far: f32,
  depth_ratio: f32,
}

impl Perspective {
  pub fn new(screen_size: Vec2, horizontal_fov: Angle, z_near: f32, z_far: f32) -> Self {
    let projection_distance = 1.0 / (horizontal_fov / 2.0).tan();
    let clip_space_to_camera_space_ratio = z_near / projection_distance;
    let aspect_ratio = screen_size.y() / screen_size.x();

    Self {
      projection_distance,
      inverse_aspect_ratio: screen_size.x() / screen_size.y(),
      x_near: clip_space_to_camera_space_ratio,
      y_near: aspect_ratio * clip_space_to_camera_space_ratio,
      z_near,
      z_far,
      depth_ratio: z_far / z_near,
    }
  }

  pub fn projection_distance(&self) -> f32 {
    self.projection_distance
  }

  pub fn inverse_aspect_ratio(&self) -> f32 {
    self.inverse_aspect_ratio
  }

  pub fn x_near(&self) -> f32 {
    self.x_near
  }

  pub fn y_near(&self) -> f32 {
    self.y_near
  }

  pub fn z_near(&self) -> f32 {
    self.z_near
  }

  pub fn z_far(&self) -> f32 {
    self.z_far
  }

  pub fn depth_ratio(&self) -> f32 {
    self.depth_ratio
  }
}
