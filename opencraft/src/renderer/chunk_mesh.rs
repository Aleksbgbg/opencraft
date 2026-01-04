use crate::core::math::Direction;
use crate::core::math::intersect::BoxFace;
use crate::model::block::Block;
use crate::model::chunk::Chunk;
use crate::model::position::BlockPosition;
use crate::model::{iterators, layout};
use crate::renderer::{VERTICES, Vertex};
use strum::IntoEnumIterator;

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

fn generate_face_mesh(vertices: &mut Vec<Vertex>, block_position: BlockPosition, face: BoxFace) {
  const FACE_VERTICES: usize = 6;

  let base_cube_face_offset = match face {
    BoxFace::XPos => 0,
    BoxFace::XNeg => 1,
    BoxFace::YPos => 2,
    BoxFace::YNeg => 3,
    BoxFace::ZPos => 4,
    BoxFace::ZNeg => 5,
  };
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
  pub vertices: Vec<Vertex>,
}

impl ChunkMesh {
  pub fn generate(chunk: &Chunk) -> Self {
    let mut vertices = Vec::new();

    for block_position in iterators::chunk_blocks(chunk.position()) {
      if chunk.get(block_position) == Block::Air {
        continue;
      }

      for face_direction in Direction::iter() {
        if is_visible_face(chunk, block_position, face_direction) {
          generate_face_mesh(&mut vertices, block_position, face_direction.into());
        }
      }
    }

    Self { vertices }
  }
}
