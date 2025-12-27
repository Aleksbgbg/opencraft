use crate::model::block::Block;
use crate::model::position::{BlockPosition, ChunkPosition};
use crate::model::{iterators, terrain};
use std::collections::HashMap;

pub struct Chunk {
  position: ChunkPosition,
  blocks: HashMap<BlockPosition, Block>,
  modifications: HashMap<BlockPosition, Block>,
}

impl Chunk {
  pub fn load(chunk_position: ChunkPosition, modifications: HashMap<BlockPosition, Block>) -> Self {
    let mut blocks = HashMap::new();

    for block_position in iterators::chunk_blocks(chunk_position) {
      let block = if let Some(&block) = modifications.get(&block_position) {
        block
      } else {
        terrain::generate(block_position)
      };

      if block == Block::Air {
        continue;
      }

      blocks.insert(block_position, block);
    }

    Self {
      position: chunk_position,
      blocks,
      modifications,
    }
  }

  pub fn unload(self) -> HashMap<BlockPosition, Block> {
    self.modifications
  }

  pub fn position(&self) -> ChunkPosition {
    self.position
  }

  pub fn get(&self, position: BlockPosition) -> Block {
    self.blocks.get(&position).copied().unwrap_or(Block::Air)
  }

  pub fn blocks(&self) -> impl Iterator<Item = (BlockPosition, Block)> {
    self.blocks.iter().map(|(key, value)| (*key, *value))
  }

  pub fn place_block(&mut self, position: BlockPosition, block: Block) {
    self.blocks.insert(position, block);
    self.update_modifications(position, block);
  }

  pub fn destroy_block(&mut self, position: BlockPosition) {
    self.blocks.remove(&position);
    self.update_modifications(position, Block::Air);
  }

  fn update_modifications(&mut self, position: BlockPosition, block: Block) {
    if terrain::generate(position) == block {
      self.modifications.remove(&position);
    } else {
      self.modifications.insert(position, block);
    }
  }
}
