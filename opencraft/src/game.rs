use crate::camera::Direction;
use crate::core::math::vec2::Vec2;
use crate::model::{Model, UpdateInputs};
use crate::platform::Instant;
use crate::renderer::Renderer;
use anyhow::Result;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use winit::dpi::PhysicalSize;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;
use winit::window::Window;

pub struct Game {
  last: Instant,

  keys_down: HashSet<KeyCode>,
  keys_released: HashSet<KeyCode>,

  mouse_movement: Vec2,
  mouse_buttons_released: HashSet<MouseButton>,

  model: Model,
  renderer: Renderer,
}

impl Game {
  pub async fn new(window: Arc<Window>) -> Result<Self> {
    Ok(Self {
      last: Instant::now(),
      keys_down: HashSet::new(),
      keys_released: HashSet::new(),
      mouse_movement: Vec2::default(),
      mouse_buttons_released: HashSet::new(),
      model: Model::new(),
      renderer: Renderer::new(window).await?,
    })
  }

  pub fn resize(&mut self, size: PhysicalSize<u32>) {
    self.renderer.resize(size);
  }

  pub fn compose(&mut self) -> Result<()> {
    let elapsed = self.last.elapsed();
    self.last = Instant::now();

    self.update(elapsed);
    self.render()?;

    self.model.clear_changes();

    Ok(())
  }

  pub fn press(&mut self, code: KeyCode) {
    self.keys_down.insert(code);
  }

  pub fn release(&mut self, code: KeyCode) {
    self.keys_down.remove(&code);
    self.keys_released.insert(code);
  }

  pub fn mouse_release(&mut self, button: MouseButton) {
    self.mouse_buttons_released.insert(button);
  }

  pub fn motion(&mut self, direction: Vec2) {
    self.mouse_movement += direction / self.renderer.screen_size();
  }

  fn update(&mut self, delta: Duration) {
    self.model.update(&UpdateInputs {
      delta,
      keys_down: &self.keys_down,
      keys_released: &self.keys_released,
      mouse_movement: self.mouse_movement,
      mouse_buttons_released: &self.mouse_buttons_released,
    });

    self.keys_released.clear();
    self.mouse_movement = Vec2::default();
    self.mouse_buttons_released.clear();
  }

  fn render(&mut self) -> Result<()> {
    let scene = self.model.scene();
    let view_direction = if self.keys_down.contains(&KeyCode::KeyC) {
      Direction::Backward
    } else {
      Direction::Forward
    };

    self.renderer.render(&scene, view_direction)?;

    Ok(())
  }
}
