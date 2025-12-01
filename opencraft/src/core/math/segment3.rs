use crate::core::math::vec3::Vec3;

pub struct Segment3 {
  origin: Vec3,
  direction: Vec3,
  extent: f32,
}

impl Segment3 {
  pub const fn origin_direction_extent(origin: Vec3, direction: Vec3, extent: f32) -> Self {
    Self {
      origin,
      direction,
      extent,
    }
  }

  pub fn start_direction_len(start: Vec3, direction: Vec3, len: f32) -> Self {
    let extent = len / 2.0;
    let origin = start + (direction * extent);

    Self::origin_direction_extent(origin, direction, extent)
  }

  pub const fn origin(&self) -> Vec3 {
    self.origin
  }

  pub const fn direction(&self) -> Vec3 {
    self.direction
  }

  pub const fn extent(&self) -> f32 {
    self.extent
  }

  pub fn start(&self) -> Vec3 {
    self.origin - (self.direction * self.extent)
  }

  pub fn end(&self) -> Vec3 {
    self.origin + (self.direction * self.extent)
  }
}
