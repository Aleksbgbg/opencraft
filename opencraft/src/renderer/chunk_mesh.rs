use crate::core::math::Direction;
use crate::core::math::intersect::BoxFace;
use crate::model::block::Block;
use crate::model::chunk::Chunk;
use crate::model::position::BlockPosition;
use crate::model::{iterators, layout};
use crate::renderer::{VERTICES, Vertex};
use arrayvec::ArrayVec;
use std::collections::HashMap;
use strum::{EnumCount, IntoEnumIterator};

fn face_to_index(face: BoxFace) -> usize {
  match face {
    BoxFace::XPos => 0,
    BoxFace::XNeg => 1,
    BoxFace::YPos => 2,
    BoxFace::YNeg => 3,
    BoxFace::ZPos => 4,
    BoxFace::ZNeg => 5,
  }
}

struct FaceSet {
  set: u8,
}

impl FaceSet {
  fn new() -> Self {
    Self { set: 0 }
  }

  fn set(&mut self, face: BoxFace) {
    self.set |= 1 << face_to_index(face);
  }

  fn unset(&mut self, face: BoxFace) {
    self.set &= !(1 << face_to_index(face));
  }

  fn is_set(&self, face: BoxFace) -> bool {
    ((self.set >> face_to_index(face)) & 0b1) == 0b1
  }

  fn is_empty(&self) -> bool {
    self.set == 0
  }

  fn faces(&self) -> ArrayVec<BoxFace, { BoxFace::COUNT }> {
    let mut faces = ArrayVec::new();

    for face in BoxFace::iter() {
      if self.is_set(face) {
        faces.push(face);
      }
    }

    faces
  }
}

fn is_visible_face(
  chunk: &Chunk,
  block_position: BlockPosition,
  face_direction: Direction,
) -> bool {
  let next = layout::advance_in_direction(block_position, face_direction);

  next.is_none()
    || next.is_some_and(|(next_chunk, next_block)| {
      (next_chunk != chunk.position()) || (chunk.get(next_block) == Block::Air)
    })
}

fn generate_block_face_set(chunk: &Chunk, block_position: BlockPosition) -> FaceSet {
  let mut face_set = FaceSet::new();

  for face_direction in Direction::iter() {
    if is_visible_face(chunk, block_position, face_direction) {
      face_set.set(face_direction.into());
    }
  }

  face_set
}

fn update_block_face_set(face_set: &mut FaceSet, next_block: Block, face: BoxFace) {
  if next_block == Block::Air {
    face_set.set(face);
  } else {
    face_set.unset(face);
  }
}

fn generate_chunk_face_set(chunk: &Chunk) -> HashMap<BlockPosition, FaceSet> {
  let mut faces = HashMap::new();

  for block_position in iterators::chunk_blocks(chunk.position()) {
    if chunk.get(block_position) == Block::Air {
      continue;
    }

    let face_set = generate_block_face_set(chunk, block_position);
    if !face_set.is_empty() {
      faces.insert(block_position, face_set);
    }
  }

  faces
}

fn generate_face_mesh(vertices: &mut Vec<Vertex>, block_position: BlockPosition, face: BoxFace) {
  const FACE_VERTICES: usize = 6;

  let base_cube_face_offset = face_to_index(face);
  let base_cube_vertex_offset = base_cube_face_offset * FACE_VERTICES;

  vertices.extend(&VERTICES[base_cube_vertex_offset..base_cube_vertex_offset + FACE_VERTICES]);

  let new_vertices_start = vertices.len() - FACE_VERTICES;
  let world_position = layout::block_to_world(block_position);
  for vertex in &mut vertices[new_vertices_start..] {
    vertex.position[0] += world_position.x();
    vertex.position[1] += world_position.y();
    vertex.position[2] += world_position.z();
  }
}

#[derive(Default)]
pub struct ChunkMesh {
  faces: HashMap<BlockPosition, FaceSet>,
  last_vertices_len: usize,
}

impl ChunkMesh {
  pub fn generate(chunk: &Chunk) -> Self {
    Self {
      faces: generate_chunk_face_set(chunk),
      last_vertices_len: 0,
    }
  }

  pub fn update_incremental(&mut self, chunk: &Chunk, block_position: BlockPosition) {
    let block = chunk.get(block_position);

    if block == Block::Air {
      self.faces.remove(&block_position);
    } else {
      self.faces.insert(
        block_position,
        generate_block_face_set(chunk, block_position),
      );
    }

    for face_direction in Direction::iter() {
      let next = layout::advance_in_direction(block_position, face_direction);

      if let Some((next_chunk, next_block)) = next
        && (next_chunk == chunk.position())
        && (chunk.get(next_block) != Block::Air)
      {
        let face: BoxFace = face_direction.into();
        let next_block_face = face.opposite();
        self
          .faces
          .entry(next_block)
          .and_modify(|next_block_face_set| {
            update_block_face_set(next_block_face_set, block, next_block_face)
          })
          .or_insert_with(|| {
            let mut next_block_face_set = FaceSet::new();
            update_block_face_set(&mut next_block_face_set, block, next_block_face);
            next_block_face_set
          });
      }
    }
  }

  pub fn generate_vertices(&mut self) -> Vec<Vertex> {
    let mut vertices = Vec::with_capacity(self.last_vertices_len);

    for (&block_position, face_set) in &self.faces {
      for face in face_set.faces() {
        generate_face_mesh(&mut vertices, block_position, face);
      }
    }

    self.last_vertices_len = vertices.len();

    vertices
  }
}
