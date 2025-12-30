use serde::Deserialize;
use zerocopy::{FromBytes, Immutable, IntoBytes};

pub mod block {
  pub const PANE_DIMENSION_PX: usize = 16;
  pub const PANE_PIXELS: usize = PANE_DIMENSION_PX * PANE_DIMENSION_PX;

  pub const PANE_DIMENSION_PX_U32: u32 = PANE_DIMENSION_PX as u32;

  pub const PANE_DIMENSION_PX_F32: f32 = PANE_DIMENSION_PX as f32;
}

pub fn offset_2d((x, y): (usize, usize), width: usize) -> usize {
  (y * width) + x
}

pub fn transfer_block(
  dst: &mut [Srgba],
  (dst_offset_x, dst_offset_y): (usize, usize),
  dst_width: usize,
  src: &[Srgba],
  (src_offset_x, src_offset_y): (usize, usize),
  src_width: usize,
  lines: usize,
) {
  for line in 0..lines {
    let dst_start = offset_2d((dst_offset_x, dst_offset_y + line), dst_width);
    let src_start = offset_2d((src_offset_x, src_offset_y + line), src_width);

    dst[dst_start..dst_start + src_width].copy_from_slice(&src[src_start..src_start + src_width]);
  }
}

#[repr(C)]
#[derive(Default, Clone, Copy, Immutable, FromBytes, IntoBytes)]
pub struct Srgba {
  r: u8,
  g: u8,
  b: u8,
  a: u8,
}

#[repr(C)]
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, IntoBytes, Deserialize)]
pub struct Srgb {
  r: u8,
  g: u8,
  b: u8,
}

impl From<Srgb> for Srgba {
  fn from(value: Srgb) -> Self {
    Self {
      r: value.r,
      g: value.g,
      b: value.b,
      a: 255,
    }
  }
}
