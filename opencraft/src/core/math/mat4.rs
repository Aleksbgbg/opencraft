use crate::core::math::angle::Radians;
use bytemuck::NoUninit;
use std::ops::{Index, IndexMut, Mul};

// Shaders are column-major
type Column = [f32; 4];

#[repr(C)]
#[derive(Clone, Copy, Default, NoUninit)]
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

pub fn perspective<A>(width: f32, height: f32, fov: A, z_near: f32, z_far: f32) -> Mat4x4
where
  A: Into<Radians>,
{
  let aspect_ratio = height / width;
  let fov_scale = 1.0 / (fov.into().value() / 2.0).tan();
  let depth_scale = z_far / (z_far - z_near);

  let mut mat = Mat4x4::default();
  mat[(0, 0)] = aspect_ratio * fov_scale;
  mat[(1, 1)] = fov_scale;
  mat[(2, 2)] = depth_scale;
  mat[(2, 3)] = 1.0;
  mat[(3, 2)] = -z_near * depth_scale;
  mat
}

pub fn translate((x, y, z): (f32, f32, f32)) -> Mat4x4 {
  let mut mat = Mat4x4::identity();
  mat[(3, 0)] = x;
  mat[(3, 1)] = y;
  mat[(3, 2)] = z;
  mat
}

pub fn rotate<A>(angle: A) -> Mat4x4
where
  A: Into<Radians>,
{
  let angle = angle.into().value();

  let mut mat = Mat4x4::default();
  mat[(0, 0)] = angle.cos();
  mat[(0, 2)] = -angle.sin();
  mat[(1, 1)] = 1.0;
  mat[(2, 0)] = angle.sin();
  mat[(2, 2)] = angle.cos();
  mat[(3, 3)] = 1.0;
  mat
}
