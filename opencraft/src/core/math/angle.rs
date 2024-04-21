use std::f32::consts::PI;

fn degrees_to_radians(degrees: f32) -> f32 {
  degrees * (PI / 180.0)
}

pub struct Degrees(f32);

impl Degrees {
  pub const fn new(value: f32) -> Self {
    Self(value)
  }

  pub const fn value(self) -> f32 {
    self.0
  }
}

pub struct Radians(f32);

impl Radians {
  pub const fn new(value: f32) -> Self {
    Self(value)
  }

  pub const fn value(self) -> f32 {
    self.0
  }
}

impl From<Degrees> for Radians {
  fn from(deg: Degrees) -> Self {
    Self::new(degrees_to_radians(deg.value()))
  }
}
