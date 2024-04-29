use crate::core::math::clamp;
use derive_more::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use std::f32::consts::PI;

fn degrees_to_radians(degrees: f32) -> f32 {
  degrees * (PI / 180.0)
}

#[derive(Clone, Copy, Neg, Add, Sub, Mul, Div, AddAssign, SubAssign, MulAssign, DivAssign)]
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

#[derive(Clone, Copy, Neg, Add, Sub, Mul, Div, AddAssign, SubAssign, MulAssign, DivAssign)]
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

impl From<Degrees> for Radians {
  fn from(deg: Degrees) -> Self {
    Self::new(degrees_to_radians(deg.value()))
  }
}
