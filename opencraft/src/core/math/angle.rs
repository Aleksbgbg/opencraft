use crate::core::math::clamp;
use derive_more::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use std::f32::consts::PI;

fn degrees_to_radians(degrees: f32) -> f32 {
  degrees * (PI / 180.0)
}

pub trait Angle:
  Copy
  + Into<Radians>
  + std::ops::Neg<Output = Self>
  + std::ops::Add<Output = Self>
  + std::ops::Sub<Output = Self>
  + std::ops::Mul<f32, Output = Self>
  + std::ops::Div<f32, Output = Self>
  + std::ops::AddAssign
  + std::ops::SubAssign
  + std::ops::MulAssign<f32>
  + std::ops::DivAssign<f32>
{
  fn sin(self) -> f32 {
    self.into().value().sin()
  }

  fn cos(self) -> f32 {
    self.into().value().cos()
  }

  fn tan(self) -> f32 {
    self.into().value().tan()
  }
}

#[derive(Clone, Copy, Neg, Add, Sub, Mul, Div, AddAssign, SubAssign, MulAssign, DivAssign)]
pub struct Degrees(f32);

impl Degrees {
  pub const fn new(value: f32) -> Self {
    Self(value)
  }

  const fn value(self) -> f32 {
    self.0
  }

  pub fn clamp(self) -> Self {
    Self::new(clamp::end(self.value(), ..=360.0))
  }
}

impl Angle for Degrees {}

#[derive(Clone, Copy, Neg, Add, Sub, Mul, Div, AddAssign, SubAssign, MulAssign, DivAssign)]
pub struct Radians(f32);

impl Radians {
  pub const fn new(value: f32) -> Self {
    Self(value)
  }

  const fn value(self) -> f32 {
    self.0
  }
}

impl Angle for Radians {}

impl From<Degrees> for Radians {
  fn from(deg: Degrees) -> Self {
    Self::new(degrees_to_radians(deg.value()))
  }
}
