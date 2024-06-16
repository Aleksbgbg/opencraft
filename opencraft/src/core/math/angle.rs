use crate::core::math::{self, FULL_ROTATION};
use derive_more::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use std::f32::consts::PI;
use std::fmt::Debug;

fn degrees_to_radians(degrees: f32) -> f32 {
  degrees * (PI / 180.0)
}

#[derive(
  Clone, Copy, Default, Debug, Neg, Add, Sub, Mul, Div, AddAssign, SubAssign, MulAssign, DivAssign,
)]
pub struct Angle {
  radians: f32,
}

impl Eq for Angle {}
impl PartialEq for Angle {
  fn eq(&self, other: &Self) -> bool {
    math::nearly_eq(self.radians, other.radians)
  }
}

impl Angle {
  pub const fn radians(radians: f32) -> Self {
    Self { radians }
  }

  pub fn degrees(degrees: f32) -> Self {
    Self::radians(degrees_to_radians(degrees))
  }

  pub fn sin(self) -> f32 {
    self.radians.sin()
  }

  pub fn cos(self) -> f32 {
    self.radians.cos()
  }

  pub fn tan(self) -> f32 {
    self.radians.tan()
  }

  pub fn wrap(self) -> Self {
    Self::radians(self.radians.rem_euclid(FULL_ROTATION.radians))
  }
}
