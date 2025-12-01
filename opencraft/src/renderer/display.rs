use crate::core::math::intersect::BoxFace;
use crate::core::math::vec3::Vec3;
use crate::model::position::{BlockPosition, ChunkPosition};
use std::fmt::{Display, Formatter, Result};
use thousands::Separable;

#[derive(Clone, Copy)]
pub struct Bytes(pub usize);

impl Display for Bytes {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    let Bytes(size) = self;
    let kilobytes = size / 1024;

    write!(f, "{}K", kilobytes.separate_with_commas())
  }
}

impl Display for BoxFace {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    write!(
      f,
      "{}",
      match self {
        BoxFace::XPos => "+X",
        BoxFace::XNeg => "-X",
        BoxFace::YPos => "+Y",
        BoxFace::YNeg => "-Y",
        BoxFace::ZPos => "+Z",
        BoxFace::ZNeg => "-Z",
      }
    )
  }
}

impl Display for ChunkPosition {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    write!(f, "({}, {})", self.x(), self.z())
  }
}

impl Display for BlockPosition {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    write!(f, "({}, {}, {})", self.x(), self.y(), self.z())
  }
}

impl Display for Vec3 {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    write!(f, "({:.1}, {:.1}, {:.1})", self.x(), self.y(), self.z())
  }
}
