use crate::core::math;
use crate::model::block::Block;
use crate::model::position::BlockPosition;

pub trait Generate: Send + Sync {
  fn generate(&self, block: BlockPosition) -> Block;
}

pub struct ClassicFlat;

impl Generate for ClassicFlat {
  fn generate(&self, block: BlockPosition) -> Block {
    if math::in_range(block.y(), -3, 0) {
      Block::Grass
    } else {
      Block::Air
    }
  }
}
