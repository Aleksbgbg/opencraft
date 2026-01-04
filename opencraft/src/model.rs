pub mod block;
pub mod chunk;
pub mod iterators;
pub mod layout;
pub mod position;
pub mod terrain;

use crate::camera::Camera;
use crate::core::math::angle::{Angle, FULL_ROTATION};
use crate::core::math::intersect::{BoxFace, Intersects};
use crate::core::math::segment3::Segment3;
use crate::core::math::vec2::Vec2;
use crate::core::math::vec3::Vec3;
use crate::core::math::{X_AXIS, Y_AXIS, Z_AXIS};
use crate::core::type_conversions::{CoerceLossy, CoerceLossyFloor};
use crate::model::block::Block;
use crate::model::chunk::Chunk;
use crate::model::layout::VISIBLE_CHUNKS_USIZE;
use crate::model::position::{BlockPosition, ChunkPosition};
use crate::model::terrain::{ClassicFlat, Generate};
use arrayvec::ArrayVec;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

const FRAME_TIME_MEASUREMENTS: usize = 60;

pub struct UpdateInputs<'a> {
  pub delta: Duration,

  pub keys_down: &'a HashSet<KeyCode>,
  pub keys_released: &'a HashSet<KeyCode>,

  pub mouse_movement: Vec2,
  pub mouse_buttons_released: &'a HashSet<MouseButton>,
}

pub struct Model {
  show_debug_display: bool,
  frame_times: ArrayVec<Duration, FRAME_TIME_MEASUREMENTS>,
  frame_time_stale_index: usize,

  player_camera: Camera,

  generator: Arc<dyn Generate>,
  cached_modifications: HashMap<ChunkPosition, HashMap<BlockPosition, Block>>,
  chunks: HashMap<ChunkPosition, Chunk>,
  unloaded_chunks: Vec<ChunkPosition>,
  loaded_chunks: Vec<ChunkPosition>,

  destroyed_blocks: Vec<(ChunkPosition, BlockPosition)>,
  created_blocks: Vec<(ChunkPosition, BlockPosition)>,
  target_block: Option<(ChunkPosition, BlockPosition, BoxFace, Block)>,
}

impl Model {
  pub fn new() -> Self {
    let player_camera = Camera::new(Vec3::new(0.0, 2.5, 0.0));
    let generator: Arc<dyn Generate> = Arc::new(ClassicFlat);

    let mut chunks = HashMap::with_capacity(VISIBLE_CHUNKS_USIZE);
    let loaded_chunks = iterators::surrounding_chunks(player_camera.position()).collect();
    for &chunk_position in &loaded_chunks {
      chunks.insert(
        chunk_position,
        Chunk::load(chunk_position, Arc::clone(&generator), HashMap::default()),
      );
    }

    Self {
      show_debug_display: cfg!(debug_assertions),
      frame_times: Default::default(),
      frame_time_stale_index: Default::default(),
      player_camera,
      generator,
      cached_modifications: Default::default(),
      chunks,
      unloaded_chunks: Default::default(),
      loaded_chunks,
      destroyed_blocks: Default::default(),
      created_blocks: Default::default(),
      target_block: Default::default(),
    }
  }

  pub fn update(
    &mut self,
    &UpdateInputs {
      delta,
      keys_down,
      keys_released,
      mouse_movement,
      mouse_buttons_released,
    }: &UpdateInputs<'_>,
  ) {
    const PLAYER_MOVEMENT_SPEED: f32 = 10.0;
    const PLAYER_CAMERA_ROTATION_SPEED: Angle = FULL_ROTATION;
    const REACH_DISTANCE: f32 = 5.0;

    if self.show_debug_display {
      if self.frame_times.len() < FRAME_TIME_MEASUREMENTS {
        self.frame_times.push(delta);
      } else {
        self.frame_times[self.frame_time_stale_index] = delta;
        self.frame_time_stale_index = (self.frame_time_stale_index + 1) % FRAME_TIME_MEASUREMENTS;
      }
    }

    self.player_camera.rotate(
      PLAYER_CAMERA_ROTATION_SPEED * mouse_movement.x(),
      PLAYER_CAMERA_ROTATION_SPEED * mouse_movement.y(),
    );

    let delta_secs = delta.as_secs_f32();

    let mut player_movement = Vec3::default();
    if keys_down.contains(&KeyCode::KeyW) {
      player_movement += Z_AXIS;
    }
    if keys_down.contains(&KeyCode::KeyS) {
      player_movement -= Z_AXIS;
    }
    if keys_down.contains(&KeyCode::KeyA) {
      player_movement -= X_AXIS;
    }
    if keys_down.contains(&KeyCode::KeyD) {
      player_movement += X_AXIS;
    }
    if keys_down.contains(&KeyCode::Space) {
      player_movement += Y_AXIS;
    }
    if keys_down.contains(&KeyCode::ShiftLeft) {
      player_movement -= Y_AXIS;
    }
    if player_movement.len_sq() > 0.0 {
      let position_before = self.player_camera.position();

      self
        .player_camera
        .translate(PLAYER_MOVEMENT_SPEED * delta_secs * player_movement.norm());

      let position_after = self.player_camera.position();

      if let Some((unloaded_chunks, loaded_chunks)) =
        iterators::chunk_difference(position_before, position_after)
      {
        for chunk_position in unloaded_chunks {
          let chunk = self.chunks.remove(&chunk_position).unwrap();
          let chunk_modifications = chunk.unload();
          if !chunk_modifications.is_empty() {
            self
              .cached_modifications
              .insert(chunk_position, chunk_modifications);
          }

          self.unloaded_chunks.push(chunk_position);
        }
        for chunk_position in loaded_chunks {
          let chunk_modifications = self
            .cached_modifications
            .remove(&chunk_position)
            .unwrap_or_default();
          self.chunks.insert(
            chunk_position,
            Chunk::load(
              chunk_position,
              Arc::clone(&self.generator),
              chunk_modifications,
            ),
          );

          self.loaded_chunks.push(chunk_position);
        }
      }
    }

    if let Some((chunk_position, block_position, face, _)) = self.target_block {
      if mouse_buttons_released.contains(&MouseButton::Left) {
        self
          .chunks
          .get_mut(&chunk_position)
          .unwrap()
          .destroy_block(block_position);

        self.destroyed_blocks.push((chunk_position, block_position));
      } else if mouse_buttons_released.contains(&MouseButton::Right)
        && let Some((chunk_position, block_position)) =
          layout::advance_in_direction(block_position, face.into())
      {
        self
          .chunks
          .get_mut(&chunk_position)
          .unwrap()
          .place_block(block_position, Block::Grass);

        self.created_blocks.push((chunk_position, block_position));
      }
    }

    let reach = Segment3::start_direction_len(
      self.player_camera.position(),
      self.player_camera.forward(),
      REACH_DISTANCE,
    );
    self.target_block = {
      let mut min_dist = f32::MAX;
      let mut intersection = None;

      for chunk in iterators::immediate_surrounding_chunks(reach.origin())
        .filter_map(|chunk_position| self.chunks.get(&chunk_position))
        .filter(|chunk| layout::chunk_bounding_volume(chunk.position()).intersects(&reach))
      {
        for (block_position, block) in chunk.blocks() {
          if let Some(face) =
            layout::block_bounding_volume(block_position).find_intersecting_face(&reach)
          {
            let world_position = layout::block_to_world(block_position);
            let dist = Vec3::dist_sq(reach.start(), world_position);

            if dist < min_dist {
              min_dist = dist;
              intersection = Some((chunk.position(), block_position, face, block));
            }
          }
        }
      }

      intersection
    };

    if keys_released.contains(&KeyCode::F3) {
      self.show_debug_display = !self.show_debug_display;
    }
  }

  pub fn scene(&self) -> Scene<'_> {
    let mean_frame_time_ms = self
      .frame_times
      .iter()
      .map(Duration::as_millis_f32)
      .sum::<f32>()
      / self.frame_times.len().coerce_lossy();
    let frames_per_second = (1000.0 / mean_frame_time_ms).coerce_lossy_floor();

    let debug_display = if self.show_debug_display {
      Some(DebugDisplay {
        frames_per_second,
        mean_frame_time_ms,
      })
    } else {
      None
    };

    Scene {
      debug_display,
      player_camera: &self.player_camera,
      chunks: &self.chunks,
      unloaded_chunks: &self.unloaded_chunks,
      loaded_chunks: &self.loaded_chunks,
      destroyed_blocks: &self.destroyed_blocks,
      created_blocks: &self.created_blocks,
      target_block: self
        .target_block
        .map(
          |(chunk_position, block_position, face, block)| TargetBlock {
            chunk_position,
            block_position,
            world_position: layout::block_to_world(block_position),
            face,
            block,
          },
        ),
    }
  }

  pub fn clear_changes(&mut self) {
    self.unloaded_chunks.clear();
    self.loaded_chunks.clear();

    self.destroyed_blocks.clear();
    self.created_blocks.clear();
  }
}

pub struct DebugDisplay {
  pub frames_per_second: u32,
  pub mean_frame_time_ms: f32,
}

pub struct TargetBlock {
  pub chunk_position: ChunkPosition,
  pub block_position: BlockPosition,
  pub world_position: Vec3,
  pub face: BoxFace,
  pub block: Block,
}

pub struct Scene<'a> {
  pub debug_display: Option<DebugDisplay>,

  pub player_camera: &'a Camera,

  pub chunks: &'a HashMap<ChunkPosition, Chunk>,
  pub unloaded_chunks: &'a [ChunkPosition],
  pub loaded_chunks: &'a [ChunkPosition],

  pub destroyed_blocks: &'a [(ChunkPosition, BlockPosition)],
  pub created_blocks: &'a [(ChunkPosition, BlockPosition)],
  pub target_block: Option<TargetBlock>,
}
