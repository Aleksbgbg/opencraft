use crate::core::math::vec3::Vec3;

pub struct AlignedBox3 {
  origin: Vec3,
  extent: Vec3,
}

impl AlignedBox3 {
  pub const fn new(origin: Vec3, extent: Vec3) -> Self {
    Self { origin, extent }
  }

  pub const fn cube(origin: Vec3, extent: f32) -> Self {
    Self::new(origin, Vec3::new(extent, extent, extent))
  }

  pub const fn origin(&self) -> Vec3 {
    self.origin
  }

  pub const fn extent(&self) -> Vec3 {
    self.extent
  }
}
