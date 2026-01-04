use crate::core::math;
use crate::core::math::aligned_box3::AlignedBox3;
use crate::core::math::segment3::Segment3;
use crate::core::math::vec3::Vec3;
use crate::core::math::{X_AXIS, Y_AXIS, Z_AXIS};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoxFace {
  XPos,
  XNeg,
  YPos,
  YNeg,
  ZPos,
  ZNeg,
}

impl BoxFace {
  pub fn normal(self) -> Vec3 {
    match self {
      BoxFace::XPos => X_AXIS,
      BoxFace::XNeg => -X_AXIS,
      BoxFace::YPos => Y_AXIS,
      BoxFace::YNeg => -Y_AXIS,
      BoxFace::ZPos => Z_AXIS,
      BoxFace::ZNeg => -Z_AXIS,
    }
  }
}

impl AlignedBox3 {
  pub fn find_intersecting_face(&self, segment: &Segment3) -> Option<BoxFace> {
    const FACES: [BoxFace; 6] = [
      BoxFace::XPos,
      BoxFace::XNeg,
      BoxFace::YPos,
      BoxFace::YNeg,
      BoxFace::ZPos,
      BoxFace::ZNeg,
    ];

    for face in FACES {
      let normal = face.normal();
      let direction_match = Vec3::dot(segment.direction(), normal);

      if direction_match >= 0.0 {
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
      BoxFace::XPos | BoxFace::XNeg => (X_AXIS, Y_AXIS, Z_AXIS),
      BoxFace::YPos | BoxFace::YNeg => (Y_AXIS, Z_AXIS, X_AXIS),
      BoxFace::ZPos | BoxFace::ZNeg => (Z_AXIS, X_AXIS, Y_AXIS),
    };

    let start = segment.start();
    let end = segment.end();
    let start_a0 = Vec3::dot(start, axis_0);
    let end_a0 = Vec3::dot(end, axis_0);

    let extent_a0 = Vec3::dot(self.extent(), axis_0);
    let face_center = self.origin() + (face.normal() * extent_a0);
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
    let extent_a1 = Vec3::dot(self.extent(), axis_1);

    let p_a2 = Vec3::dot(p, axis_2);
    let face_center_a2 = Vec3::dot(face_center, axis_2);
    let extent_a2 = Vec3::dot(self.extent(), axis_2);

    math::in_range(p_a1, face_center_a1 - extent_a1, face_center_a1 + extent_a1)
      && math::in_range(p_a2, face_center_a2 - extent_a2, face_center_a2 + extent_a2)
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

    assert_eq!(Some(BoxFace::XNeg), face);
  }
}
