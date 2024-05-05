use crate::core::math::vec3::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct Rotor3 {
  scalar: f32,
  xy: f32,
  yz: f32,
  zx: f32,
}

impl Rotor3 {
  pub fn new(from: Vec3, to: Vec3) -> Self {
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
    let wedge = Vec3::wedge(halfway, from);

    Self {
      scalar: dot,
      xy: wedge.x(),
      yz: wedge.y(),
      zx: wedge.z(),
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
