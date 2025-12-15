use crate::core::math::angle::Angle;
use crate::core::math::bivec3::BiVec3;
use crate::core::math::vec3::Vec3;
use std::ops::{Mul, Neg};

#[derive(Debug, Clone, Copy)]
pub struct Rotor3 {
  scalar: f32,
  xy: f32,
  yz: f32,
  zx: f32,
}

impl Rotor3 {
  pub fn angle_plane(angle: Angle, plane: BiVec3) -> Self {
    let half = angle / 2.0;
    let neg_sin = -half.sin();

    Self {
      scalar: half.cos(),
      xy: neg_sin * plane.xy(),
      yz: neg_sin * plane.yz(),
      zx: neg_sin * plane.zx(),
    }
  }

  pub fn from_to(from: Vec3, to: Vec3) -> Self {
    assert!(
      from.is_norm() && to.is_norm(),
      "input vectors must be normalized (from.len() = {}, to.len() = {})",
      from.len(),
      to.len(),
    );

    assert!(
      from != -to,
      "180° rotation is not well defined (from = {:?}, to = {:?})",
      from,
      to
    );

    let halfway = (from + to).norm();
    let dot = Vec3::dot(halfway, from);
    let wedge = BiVec3::wedge(halfway, from);

    Self {
      scalar: dot,
      xy: wedge.xy(),
      yz: wedge.yz(),
      zx: wedge.zx(),
    }
  }

  pub fn rotate(self, vec: Vec3) -> Vec3 {
    let s_x = (self.scalar * vec.x()) + (self.xy * vec.y()) - (self.zx * vec.z());
    let s_y = (self.scalar * vec.y()) - (self.xy * vec.x()) + (self.yz * vec.z());
    let s_z = (self.scalar * vec.z()) - (self.yz * vec.y()) + (self.zx * vec.x());
    let s_xyz = (self.xy * vec.z()) + (self.yz * vec.x()) + (self.zx * vec.y());

    Vec3::new(
      (s_x * self.scalar) + (s_y * self.xy) - (s_z * self.zx) + (s_xyz * self.yz),
      -(s_x * self.xy) + (s_y * self.scalar) + (s_z * self.yz) + (s_xyz * self.zx),
      (s_x * self.zx) - (s_y * self.yz) + (s_z * self.scalar) + (s_xyz * self.xy),
    )
  }
}

impl Neg for Rotor3 {
  type Output = Self;

  fn neg(self) -> Self::Output {
    Self {
      scalar: self.scalar,
      xy: -self.xy,
      yz: -self.yz,
      zx: -self.zx,
    }
  }
}

impl Mul<Rotor3> for Rotor3 {
  type Output = Self;

  fn mul(self, rhs: Rotor3) -> Self::Output {
    Self {
      scalar: (self.scalar * rhs.scalar)
        - (self.xy * rhs.xy)
        - (self.yz * rhs.yz)
        - (self.zx * rhs.zx),
      xy: (self.scalar * rhs.xy) + (self.xy * rhs.scalar) - (self.yz * rhs.zx) + (self.zx * rhs.yz),
      yz: (self.scalar * rhs.yz) + (self.xy * rhs.zx) + (self.yz * rhs.scalar) - (self.zx * rhs.xy),
      zx: (self.scalar * rhs.zx) - (self.xy * rhs.yz) + (self.yz * rhs.xy) + (self.zx * rhs.scalar),
    }
  }
}
