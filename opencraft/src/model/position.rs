use derive_more::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

#[derive(
  Debug,
  Default,
  Clone,
  Copy,
  PartialEq,
  Eq,
  Hash,
  Neg,
  Add,
  Sub,
  Mul,
  Div,
  AddAssign,
  SubAssign,
  MulAssign,
  DivAssign,
)]
pub struct ChunkPosition {
  x: i32,
  z: i32,
}

impl ChunkPosition {
  pub const fn new(x: i32, z: i32) -> Self {
    Self { x, z }
  }

  pub const fn x(self) -> i32 {
    self.x
  }

  pub const fn z(self) -> i32 {
    self.z
  }

  pub fn dot(lhs: Self, rhs: Self) -> i32 {
    (lhs.x() * rhs.x()) + (lhs.z() * rhs.z())
  }

  pub fn len_sq(self) -> i32 {
    Self::dot(self, self)
  }
}

#[derive(
  Debug,
  Default,
  Clone,
  Copy,
  PartialEq,
  Eq,
  Hash,
  Neg,
  Add,
  Sub,
  Mul,
  Div,
  AddAssign,
  SubAssign,
  MulAssign,
  DivAssign,
)]
pub struct BlockPosition {
  x: i32,
  y: i32,
  z: i32,
}

impl BlockPosition {
  pub const fn new(x: i32, y: i32, z: i32) -> Self {
    Self { x, y, z }
  }

  pub const fn x(self) -> i32 {
    self.x
  }

  pub const fn y(self) -> i32 {
    self.y
  }

  pub const fn z(self) -> i32 {
    self.z
  }
}
