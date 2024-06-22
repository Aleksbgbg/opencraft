use crate::core::math::vec3::Vec3;
use derive_more::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

#[derive(
  Clone, Copy, Default, Debug, Neg, Add, Sub, Mul, Div, AddAssign, SubAssign, MulAssign, DivAssign,
)]
pub struct BiVec3 {
  xy: f32,
  yz: f32,
  zx: f32,
}

impl BiVec3 {
  pub const fn new(xy: f32, yz: f32, zx: f32) -> Self {
    Self { xy, yz, zx }
  }

  pub const fn xy(self) -> f32 {
    self.xy
  }

  pub const fn yz(self) -> f32 {
    self.yz
  }

  pub const fn zx(self) -> f32 {
    self.zx
  }

  pub fn wedge(lhs: impl Into<Self>, rhs: impl Into<Self>) -> Self {
    let lhs = lhs.into();
    let rhs = rhs.into();

    Self::new(
      (lhs.xy() * rhs.yz()) - (lhs.yz() * rhs.xy()),
      (lhs.yz() * rhs.zx()) - (lhs.zx() * rhs.yz()),
      (lhs.zx() * rhs.xy()) - (lhs.xy() * rhs.zx()),
    )
  }
}

impl From<Vec3> for BiVec3 {
  fn from(value: Vec3) -> Self {
    Self::new(value.x(), value.y(), value.z())
  }
}
