use crate::core::math;
use crate::model::block::Block;
use crate::model::position::BlockPosition;

pub fn generate(block: BlockPosition) -> Block {
  if math::in_range(block.y(), -3, 0) {
    Block::Grass
  } else {
    Block::Air
  }
}
