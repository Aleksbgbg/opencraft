use crate::core::math::vec2::Vec2;
use crate::core::math::vec3::Vec3;
use zerocopy::{Immutable, IntoBytes};

#[repr(C)]
#[derive(Clone, Copy, Immutable, IntoBytes)]
pub struct Vertex {
  pub position: Vec3,
}

#[repr(C)]
#[derive(Default, Clone, Copy, Immutable, IntoBytes)]
pub struct BlockVertex {
  pub position: Vec3,
  pub texture_coordinate: Vec2,
  pub line_coordinates: Vec2,
  pub block_index: u32,
}
