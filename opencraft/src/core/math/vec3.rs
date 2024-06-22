use crate::core::math;
use derive_more::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

#[derive(
  Clone, Copy, Default, Debug, Neg, Add, Sub, Mul, Div, AddAssign, SubAssign, MulAssign, DivAssign,
)]
pub struct Vec3 {
  x: f32,
  y: f32,
  z: f32,
}

impl Eq for Vec3 {}
impl PartialEq for Vec3 {
  fn eq(&self, other: &Self) -> bool {
    math::nearly_eq(self.x, other.x)
      && math::nearly_eq(self.y, other.y)
      && math::nearly_eq(self.z, other.z)
  }
}

impl Vec3 {
  pub const fn new(x: f32, y: f32, z: f32) -> Self {
    Self { x, y, z }
  }

  pub const fn x(self) -> f32 {
    self.x
  }

  pub const fn y(self) -> f32 {
    self.y
  }

  pub const fn z(self) -> f32 {
    self.z
  }

  pub fn is_norm(self) -> bool {
    math::nearly_eq_tolerance(self.len(), 1.0, 2.0)
  }

  pub fn dot(lhs: Self, rhs: Self) -> f32 {
    (lhs.x() * rhs.x()) + (lhs.y() * rhs.y()) + (lhs.z() * rhs.z())
  }

  pub fn cross(lhs: Self, rhs: Self) -> Self {
    Self::new(
      (lhs.y() * rhs.z()) - (lhs.z() * rhs.y()),
      (lhs.z() * rhs.x()) - (lhs.x() * rhs.z()),
      (lhs.x() * rhs.y()) - (lhs.y() * rhs.x()),
    )
  }

  pub fn len(self) -> f32 {
    self.len_sq().sqrt()
  }

  pub fn len_sq(self) -> f32 {
    Self::dot(self, self)
  }

  pub fn norm(self) -> Self {
    assert!(self.len_sq() > 0.0, "cannot normalize the zero vector");

    let len = self.len();
    Self::new(self.x() / len, self.y() / len, self.z() / len)
  }
}

impl std::ops::Mul<Vec3> for f32 {
  type Output = Vec3;

  fn mul(self, rhs: Vec3) -> Self::Output {
    rhs * self
  }
}

impl std::ops::Div<Vec3> for f32 {
  type Output = Vec3;

  fn div(self, rhs: Vec3) -> Self::Output {
    rhs / self
  }
}
