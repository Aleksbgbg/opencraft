use crate::core::math;
use crate::core::math::Direction;
use crate::core::math::aligned_box3::AlignedBox3;
use crate::core::math::vec3::Vec3;
use crate::core::type_conversions::{Coerce, CoerceLossy, CoerceLossyFloor};
use crate::model::position::{BlockPosition, ChunkPosition};

const CUBE_SIZE: f32 = 1.0;
pub const CUBE_EXTENT: f32 = CUBE_SIZE / 2.0;

const BLOCK_OFFSET: f32 = 0.5;

const CHUNK_BASE_SIZE: i32 = 16;
const HALF_CHUNK_BASE_SIZE: i32 = CHUNK_BASE_SIZE / 2;
const CHUNK_HEIGHT: i32 = 384;
const HALF_CHUNK_HEIGHT: i32 = CHUNK_HEIGHT / 2;
const BELOW_GROUND_LEVELS: i32 = 54;
pub const Y_MIN_BLOCK_VALUE: i32 = -BELOW_GROUND_LEVELS;
pub const Y_MAX_BLOCK_VALUE: i32 = CHUNK_HEIGHT - BELOW_GROUND_LEVELS - 1;
pub const CHUNK_RADIUS: i32 = 16;
const VISIBLE_CHUNKS: i32 = CHUNK_RADIUS * CHUNK_RADIUS;

pub const VISIBLE_CHUNKS_USIZE: usize = VISIBLE_CHUNKS as usize;

const HALF_CHUNK_BASE_SIZE_F32: f32 = HALF_CHUNK_BASE_SIZE as f32;
const HALF_CHUNK_HEIGHT_F32: f32 = HALF_CHUNK_HEIGHT as f32;
const CHUNK_Y_CENTER_F32: f32 = (Y_MIN_BLOCK_VALUE + HALF_CHUNK_HEIGHT) as f32;

fn y_block_in_range(block_position: i32) -> bool {
  math::in_range(block_position, Y_MIN_BLOCK_VALUE, Y_MAX_BLOCK_VALUE)
}

fn xyz_block_to_world(block_position: i32) -> f32 {
  block_position.coerce_lossy() + BLOCK_OFFSET
}

fn xyz_world_to_block(world_position: f32) -> i32 {
  world_position.coerce_lossy_floor()
}

fn xz_block_to_chunk(block_position: i32) -> i32 {
  let negative_offset = block_position.signum().clamp(-1, 0);

  ((block_position - negative_offset) / CHUNK_BASE_SIZE) + negative_offset
}

fn xz_chunk_to_world(chunk_position: i32) -> f32 {
  ((chunk_position * CHUNK_BASE_SIZE) + HALF_CHUNK_BASE_SIZE).coerce_lossy()
}

fn xz_world_to_chunk(world_position: f32) -> i32 {
  xz_block_to_chunk(xyz_world_to_block(world_position))
}

fn xz_chunk_block_bounds(chunk_position: i32) -> (i32, i32) {
  let min = chunk_position * CHUNK_BASE_SIZE;
  let max = min + CHUNK_BASE_SIZE - 1;

  (min, max)
}

pub fn block_to_world(block_position: BlockPosition) -> Vec3 {
  let x = xyz_block_to_world(block_position.x());
  let y = xyz_block_to_world(block_position.y());
  let z = xyz_block_to_world(block_position.z());

  Vec3::new(x, y, z)
}

pub fn world_to_chunk(world_position: Vec3) -> ChunkPosition {
  ChunkPosition::new(
    xz_world_to_chunk(world_position.x()),
    xz_world_to_chunk(world_position.z()),
  )
}

pub fn block_to_chunk(block_position: BlockPosition) -> ChunkPosition {
  ChunkPosition::new(
    xz_block_to_chunk(block_position.x()),
    xz_block_to_chunk(block_position.z()),
  )
}

pub fn chunk_bounds(chunk_position: ChunkPosition) -> (BlockPosition, BlockPosition) {
  let (min_x, max_x) = xz_chunk_block_bounds(chunk_position.x());
  let (min_z, max_z) = xz_chunk_block_bounds(chunk_position.z());

  (
    BlockPosition::new(min_x, Y_MIN_BLOCK_VALUE, min_z),
    BlockPosition::new(max_x, Y_MAX_BLOCK_VALUE, max_z),
  )
}

pub fn block_index(block_position: BlockPosition) -> u32 {
  let chunk = block_to_chunk(block_position);
  let (chunk_block_min, _) = chunk_bounds(chunk);

  let relative_block_position = block_position - chunk_block_min;

  let x = relative_block_position.x();
  let y = relative_block_position.y() + BELOW_GROUND_LEVELS;
  let z = relative_block_position.z();

  ((y * CHUNK_BASE_SIZE * CHUNK_BASE_SIZE) + (z * CHUNK_BASE_SIZE) + x).coerce()
}

pub fn advance_in_direction(
  block_position: BlockPosition,
  direction: Direction,
) -> Option<(ChunkPosition, BlockPosition)> {
  let (x_offset, y_offset, z_offset) = match direction {
    Direction::XPos => (1, 0, 0),
    Direction::XNeg => (-1, 0, 0),
    Direction::YPos => (0, 1, 0),
    Direction::YNeg => (0, -1, 0),
    Direction::ZPos => (0, 0, 1),
    Direction::ZNeg => (0, 0, -1),
  };

  let offset_block_position = block_position + BlockPosition::new(x_offset, y_offset, z_offset);

  if y_block_in_range(offset_block_position.y()) {
    Some((block_to_chunk(offset_block_position), offset_block_position))
  } else {
    None
  }
}

pub fn block_bounding_volume(block_position: BlockPosition) -> AlignedBox3 {
  AlignedBox3::cube(block_to_world(block_position), CUBE_EXTENT)
}

pub fn chunk_bounding_volume(chunk_position: ChunkPosition) -> AlignedBox3 {
  AlignedBox3::new(
    Vec3::new(
      xz_chunk_to_world(chunk_position.x()),
      CHUNK_Y_CENTER_F32,
      xz_chunk_to_world(chunk_position.z()),
    ),
    Vec3::new(
      HALF_CHUNK_BASE_SIZE_F32,
      HALF_CHUNK_HEIGHT_F32,
      HALF_CHUNK_BASE_SIZE_F32,
    ),
  )
}

pub fn chunk_within_visible_radius(chunk_position: ChunkPosition) -> bool {
  chunk_position.len_sq() <= VISIBLE_CHUNKS
}

pub fn chunk_within_visible_radius_around(chunk: ChunkPosition, base: ChunkPosition) -> bool {
  chunk_within_visible_radius(chunk - base)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_y_block_in_range() {
    for &(y, block_in_range) in Y_BLOCK_IN_RANGE {
      assert_eq!(
        block_in_range,
        y_block_in_range(y),
        "expected block Y position {} to be {} range",
        y,
        if block_in_range { "in" } else { "out of" },
      );
    }
  }

  #[test]
  fn test_xyz_block_to_world() {
    for &(block, world) in XYZ_BLOCK_TO_WORLD {
      assert_eq!(
        world,
        xyz_block_to_world(block),
        "expected block {} to map to world position {:.1}",
        block,
        world,
      );
    }
  }

  #[test]
  fn test_xyz_world_to_block() {
    for &(world, block) in XYZ_WORLD_TO_BLOCK {
      assert_eq!(
        block,
        xyz_world_to_block(world),
        "expected world position {:.2} to map to block {}",
        world,
        block,
      );
    }
  }

  #[test]
  fn test_xz_block_to_chunk() {
    for &(block, chunk) in XZ_BLOCK_TO_CHUNK {
      assert_eq!(
        chunk,
        xz_block_to_chunk(block),
        "expected block {} to map to chunk {}",
        block,
        chunk,
      );
    }
  }

  #[test]
  fn test_xz_chunk_to_world() {
    for &(chunk, world) in XZ_CHUNK_TO_WORLD {
      assert_eq!(
        world,
        xz_chunk_to_world(chunk),
        "expected chunk {} to map to world position {:.1}",
        chunk,
        world,
      );
    }
  }

  #[test]
  fn test_xz_world_to_chunk() {
    for &(world, chunk) in XZ_WORLD_TO_CHUNK {
      assert_eq!(
        chunk,
        xz_world_to_chunk(world),
        "expected world position {:.2} to map to chunk {}",
        world,
        chunk,
      );
    }
  }

  #[test]
  fn test_xz_chunk_block_bounds() {
    for &(chunk, expected_min, expected_max) in XZ_CHUNK_BLOCK_BOUNDS {
      let (actual_min, actual_max) = xz_chunk_block_bounds(chunk);

      assert_eq!(
        expected_min, actual_min,
        "expected chunk {} to have min block {}",
        chunk, expected_min,
      );
      assert_eq!(
        expected_max, actual_max,
        "expected chunk {} to have max block {}",
        chunk, expected_max,
      );
    }
  }

  #[test]
  fn test_advance_in_direction() {
    for &(block, direction, expected_result) in ADVANCE_IN_DIRECTION {
      assert_eq!(
        expected_result,
        advance_in_direction(block, direction),
        "expected that advancing from {} in the {:#?} direction would result in {:#?}",
        block,
        direction,
        expected_result,
      );
    }
  }

  #[test]
  fn test_block_bounding_volume() {
    const EXTENT: Vec3 = Vec3::new(0.5, 0.5, 0.5);

    for &(block, origin) in BLOCK_BOUNDING_VOLUME {
      let volume = block_bounding_volume(block);

      assert_eq!(
        origin,
        volume.origin(),
        "expected block {} to be centered at {}",
        block,
        origin,
      );
      assert_eq!(
        EXTENT,
        volume.extent(),
        "expected block {} to have extent {}",
        block,
        EXTENT,
      );
    }
  }

  #[test]
  fn test_chunk_bounding_volume() {
    const EXTENT: Vec3 = Vec3::new(8.0, 192.0, 8.0);

    for &(chunk, origin) in CHUNK_BOUNDING_VOLUME {
      let volume = chunk_bounding_volume(chunk);

      assert_eq!(
        origin,
        volume.origin(),
        "expected chunk at {} to be centered at {}",
        chunk,
        origin,
      );
      assert_eq!(
        EXTENT,
        volume.extent(),
        "expected chunk at {} to have extent {}",
        chunk,
        EXTENT,
      );
    }
  }

  const Y_BLOCK_IN_RANGE: &[(i32, bool)] = &[
    (-55, false),
    (-54, true),
    (-5, true),
    (0, true),
    (5, true),
    (329, true),
    (330, false),
  ];

  const XYZ_BLOCK_TO_WORLD: &[(i32, f32)] = &[
    (0, 0.5),
    (1, 1.5),
    (2, 2.5),
    (-1, -0.5),
    (-2, -1.5),
    (-3, -2.5),
  ];
  const XYZ_WORLD_TO_BLOCK: &[(f32, i32)] = &[
    (0.0, 0),
    (0.5, 0),
    (0.99, 0),
    (1.0, 1),
    (1.5, 1),
    (1.99, 1),
    (2.0, 2),
    (2.5, 2),
    (2.99, 2),
    (-0.01, -1),
    (-0.5, -1),
    (-1.0, -1),
    (-1.01, -2),
    (-1.5, -2),
    (-2.0, -2),
    (-2.01, -3),
    (-2.5, -3),
    (-2.99, -3),
  ];

  const XZ_BLOCK_TO_CHUNK: &[(i32, i32)] = &[
    (0, 0),
    (15, 0),
    (16, 1),
    (31, 1),
    (32, 2),
    (-1, -1),
    (-16, -1),
    (-17, -2),
    (-32, -2),
    (-33, -3),
  ];

  const XZ_CHUNK_TO_WORLD: &[(i32, f32)] = &[
    (0, 8.0),
    (1, 24.0),
    (2, 40.0),
    (-1, -8.0),
    (-2, -24.0),
    (-3, -40.0),
  ];
  const XZ_WORLD_TO_CHUNK: &[(f32, i32)] = &[
    (0.0, 0),
    (15.99, 0),
    (16.0, 1),
    (31.99, 1),
    (32.0, 2),
    (47.99, 2),
    (-0.0, 0),
    (-0.01, -1),
    (-16.0, -1),
    (-16.01, -2),
    (-32.0, -2),
  ];

  const XZ_CHUNK_BLOCK_BOUNDS: &[(i32, i32, i32)] = &[
    (0, 0, 15),
    (1, 16, 31),
    (2, 32, 47),
    (-1, -16, -1),
    (-2, -32, -17),
    (-3, -48, -33),
  ];

  const ADVANCE_IN_DIRECTION: &[(
    BlockPosition,
    Direction,
    Option<(ChunkPosition, BlockPosition)>,
  )] = &[
    // Y
    (
      BlockPosition::new(0, 0, 0),
      Direction::YPos,
      Some((ChunkPosition::new(0, 0), BlockPosition::new(0, 1, 0))),
    ),
    (
      BlockPosition::new(0, 0, 0),
      Direction::YNeg,
      Some((ChunkPosition::new(0, 0), BlockPosition::new(0, -1, 0))),
    ),
    (BlockPosition::new(0, 329, 0), Direction::YPos, None),
    (BlockPosition::new(0, -54, 0), Direction::YNeg, None),
    // X
    (
      BlockPosition::new(0, 0, 0),
      Direction::XPos,
      Some((ChunkPosition::new(0, 0), BlockPosition::new(1, 0, 0))),
    ),
    (
      BlockPosition::new(0, 0, 0),
      Direction::XNeg,
      Some((ChunkPosition::new(-1, 0), BlockPosition::new(-1, 0, 0))),
    ),
    // Z
    (
      BlockPosition::new(0, 0, 0),
      Direction::ZPos,
      Some((ChunkPosition::new(0, 0), BlockPosition::new(0, 0, 1))),
    ),
    (
      BlockPosition::new(0, 0, 0),
      Direction::ZNeg,
      Some((ChunkPosition::new(0, -1), BlockPosition::new(0, 0, -1))),
    ),
  ];

  const BLOCK_BOUNDING_VOLUME: &[(BlockPosition, Vec3)] = &[
    (BlockPosition::new(0, 0, 0), Vec3::new(0.5, 0.5, 0.5)),
    // Y
    (BlockPosition::new(0, 5, 0), Vec3::new(0.5, 5.5, 0.5)),
    (BlockPosition::new(0, -5, 0), Vec3::new(0.5, -4.5, 0.5)),
    (BlockPosition::new(0, -54, 0), Vec3::new(0.5, -53.5, 0.5)),
    (BlockPosition::new(0, 329, 0), Vec3::new(0.5, 329.5, 0.5)),
    // X (also applies to Z)
    (BlockPosition::new(0, 0, 0), Vec3::new(0.5, 0.5, 0.5)),
    (BlockPosition::new(1, 0, 0), Vec3::new(1.5, 0.5, 0.5)),
    (BlockPosition::new(2, 0, 0), Vec3::new(2.5, 0.5, 0.5)),
    (BlockPosition::new(-1, 0, 0), Vec3::new(-0.5, 0.5, 0.5)),
    (BlockPosition::new(-2, 0, 0), Vec3::new(-1.5, 0.5, 0.5)),
    (BlockPosition::new(-3, 0, 0), Vec3::new(-2.5, 0.5, 0.5)),
  ];
  const CHUNK_BOUNDING_VOLUME: &[(ChunkPosition, Vec3)] = &[
    (ChunkPosition::new(0, 0), Vec3::new(8.0, 138.0, 8.0)),
    (ChunkPosition::new(1, 0), Vec3::new(24.0, 138.0, 8.0)),
    (ChunkPosition::new(2, 0), Vec3::new(40.0, 138.0, 8.0)),
    (ChunkPosition::new(-1, 0), Vec3::new(-8.0, 138.0, 8.0)),
    (ChunkPosition::new(-2, 0), Vec3::new(-24.0, 138.0, 8.0)),
    (ChunkPosition::new(-3, 0), Vec3::new(-40.0, 138.0, 8.0)),
  ];
}
