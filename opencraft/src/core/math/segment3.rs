use crate::core::math::vec3::Vec3;

pub struct Segment3 {
  start: Vec3,
  direction: Vec3,
  len: f32,
}

impl Segment3 {
  pub const fn start_direction_len(start: Vec3, direction: Vec3, len: f32) -> Self {
    Self {
      start,
      direction,
      len,
    }
  }

  pub const fn direction(&self) -> Vec3 {
    self.direction
  }

  pub const fn start(&self) -> Vec3 {
    self.start
  }

  pub fn end(&self) -> Vec3 {
    self.start + (self.direction * self.len)
  }
}
