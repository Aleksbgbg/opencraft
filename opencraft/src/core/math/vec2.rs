use derive_more::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use zerocopy::{Immutable, IntoBytes};

#[repr(C)]
#[derive(
  Debug,
  Default,
  Clone,
  Copy,
  Neg,
  Add,
  Sub,
  Mul,
  Div,
  AddAssign,
  SubAssign,
  MulAssign,
  DivAssign,
  Immutable,
  IntoBytes,
)]
pub struct Vec2 {
  x: f32,
  y: f32,
}

impl Vec2 {
  pub const fn new(x: f32, y: f32) -> Self {
    Self { x, y }
  }

  pub const fn x(self) -> f32 {
    self.x
  }

  pub const fn y(self) -> f32 {
    self.y
  }

  pub fn normalise_components_to(self, rhs: Self) -> Self {
    Self::new(self.x() / rhs.x(), self.y() / rhs.y())
  }
}

impl std::ops::Mul<Vec2> for f32 {
  type Output = Vec2;

  fn mul(self, rhs: Vec2) -> Self::Output {
    rhs * self
  }
}

impl std::ops::Div<Vec2> for f32 {
  type Output = Vec2;

  fn div(self, rhs: Vec2) -> Self::Output {
    rhs / self
  }
}
