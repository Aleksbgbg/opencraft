use crate::core::math::vec3::Vec3;
use crate::model::layout;
use crate::model::layout::{CHUNK_RADIUS, Y_MAX_BLOCK_VALUE, Y_MIN_BLOCK_VALUE};
use crate::model::position::{BlockPosition, ChunkPosition};
use itertools::iproduct;
use std::ops::RangeInclusive;

const Y_ITERATOR: RangeInclusive<i32> = Y_MIN_BLOCK_VALUE..=Y_MAX_BLOCK_VALUE;

pub fn chunk_levels() -> impl Iterator<Item = i32> {
  Y_ITERATOR
}

pub fn chunk_level_blocks(
  chunk_position: ChunkPosition,
) -> impl Iterator<Item = (i32, i32)> + Clone {
  let (min, max) = layout::chunk_bounds(chunk_position);
  iproduct!(min.x()..=max.x(), min.z()..=max.z())
}

pub fn chunk_blocks(chunk_position: ChunkPosition) -> impl Iterator<Item = BlockPosition> {
  iproduct!(Y_ITERATOR, chunk_level_blocks(chunk_position))
    .map(|(y, (x, z))| BlockPosition::new(x, y, z))
}

pub fn surrounding_chunks(world_position: Vec3) -> impl Iterator<Item = ChunkPosition> {
  SurroundingChunksIterator::new(layout::world_to_chunk(world_position))
}

pub fn immediate_surrounding_chunks(world_position: Vec3) -> impl Iterator<Item = ChunkPosition> {
  SurroundingChunksIterator::last_ring(layout::world_to_chunk(world_position), 1)
}

pub fn chunk_difference(
  position_before: Vec3,
  position_after: Vec3,
) -> Option<(
  impl Iterator<Item = ChunkPosition>,
  impl Iterator<Item = ChunkPosition>,
)> {
  let before_chunk = layout::world_to_chunk(position_before);
  let after_chunk = layout::world_to_chunk(position_after);

  if (before_chunk.x() == after_chunk.x()) && (before_chunk.z() == after_chunk.z()) {
    return None;
  }

  Some((
    SurroundingChunksIterator::new(before_chunk)
      .filter(move |&chunk| !layout::chunk_within_visible_radius_around(chunk, after_chunk)),
    SurroundingChunksIterator::new(after_chunk)
      .filter(move |&chunk| !layout::chunk_within_visible_radius_around(chunk, before_chunk)),
  ))
}

struct SurroundingChunksIterator {
  base_chunk: ChunkPosition,
  current_ring: i32,
  last_ring: i32,
  chunks_per_ring_length: i32,
  current_chunk: i32,
  last_chunk: i32,
}

impl SurroundingChunksIterator {
  #[allow(clippy::new_ret_no_self)]
  fn new(base_chunk: ChunkPosition) -> impl Iterator<Item = ChunkPosition> {
    Self::last_ring(base_chunk, CHUNK_RADIUS)
  }

  fn last_ring(base_chunk: ChunkPosition, last_ring: i32) -> impl Iterator<Item = ChunkPosition> {
    // Treat ring 0 (containing only the base chunk) as a special case to simplify
    // the rest of the iterator
    [base_chunk].into_iter().chain(Self {
      base_chunk,
      current_ring: 1,
      last_ring,
      chunks_per_ring_length: 2,
      current_chunk: 0,
      last_chunk: 8,
    })
  }
}

impl Iterator for SurroundingChunksIterator {
  type Item = ChunkPosition;

  fn next(&mut self) -> Option<Self::Item> {
    if self.current_ring > self.last_ring {
      return None;
    }

    // Split the ring into 4 equal-sized "lengths", one for each cardinal direction
    let length_index = self.current_chunk / self.chunks_per_ring_length;
    // Move the block one across to avoid a visible edge during loading
    let across = (self.current_chunk % self.chunks_per_ring_length) + 1;
    let chunk_position = match length_index {
      0 => {
        let corner_x = -self.current_ring;
        let corner_z = self.current_ring;

        ChunkPosition::new(corner_x + across, corner_z)
      }
      1 => {
        let corner_x = self.current_ring;
        let corner_z = self.current_ring;

        ChunkPosition::new(corner_x, corner_z - across)
      }
      2 => {
        let corner_x = self.current_ring;
        let corner_z = -self.current_ring;

        ChunkPosition::new(corner_x - across, corner_z)
      }
      3 => {
        let corner_x = -self.current_ring;
        let corner_z = -self.current_ring;

        ChunkPosition::new(corner_x, corner_z + across)
      }
      _ => unreachable!("length {} should be in the range [0, 4)", length_index),
    };

    self.current_chunk += 1;
    if self.current_chunk == self.last_chunk {
      self.current_ring += 1;
      self.chunks_per_ring_length = self.current_ring * 2;
      self.last_chunk = self.chunks_per_ring_length * 4;
      self.current_chunk = 0;
    }

    if layout::chunk_within_visible_radius(chunk_position) {
      Some(self.base_chunk + chunk_position)
    } else {
      self.next()
    }
  }
}
