mod entry;

use anyhow::Result;
use gloo_timers::future::TimeoutFuture;
use log::Level;
use std::time::Duration;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::Window;
use winit::platform::web::WindowAttributesExtWebSys;
use winit::window::WindowAttributes;

#[rustfmt::skip]
#[allow(unused_imports)]
mod log_macros {
  pub use log::error;
  pub use log::warn;
  pub use log::info;
  pub use log::debug;
  pub use log::trace;
}

pub use log_macros::*;

pub type Instant = web_time::Instant;

pub fn init_logging() {
  console_log::init_with_level(Level::Info).expect_throw("could not initialise console logging");
}

fn window() -> Window {
  web_sys::window().expect_throw("could not get browser window object")
}

pub fn init_window_attributes(window_attributes: WindowAttributes) -> WindowAttributes {
  const CANVAS_ID: &str = "app";

  window_attributes.with_canvas(Some(
    window()
      .document()
      .expect_throw("could not get browser document object")
      .get_element_by_id(CANVAS_ID)
      .expect_throw("could not get canvas element by ID")
      .unchecked_into(),
  ))
}

pub fn run_future<F>(future: F)
where
  F: Future<Output = ()> + 'static,
{
  wasm_bindgen_futures::spawn_local(future)
}

pub async fn sleep(duration: Duration) {
  if duration.as_millis() > i32::MAX.try_into().unwrap_throw() {
    panic!("sleep duration in milliseconds must fit into i32");
  }

  TimeoutFuture::new(duration.as_millis().try_into().unwrap_throw()).await
}

pub struct ResourceReader {
  origin: String,
}

impl ResourceReader {
  pub fn new() -> Result<Self> {
    Ok(Self {
      origin: window()
        .location()
        .origin()
        .expect_throw("could not get browser URL origin"),
    })
  }

  pub async fn read(&self, path: &str) -> Result<Vec<u8>> {
    Ok(
      reqwest::get(format!("{}/assets/{}", self.origin, path))
        .await?
        .bytes()
        .await?
        .to_vec(),
    )
  }
}
