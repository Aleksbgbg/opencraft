use winit::window::{Fullscreen, Window};

pub fn toggle(window: &Window) {
  if window.fullscreen().is_some() {
    disable(window);
  } else {
    enable(window);
  }
}

fn disable(window: &Window) {
  window.set_fullscreen(None);
}

fn enable(window: &Window) {
  if cfg!(target_family = "wasm") {
    window.set_fullscreen(Some(Fullscreen::Borderless(None)));
  } else if cfg!(any(target_os = "macos", unix)) {
    if let Some(monitor) = window.current_monitor() {
      window.set_fullscreen(Some(Fullscreen::Borderless(Some(monitor))));
    }
  } else if let Some(monitor) = window.current_monitor()
    && let Some(video_mode) = monitor.video_modes().next()
  {
    window.set_fullscreen(Some(Fullscreen::Exclusive(video_mode)));
  }
}
