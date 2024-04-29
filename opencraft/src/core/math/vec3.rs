#[derive(Clone, Copy)]
pub struct Vec3 {
  x: f32,
  y: f32,
  z: f32,
}

impl Vec3 {
  pub const fn new(x: f32, y: f32, z: f32) -> Self {
    Self { x, y, z }
  }

  pub const fn x(self) -> f32 {
    self.x
  }

  pub const fn y(self) -> f32 {
    self.y
  }

  pub const fn z(self) -> f32 {
    self.z
  }
}
