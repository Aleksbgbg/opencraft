use crate::core::math::angle::Angle;
use crate::core::math::rotor3::Rotor3;
use crate::core::math::vec3::Vec3;
use crate::core::math::{X_AXIS, Y_AXIS, Z_AXIS};
use bytemuck::NoUninit;
use std::ops::{Index, IndexMut, Mul};

// Shaders are column-major
type Column = [f32; 4];

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, NoUninit)]
pub struct Mat4x4 {
  values: [Column; 4],
}

impl Mat4x4 {
  fn identity() -> Self {
    let mut mat = Self::default();
    mat[(0, 0)] = 1.0;
    mat[(1, 1)] = 1.0;
    mat[(2, 2)] = 1.0;
    mat[(3, 3)] = 1.0;
    mat
  }
}

impl Index<(usize, usize)> for Mat4x4 {
  type Output = f32;

  fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
    &self.values[x][y]
  }
}

impl IndexMut<(usize, usize)> for Mat4x4 {
  fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
    &mut self.values[x][y]
  }
}

impl Mul<Mat4x4> for Mat4x4 {
  type Output = Mat4x4;

  fn mul(self, rhs: Mat4x4) -> Self::Output {
    let mut result = Mat4x4::default();
    for col in 0..4 {
      for row in 0..4 {
        let mut sum = 0.0;
        for op in 0..4 {
          sum += self[(op, row)] * rhs[(col, op)];
        }
        result[(col, row)] = sum;
      }
    }
    result
  }
}

pub fn perspective<A: Angle>(width: f32, height: f32, fov: A, z_near: f32, z_far: f32) -> Mat4x4 {
  let aspect_ratio = height / width;
  let fov_scale = 1.0 / (fov / 2.0).tan();
  let depth_scale = z_far / (z_far - z_near);

  let mut mat = Mat4x4::default();
  mat[(0, 0)] = aspect_ratio * fov_scale;
  mat[(1, 1)] = fov_scale;
  mat[(2, 2)] = depth_scale;
  mat[(2, 3)] = 1.0;
  mat[(3, 2)] = -z_near * depth_scale;
  mat
}

pub fn translate(offset: Vec3) -> Mat4x4 {
  let mut mat = Mat4x4::identity();
  mat[(3, 0)] = offset.x();
  mat[(3, 1)] = offset.y();
  mat[(3, 2)] = offset.z();
  mat
}

pub fn rotate(rotor: Rotor3) -> Mat4x4 {
  let basis_x = rotor.rotate(X_AXIS);
  let basis_y = rotor.rotate(Y_AXIS);
  let basis_z = rotor.rotate(Z_AXIS);

  let mut mat = Mat4x4::default();
  mat[(0, 0)] = basis_x.x();
  mat[(0, 1)] = basis_x.y();
  mat[(0, 2)] = basis_x.z();
  mat[(1, 0)] = basis_y.x();
  mat[(1, 1)] = basis_y.y();
  mat[(1, 2)] = basis_y.z();
  mat[(2, 0)] = basis_z.x();
  mat[(2, 1)] = basis_z.y();
  mat[(2, 2)] = basis_z.z();
  mat[(3, 3)] = 1.0;
  mat
}
