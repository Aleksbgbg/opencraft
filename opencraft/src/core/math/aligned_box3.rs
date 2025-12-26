use crate::core::math;
use crate::core::math::segment3::Segment3;
use crate::core::math::vec3::Vec3;
use crate::core::math::{X_AXIS, Y_AXIS, Z_AXIS};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoxFace {
  Left,
  Right,
  Top,
  Bottom,
  Back,
  Front,
}

impl BoxFace {
  pub fn normal(self) -> Vec3 {
    match self {
      BoxFace::Left => X_AXIS,
      BoxFace::Right => -X_AXIS,
      BoxFace::Top => Y_AXIS,
      BoxFace::Bottom => -Y_AXIS,
      BoxFace::Back => Z_AXIS,
      BoxFace::Front => -Z_AXIS,
    }
  }
}

pub struct AlignedBox3 {
  center: Vec3,
  extent: f32,
}

impl AlignedBox3 {
  pub const fn cube(center: Vec3, extent: f32) -> Self {
    Self { center, extent }
  }

  pub fn find_intersecting_face(&self, segment: &Segment3) -> Option<BoxFace> {
    const FACES: [BoxFace; 6] = [
      BoxFace::Left,
      BoxFace::Right,
      BoxFace::Top,
      BoxFace::Bottom,
      BoxFace::Back,
      BoxFace::Front,
    ];

    for face in FACES {
      let normal = face.normal();
      let direction_match = Vec3::dot(segment.direction(), normal);

      if direction_match > 0.0 {
        continue;
      }

      if self.intersects_box_face(face, segment) {
        return Some(face);
      }
    }

    None
  }

  fn intersects_box_face(&self, face: BoxFace, segment: &Segment3) -> bool {
    let (axis_0, axis_1, axis_2) = match face {
      BoxFace::Left | BoxFace::Right => (X_AXIS, Y_AXIS, Z_AXIS),
      BoxFace::Top | BoxFace::Bottom => (Y_AXIS, Z_AXIS, X_AXIS),
      BoxFace::Back | BoxFace::Front => (Z_AXIS, X_AXIS, Y_AXIS),
    };

    let start = segment.start();
    let end = segment.end();
    let start_a0 = Vec3::dot(start, axis_0);
    let end_a0 = Vec3::dot(end, axis_0);

    let face_center = self.center + (face.normal() * self.extent);
    let face_center_a0 = Vec3::dot(face_center, axis_0);

    let (min_a0, max_a0) = math::min_max(start_a0, end_a0);
    if !math::in_range(face_center_a0, min_a0, max_a0) {
      return false;
    }

    let direction = segment.direction();
    let direction_a0 = Vec3::dot(direction, axis_0);

    let t = (face_center_a0 - start_a0) / direction_a0;
    let p = start + (t * direction);

    let p_a1 = Vec3::dot(p, axis_1);
    let face_center_a1 = Vec3::dot(face_center, axis_1);

    let p_a2 = Vec3::dot(p, axis_2);
    let face_center_a2 = Vec3::dot(face_center, axis_2);

    math::in_range(
      p_a1,
      face_center_a1 - self.extent,
      face_center_a1 + self.extent,
    ) && math::in_range(
      p_a2,
      face_center_a2 - self.extent,
      face_center_a2 + self.extent,
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  // Check that an intersection with a face not directly facing the segment start
  // point is correctly detected.
  #[test]
  fn test_intersect_side_face() {
    let cube = AlignedBox3::cube(Vec3::new(0.0, 0.0, 1.0), 0.5);
    let segment = Segment3::start_direction_len(
      Vec3::new(-1.0, 0.0, 0.0),
      Vec3::new(1.0, 0.0, 2.0).norm(),
      5.0,
    );

    let face = cube.find_intersecting_face(&segment);

    assert_eq!(Some(BoxFace::Right), face);
  }
}
