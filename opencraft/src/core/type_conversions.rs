use wgpu::BufferAddress;

pub trait Coerce<T> {
  fn coerce(self) -> T;
}

pub trait CoerceLossy<T> {
  fn coerce_lossy(self) -> T;
}

impl Coerce<u32> for usize {
  fn coerce(self) -> u32 {
    self.try_into().unwrap()
  }
}

impl CoerceLossy<f32> for usize {
  fn coerce_lossy(self) -> f32 {
    self as f32
  }
}

impl Coerce<BufferAddress> for usize {
  fn coerce(self) -> BufferAddress {
    self.try_into().unwrap()
  }
}

impl Coerce<usize> for u32 {
  fn coerce(self) -> usize {
    self.try_into().unwrap()
  }
}

impl CoerceLossy<f32> for u32 {
  fn coerce_lossy(self) -> f32 {
    self as f32
  }
}

impl Coerce<usize> for i32 {
  fn coerce(self) -> usize {
    self.try_into().unwrap()
  }
}

impl CoerceLossy<f32> for f64 {
  fn coerce_lossy(self) -> f32 {
    self as f32
  }
}

impl CoerceLossy<usize> for f32 {
  fn coerce_lossy(self) -> usize {
    self as usize
  }
}

impl CoerceLossy<u32> for f32 {
  fn coerce_lossy(self) -> u32 {
    self as u32
  }
}

impl CoerceLossy<u8> for f32 {
  fn coerce_lossy(self) -> u8 {
    self as u8
  }
}
