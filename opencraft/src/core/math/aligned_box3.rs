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

  pub fn intersect_with(&self, segment: &Segment3) -> Option<BoxFace> {
    const FACES: [BoxFace; 6] = [
      BoxFace::Left,
      BoxFace::Right,
      BoxFace::Top,
      BoxFace::Bottom,
      BoxFace::Back,
      BoxFace::Front,
    ];

    let mut best_face = None;
    let mut best_match = f32::MAX;
    for face in FACES {
      let normal = face.normal();
      let direction_match = Vec3::dot(segment.direction(), normal);

      if direction_match > 0.0 {
        continue;
      }

      if direction_match < best_match
        && segment.intersects_cube_face(
          self.center + (self.extent * normal),
          self.extent,
          match face {
            BoxFace::Left | BoxFace::Right => Vec3::x,
            BoxFace::Top | BoxFace::Bottom => Vec3::y,
            BoxFace::Back | BoxFace::Front => Vec3::z,
          },
          match face {
            BoxFace::Left | BoxFace::Right => Vec3::y,
            BoxFace::Top | BoxFace::Bottom => Vec3::x,
            BoxFace::Back | BoxFace::Front => Vec3::x,
          },
          match face {
            BoxFace::Left | BoxFace::Right => Vec3::z,
            BoxFace::Top | BoxFace::Bottom => Vec3::z,
            BoxFace::Back | BoxFace::Front => Vec3::y,
          },
        )
      {
        best_face = Some(face);
        best_match = direction_match;
      }
    }

    best_face
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

    let face = cube.intersect_with(&segment);

    assert_eq!(Some(BoxFace::Right), face);
  }
}
