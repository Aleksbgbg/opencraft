use crate::core::math::angle::Angle;
use crate::core::math::{self, X_AXIS, Y_AXIS, Z_AXIS};
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

  pub fn wedge(lhs: Self, rhs: Self) -> Self {
    Self::new(
      (lhs.x() * rhs.y()) - (lhs.y() * rhs.x()),
      (lhs.y() * rhs.z()) - (lhs.z() * rhs.y()),
      (lhs.z() * rhs.x()) - (lhs.x() * rhs.z()),
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

  pub fn perpendicular(self) -> Self {
    let most_orthogonal_axis = [X_AXIS, Y_AXIS, Z_AXIS]
      .into_iter()
      .reduce(|a, b| {
        if Self::dot(self, a) < Self::dot(self, b) {
          a
        } else {
          b
        }
      })
      .unwrap();

    Self::cross(most_orthogonal_axis, self)
  }

  pub fn angle_axis_rotate(self, angle: Angle, axis: Vec3) -> Self {
    let proj = axis * (Self::dot(self, axis) / Self::dot(axis, axis));
    let rej = self - proj;
    let rej_len = rej.len();
    let orthogonal = Self::cross(axis, rej);
    let x_1 = angle.cos() / rej_len;
    let x_2 = angle.sin() / orthogonal.len();

    proj + (rej_len * ((x_1 * rej) + (x_2 * orthogonal)))
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

#[cfg(test)]
mod tests {
  use crate::core::math::{QUARTER_ROTATION, X_AXIS, Y_AXIS, Z_AXIS};

  #[test]
  fn rotate_x_on_y() {
    assert_eq!(-Z_AXIS, X_AXIS.angle_axis_rotate(QUARTER_ROTATION, Y_AXIS));
  }

  #[test]
  fn rotate_x_on_z() {
    assert_eq!(Y_AXIS, X_AXIS.angle_axis_rotate(QUARTER_ROTATION, Z_AXIS));
  }

  #[test]
  fn rotate_y_on_x() {
    assert_eq!(Z_AXIS, Y_AXIS.angle_axis_rotate(QUARTER_ROTATION, X_AXIS));
  }

  #[test]
  fn rotate_y_on_z() {
    assert_eq!(-X_AXIS, Y_AXIS.angle_axis_rotate(QUARTER_ROTATION, Z_AXIS));
  }

  #[test]
  fn rotate_z_on_x() {
    assert_eq!(-Y_AXIS, Z_AXIS.angle_axis_rotate(QUARTER_ROTATION, X_AXIS));
  }

  #[test]
  fn rotate_z_on_y() {
    assert_eq!(X_AXIS, Z_AXIS.angle_axis_rotate(QUARTER_ROTATION, Y_AXIS));
  }
}
