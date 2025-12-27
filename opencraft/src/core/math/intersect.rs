use crate::core::math;
use crate::core::math::aligned_box3::AlignedBox3;
use crate::core::math::frustum3::Frustum3;
use crate::core::math::segment3::Segment3;
use crate::core::math::vec3::Vec3;
use crate::core::math::{Direction, X_AXIS, Y_AXIS, Z_AXIS};
use strum::{EnumCount, EnumIter, IntoEnumIterator};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumCount, EnumIter)]
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
    Into::<Direction>::into(self).normal()
  }

  pub fn opposite(self) -> Self {
    match self {
      BoxFace::XPos => BoxFace::XNeg,
      BoxFace::XNeg => BoxFace::XPos,
      BoxFace::YPos => BoxFace::YNeg,
      BoxFace::YNeg => BoxFace::YPos,
      BoxFace::ZPos => BoxFace::ZNeg,
      BoxFace::ZNeg => BoxFace::ZPos,
    }
  }
}

impl From<BoxFace> for Direction {
  fn from(value: BoxFace) -> Self {
    match value {
      BoxFace::XPos => Direction::XPos,
      BoxFace::XNeg => Direction::XNeg,
      BoxFace::YPos => Direction::YPos,
      BoxFace::YNeg => Direction::YNeg,
      BoxFace::ZPos => Direction::ZPos,
      BoxFace::ZNeg => Direction::ZNeg,
    }
  }
}

impl From<Direction> for BoxFace {
  fn from(value: Direction) -> Self {
    match value {
      Direction::XPos => BoxFace::XPos,
      Direction::XNeg => BoxFace::XNeg,
      Direction::YPos => BoxFace::YPos,
      Direction::YNeg => BoxFace::YNeg,
      Direction::ZPos => BoxFace::ZPos,
      Direction::ZNeg => BoxFace::ZNeg,
    }
  }
}

impl AlignedBox3 {
  pub fn find_intersecting_face(&self, segment: &Segment3) -> Option<BoxFace> {
    for face in BoxFace::iter() {
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

pub trait Intersects<P> {
  fn intersects(&self, polytope: &P) -> bool;
}

impl Intersects<Segment3> for AlignedBox3 {
  fn intersects(&self, segment: &Segment3) -> bool {
    let delta = segment.origin() - self.origin();

    let direction_cross_delta = Vec3::cross(segment.direction(), delta);

    let direction_dot_x_abs = Vec3::dot(segment.direction(), X_AXIS).abs();
    let direction_dot_y_abs = Vec3::dot(segment.direction(), Y_AXIS).abs();
    let direction_dot_z_abs = Vec3::dot(segment.direction(), Z_AXIS).abs();

    let box_extent = self.extent();

    (Vec3::dot(direction_cross_delta, X_AXIS).abs()
      <= ((box_extent.y() * direction_dot_z_abs) + (box_extent.z() * direction_dot_y_abs)))
      && (Vec3::dot(direction_cross_delta, Y_AXIS).abs()
        <= ((box_extent.x() * direction_dot_z_abs) + (box_extent.z() * direction_dot_x_abs)))
      && (Vec3::dot(direction_cross_delta, Z_AXIS).abs()
        <= ((box_extent.x() * direction_dot_y_abs) + (box_extent.y() * direction_dot_x_abs)))
      && (Vec3::dot(delta, X_AXIS).abs()
        <= (box_extent.x() + (segment.extent() * direction_dot_x_abs)))
      && (Vec3::dot(delta, Y_AXIS).abs()
        <= (box_extent.y() + (segment.extent() * direction_dot_y_abs)))
      && (Vec3::dot(delta, Z_AXIS).abs()
        <= (box_extent.z() + (segment.extent() * direction_dot_z_abs)))
  }
}

impl Intersects<AlignedBox3> for Frustum3 {
  fn intersects(&self, aligned_box: &AlignedBox3) -> bool {
    let zx = self.extent_z_near() * self.axis_x();
    let xz = self.extent_x_near() * self.axis_z();
    let zy = self.extent_z_near() * self.axis_y();
    let yz = self.extent_y_near() * self.axis_z();
    let frustum_face_normals = [
      // forward
      self.axis_z(),
      // left, right
      zx - xz,
      -zx - xz,
      // top, bottom
      zy - yz,
      -zy - yz,
    ];

    let box_face_normals = [X_AXIS, Y_AXIS, Z_AXIS];

    let xx = self.extent_x_near() * self.axis_x();
    let yy = self.extent_y_near() * self.axis_y();
    let zz = self.extent_z_near() * self.axis_z();
    let diagonal_edge_0 = xx + yy + zz;
    let diagonal_edge_1 = -xx + yy + zz;
    let diagonal_edge_2 = xx - yy + zz;
    let diagonal_edge_3 = -xx - yy + zz;
    let edge_cross_products = [
      // Frustum horizontal edges with box edges
      Vec3::cross(self.axis_x(), X_AXIS),
      Vec3::cross(self.axis_x(), Y_AXIS),
      Vec3::cross(self.axis_x(), Z_AXIS),
      // Frustum vertical edges with box edges
      Vec3::cross(self.axis_y(), X_AXIS),
      Vec3::cross(self.axis_y(), Y_AXIS),
      Vec3::cross(self.axis_y(), Z_AXIS),
      // Frustum diagonal edges with box edges
      // X
      Vec3::cross(diagonal_edge_0, X_AXIS),
      Vec3::cross(diagonal_edge_1, X_AXIS),
      Vec3::cross(diagonal_edge_2, X_AXIS),
      Vec3::cross(diagonal_edge_3, X_AXIS),
      // Y
      Vec3::cross(diagonal_edge_0, Y_AXIS),
      Vec3::cross(diagonal_edge_1, Y_AXIS),
      Vec3::cross(diagonal_edge_2, Y_AXIS),
      Vec3::cross(diagonal_edge_3, Y_AXIS),
      // Z
      Vec3::cross(diagonal_edge_0, Z_AXIS),
      Vec3::cross(diagonal_edge_1, Z_AXIS),
      Vec3::cross(diagonal_edge_2, Z_AXIS),
      Vec3::cross(diagonal_edge_3, Z_AXIS),
    ];

    let separating_axes = frustum_face_normals
      .into_iter()
      .chain(box_face_normals)
      .chain(
        edge_cross_products
          .into_iter()
          .filter(|product| product.len_sq() != 0.0),
      );

    let box_origin = aligned_box.origin() - self.origin();
    let box_extent = aligned_box.extent();

    for direction in separating_axes {
      let box_origin_proj = Vec3::dot(direction, box_origin);
      let box_extent_proj = (box_extent.x() * Vec3::dot(direction, X_AXIS).abs())
        + (box_extent.y() * Vec3::dot(direction, Y_AXIS).abs())
        + (box_extent.z() * Vec3::dot(direction, Z_AXIS).abs());

      let box_min_proj = box_origin_proj - box_extent_proj;
      let box_max_proj = box_origin_proj + box_extent_proj;

      let frustum_z_proj = self.extent_z_near() * Vec3::dot(direction, self.axis_z());
      let frustum_xy_proj = (self.extent_x_near() * Vec3::dot(direction, self.axis_x()).abs())
        + (self.extent_y_near() * Vec3::dot(direction, self.axis_y()).abs());

      let mut frustum_min_proj = frustum_z_proj - frustum_xy_proj;
      let mut frustum_max_proj = frustum_z_proj + frustum_xy_proj;
      if frustum_min_proj < 0.0 {
        frustum_min_proj *= self.depth_ratio();
      }
      if frustum_max_proj > 0.0 {
        frustum_max_proj *= self.depth_ratio();
      }

      if (box_max_proj < frustum_min_proj) || (frustum_max_proj < box_min_proj) {
        return false;
      }
    }

    true
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
