use winit::window::{CursorGrabMode, Window};

#[derive(Default)]
pub struct CursorLock {
  #[cfg(not(target_family = "wasm"))]
  manual: bool,
}

impl CursorLock {
  pub fn hide_mouse(&self, window: &Window) {
    window.set_cursor_visible(false);
  }
}

#[cfg(not(target_family = "wasm"))]
impl CursorLock {
  pub fn try_lock(&mut self, window: &Window) {
    let result = window
      .set_cursor_grab(CursorGrabMode::Confined)
      .or_else(|_| window.set_cursor_grab(CursorGrabMode::Locked));

    self.manual = result.is_err();
  }

  pub fn update_position(&self, window: &Window) {
    use winit::dpi::PhysicalPosition;

    if !self.manual {
      return;
    }

    let window_size = window.inner_size();
    let _ = window.set_cursor_position(PhysicalPosition::new(
      window_size.width / 2,
      window_size.height / 2,
    ));
  }

  pub fn try_user_requested_lock(&self, _window: &Window) {}
}

#[cfg(target_family = "wasm")]
impl CursorLock {
  pub fn try_lock(&self, _window: &Window) {}

  pub fn update_position(&self, _window: &Window) {}

  pub fn try_user_requested_lock(&self, window: &Window) {
    let _ = window.set_cursor_grab(CursorGrabMode::Locked);
  }
}
