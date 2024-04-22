use crate::core::math::clamp;
use std::f32::consts::PI;
use std::ops::AddAssign;

fn degrees_to_radians(degrees: f32) -> f32 {
  degrees * (PI / 180.0)
}

#[derive(Clone, Copy)]
pub struct Degrees(f32);

impl Degrees {
  pub const fn new(value: f32) -> Self {
    Self(value)
  }

  pub const fn value(self) -> f32 {
    self.0
  }

  pub fn radians(self) -> Radians {
    self.into()
  }
}

impl AddAssign for Degrees {
  fn add_assign(&mut self, rhs: Self) {
    self.0 += rhs.value();
  }
}

#[derive(Clone, Copy)]
pub struct Radians(f32);

impl Radians {
  pub const fn new(value: f32) -> Self {
    Self(value)
  }

  pub const fn value(self) -> f32 {
    self.0
  }

  pub fn clamp(self) -> Self {
    clamp::rotation(self)
  }
}

impl AddAssign for Radians {
  fn add_assign(&mut self, rhs: Self) {
    self.0 += rhs.value();
  }
}

impl From<Degrees> for Radians {
  fn from(deg: Degrees) -> Self {
    Self::new(degrees_to_radians(deg.value()))
  }
}
