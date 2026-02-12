use zerocopy::{Immutable, IntoBytes};

#[repr(C)]
#[derive(Clone, Copy, Immutable, IntoBytes)]
pub struct Vertex {
  pub position: [f32; 3],
  pub texture_coordinate: [f32; 2],
}
