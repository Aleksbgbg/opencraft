use wgpu::BufferAddress;

pub trait Coerce<T> {
  fn coerce(self) -> T;
}

pub trait CoerceLossy<T> {
  fn coerce_lossy(self) -> T;
}

pub trait CoerceLossyRound<T> {
  fn coerce_lossy_round(self) -> T;
}

pub trait CoerceLossyFloor<T> {
  fn coerce_lossy_floor(self) -> T;
}

pub trait CoerceLossyCeil<T> {
  fn coerce_lossy_ceil(self) -> T;
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

impl CoerceLossyRound<usize> for f32 {
  fn coerce_lossy_round(self) -> usize {
    self.round() as usize
  }
}

impl CoerceLossyFloor<usize> for f32 {
  fn coerce_lossy_floor(self) -> usize {
    self.floor() as usize
  }
}

impl CoerceLossyCeil<usize> for f32 {
  fn coerce_lossy_ceil(self) -> usize {
    self.ceil() as usize
  }
}

impl CoerceLossyFloor<u32> for f32 {
  fn coerce_lossy_floor(self) -> u32 {
    self.floor() as u32
  }
}

impl CoerceLossyRound<u8> for f32 {
  fn coerce_lossy_round(self) -> u8 {
    self.round() as u8
  }
}
