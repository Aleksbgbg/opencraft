use crate::core::math;
use crate::core::math::vec3::Vec3;

pub struct Segment3 {
  start: Vec3,
  direction: Vec3,
  len: f32,
}

impl Segment3 {
  pub const fn start_direction_len(start: Vec3, direction: Vec3, len: f32) -> Self {
    Self {
      start,
      direction,
      len,
    }
  }

  pub const fn direction(&self) -> Vec3 {
    self.direction
  }

  fn end(&self) -> Vec3 {
    self.start + (self.direction * self.len)
  }

  pub fn intersects_cube_face(
    &self,
    face_center: Vec3,
    extent_2d: f32,
    axis0: impl Fn(Vec3) -> f32,
    axis1: impl Fn(Vec3) -> f32,
    axis2: impl Fn(Vec3) -> f32,
  ) -> bool {
    let (min, max) = math::min_max(axis0(self.start), axis0(self.end()));
    if !math::in_range(axis0(face_center), min, max) {
      return false;
    }

    let t = (axis0(face_center) - axis0(self.start)) / axis0(self.direction);
    let p = self.start + (t * self.direction);

    math::in_range(
      axis1(p),
      axis1(face_center) - extent_2d,
      axis1(face_center) + extent_2d,
    ) && math::in_range(
      axis2(p),
      axis2(face_center) - extent_2d,
      axis2(face_center) + extent_2d,
    )
  }
}
