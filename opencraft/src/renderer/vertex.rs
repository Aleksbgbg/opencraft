use zerocopy::{Immutable, IntoBytes};

#[repr(C)]
#[derive(Clone, Copy, Immutable, IntoBytes)]
pub struct BlockVertex {
  pub position: [f32; 3],
  pub texture_coordinate: [f32; 2],
}
