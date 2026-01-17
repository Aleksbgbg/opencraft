use crate::camera::Camera;
use crate::core::math::aligned_box3::{AlignedBox3, BoxFace};
use crate::core::math::angle::{Angle, FULL_ROTATION};
use crate::core::math::segment3::Segment3;
use crate::core::math::vec2::Vec2;
use crate::core::math::vec3::Vec3;
use crate::core::math::{X_AXIS, Y_AXIS, Z_AXIS};
use crate::core::type_conversions::{CoerceLossy, CoerceLossyFloor};
use arrayvec::ArrayVec;
use std::collections::HashSet;
use std::time::Duration;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

const FRAME_TIME_MEASUREMENTS: usize = 60;

pub const BLOCK_LIMIT: usize = 256;

const CUBE_SIZE: f32 = 1.0;
pub const CUBE_EXTENT: f32 = CUBE_SIZE / 2.0;
const CUBE_TRANSLATE: Vec3 = Vec3::new(0.0, 0.0, 3.0);

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

  blocks: Vec<Vec3>,
  target_block_index_face: Option<(usize, BoxFace)>,
}

impl Model {
  pub fn new() -> Self {
    Self {
      show_debug_display: cfg!(debug_assertions),
      frame_times: Default::default(),
      frame_time_stale_index: Default::default(),
      player_camera: Default::default(),
      blocks: Vec::from([CUBE_TRANSLATE]),
      target_block_index_face: Default::default(),
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
      self
        .player_camera
        .translate(PLAYER_MOVEMENT_SPEED * delta_secs * player_movement.norm());
    }

    if let Some((index, face)) = self.target_block_index_face {
      if mouse_buttons_released.contains(&MouseButton::Left) {
        self.blocks.swap_remove(index);
      } else if mouse_buttons_released.contains(&MouseButton::Right)
        && (self.blocks.len() < BLOCK_LIMIT)
      {
        let target_block = self.blocks.get(index).unwrap();
        let next_block = *target_block + (CUBE_SIZE * face.normal());

        self.blocks.push(next_block);
      }
    }

    let reach = Segment3::start_direction_len(
      self.player_camera.position(),
      self.player_camera.forward(),
      REACH_DISTANCE,
    );

    self.target_block_index_face = None;
    let mut min_dist = f32::MAX;
    for (index, block) in self.blocks.iter().enumerate() {
      if let Some(face) = AlignedBox3::cube(*block, CUBE_EXTENT).find_intersecting_face(&reach) {
        let dist = Vec3::dist_sq(self.player_camera.position(), *block);

        if dist < min_dist {
          self.target_block_index_face = Some((index, face));
          min_dist = dist;
        }
      }
    }

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

    let target_block_index = self.target_block_index_face.map(|(index, _)| index);

    Scene {
      debug_display,
      player_camera: &self.player_camera,
      blocks: &self.blocks,
      target_block_index,
    }
  }
}

pub struct DebugDisplay {
  pub frames_per_second: u32,
  pub mean_frame_time_ms: f32,
}

pub struct Scene<'a> {
  pub debug_display: Option<DebugDisplay>,

  pub player_camera: &'a Camera,

  pub blocks: &'a Vec<Vec3>,
  pub target_block_index: Option<usize>,
}
