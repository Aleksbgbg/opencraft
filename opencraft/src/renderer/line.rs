use crate::core::math;
use crate::core::type_conversions::{Coerce, CoerceLossy, CoerceLossyFloor};

pub struct LineImage {
  pub alpha: Vec<u8>,
  pub width: u32,
  pub height: u32,
}

pub fn generate_line_image(resolution_px: usize, line_width_px: f32) -> LineImage {
  assert!(line_width_px < resolution_px.coerce_lossy());

  let line_width_integer_px: usize = line_width_px.coerce_lossy_floor();
  let line_width_real_px = line_width_px.fract();

  let fill_start = resolution_px - line_width_integer_px;

  let mut alpha = vec![0; resolution_px];
  alpha[fill_start..resolution_px].fill(u8::MAX);
  alpha[fill_start - 1] = math::normalized_f32_to_u8(line_width_real_px);

  LineImage {
    alpha,
    width: resolution_px.coerce(),
    height: 1,
  }
}
