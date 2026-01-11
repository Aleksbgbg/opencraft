use crate::core::math;
use crate::core::type_conversions::{Coerce, CoerceLossy, CoerceLossyCeil};
use base::texture::block::PANE_DIMENSION_PX;
use base::texture::{self, Srgba};
use std::f32::consts::E;
use std::ops::{Add, AddAssign, Div, DivAssign};

fn offset_index_reflecting(index: usize, offset: usize, size: usize) -> usize {
  let mut index: isize = index.coerce();
  let offset: isize = offset.coerce();
  let size: isize = size.coerce();

  index -= offset;

  while (index < 0) || (index >= size) {
    if index < 0 {
      index = index.abs();
    } else if index >= size {
      let difference = index - size;
      index = size - difference - 1;
    }
  }

  index.coerce()
}

struct Image<'a, T> {
  rgba: &'a [T],
  width: usize,
  height: usize,
}

impl<'a, T> Image<'a, T>
where
  T: Clone + Copy,
{
  fn get(&self, (x, y): (usize, usize)) -> T {
    self.rgba[texture::offset_2d((x, y), self.width)]
  }
}

struct ImageMut<'a, T> {
  rgba: &'a mut [T],
  width: usize,
  #[allow(dead_code)]
  height: usize,
}

impl<'a, T> ImageMut<'a, T>
where
  T: Clone + Copy,
{
  fn set(&mut self, (x, y): (usize, usize), sample: T) {
    self.rgba[texture::offset_2d((x, y), self.width)] = sample;
  }
}

pub struct Mipmap {
  pub rgba: Vec<Srgba>,
  pub mip_levels: usize,
}

pub fn generate(base_rgba: &[Srgba], width: usize, height: usize) -> Mipmap {
  const MIP_LEVEL_TEXEL_RATIO: usize = 4;

  const GAUSSIAN_STD_DEV: f32 = 3.0;
  const GAUSSIAN_WIDTH: f32 = 3.0 * GAUSSIAN_STD_DEV;

  let mip_chain_length: usize = width.min(height).ilog2().coerce();
  let mip_levels = mip_chain_length + 1;

  let largest_dimension_smallest_size = width.max(height) / 2_usize.pow(mip_chain_length.coerce());
  let mipmap_texels = math::geometric_sequence_sum(
    largest_dimension_smallest_size,
    MIP_LEVEL_TEXEL_RATIO,
    mip_levels,
  );

  let mut rgba = vec![Srgba::default(); mipmap_texels];
  rgba[..base_rgba.len()].copy_from_slice(base_rgba);

  let base_rgba: Vec<RgbF32> = base_rgba.iter().map(|&texel| texel.into()).collect();
  let blocks = width / PANE_DIMENSION_PX;
  let kernel = FilterKernel::new(generate_gaussian_filter(GAUSSIAN_STD_DEV, GAUSSIAN_WIDTH));
  let mut current_width = width;
  let mut current_height = height;
  let mut current_block_size = PANE_DIMENSION_PX;
  let mut offset = 0;
  for _ in 1..mip_levels {
    let previous_width = current_width;
    let previous_height = current_height;

    current_width /= 2;
    current_height /= 2;
    current_block_size /= 2;

    let previous_texels = previous_width * previous_height;
    let current_texels = current_width * current_height;

    offset += previous_texels;

    for block_index in 0..blocks {
      generate_mip_level_block(
        ImageMut {
          rgba: &mut rgba[offset..offset + current_texels],
          width: current_width,
          height: current_height,
        },
        Image {
          rgba: &base_rgba,
          width,
          height,
        },
        &kernel,
        current_block_size,
        PANE_DIMENSION_PX,
        block_index,
      );
    }
  }

  Mipmap { rgba, mip_levels }
}

fn generate_mip_level_block(
  mut dst: ImageMut<'_, Srgba>,
  src: Image<'_, RgbF32>,
  kernel: &FilterKernel,
  dst_block_size: usize,
  src_block_size: usize,
  block_index: usize,
) {
  let mut tmp_texels = vec![RgbF32::default(); src_block_size * src_block_size];

  let step = src_block_size / dst_block_size;
  let src_offset_x = block_index * src_block_size;

  for src_y in 0..src_block_size {
    for src_x in (0..src_block_size).step_by(step) {
      let mut sum = RgbF32::default();

      for (index, sample) in kernel.iter().enumerate() {
        let src_x =
          offset_index_reflecting(src_offset_x + src_x + index, kernel.offset(), src.width);
        let texel = src.get((src_x, src_y));

        sum += texel.map(|component| component * sample);
      }

      let tmp_x = src_x / step;
      let tmp_y = src_y;

      tmp_texels[texture::offset_2d((tmp_x, tmp_y), src_block_size)] = sum;
    }
  }

  let dst_offset_x = block_index * dst_block_size;

  for src_x in 0..(src_block_size / step) {
    for src_y in (0..src_block_size).step_by(2) {
      let mut sum = RgbF32::default();

      for (index, sample) in kernel.iter().enumerate() {
        let src_y = offset_index_reflecting(src_y + index, kernel.offset(), src.height);
        let texel = src.get((src_offset_x + src_x, src_y));

        sum += texel.map(|component| component * sample);
      }

      let dst_x = dst_offset_x + src_x;
      let dst_y = src_y / step;

      dst.set((dst_x, dst_y), sum.into());
    }
  }
}

struct FilterKernel {
  values: Vec<f32>,
  offset: usize,
}

impl FilterKernel {
  fn new(filter: impl Iterator<Item = f32>) -> Self {
    let mut values: Vec<f32> = filter.collect();
    let sum: f32 = values.iter().sum();

    for value in &mut values {
      *value /= sum;
    }

    let offset = (values.len() / 2) - 1;
    Self { values, offset }
  }

  fn iter(&self) -> impl Iterator<Item = f32> {
    self.values.iter().copied()
  }

  fn offset(&self) -> usize {
    self.offset
  }
}

fn generate_gaussian_filter(std_dev: f32, width: f32) -> impl Iterator<Item = f32> {
  let half_width = width / 2.0;
  let offset = -half_width;

  (0_usize..width.coerce_lossy_ceil()).map(move |index| {
    let index_f32: f32 = index.coerce_lossy();
    let position = index_f32 + offset;
    E.powf(-((position * position) / (2.0 * std_dev * std_dev)))
  })
}

#[derive(Clone, Copy, Default)]
struct Rgb<T> {
  r: T,
  g: T,
  b: T,
}

type RgbF32 = Rgb<f32>;

impl RgbF32 {
  fn map(self, function: impl Fn(f32) -> f32) -> Self {
    Self {
      r: function(self.r),
      g: function(self.g),
      b: function(self.b),
    }
  }
}

impl Add<Self> for RgbF32 {
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output {
    Self {
      r: self.r + rhs.r,
      g: self.g + rhs.g,
      b: self.b + rhs.b,
    }
  }
}

impl AddAssign<Self> for RgbF32 {
  fn add_assign(&mut self, rhs: Self) {
    *self = *self + rhs;
  }
}

impl Div<f32> for RgbF32 {
  type Output = Self;

  fn div(self, rhs: f32) -> Self::Output {
    Self {
      r: self.r / rhs,
      g: self.g / rhs,
      b: self.b / rhs,
    }
  }
}

impl DivAssign<f32> for RgbF32 {
  fn div_assign(&mut self, rhs: f32) {
    *self = *self / rhs;
  }
}

impl From<Srgba> for RgbF32 {
  fn from(value: Srgba) -> Self {
    Self {
      r: standard_to_linear_rgb(math::normalized_u8_to_f32(value.r)),
      g: standard_to_linear_rgb(math::normalized_u8_to_f32(value.g)),
      b: standard_to_linear_rgb(math::normalized_u8_to_f32(value.b)),
    }
  }
}

impl From<RgbF32> for Srgba {
  fn from(value: RgbF32) -> Self {
    Self {
      r: math::normalized_f32_to_u8(linear_to_standard_rgb(value.r)),
      g: math::normalized_f32_to_u8(linear_to_standard_rgb(value.g)),
      b: math::normalized_f32_to_u8(linear_to_standard_rgb(value.b)),
      a: u8::MAX,
    }
  }
}

fn standard_to_linear_rgb(value: f32) -> f32 {
  if value >= 0.0031308 {
    (1.055 * value.powf(1.0 / 2.4)) - 0.055
  } else {
    12.92 * value
  }
}

fn linear_to_standard_rgb(value: f32) -> f32 {
  if value >= 0.04045 {
    ((value + 0.055) / 1.055).powf(2.4)
  } else {
    value / 12.92
  }
}
