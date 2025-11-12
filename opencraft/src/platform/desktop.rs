use anyhow::Result;
use std::time::Duration;
use std::{fs, thread};
use winit::window::WindowAttributes;

#[rustfmt::skip]
#[allow(unused_imports)]
mod log_macros {
  pub use std::eprintln as error;
  pub use std::println as warn;
  pub use std::println as info;
  pub use std::println as debug;
  pub use std::println as trace;
}

pub use log_macros::*;

pub fn init_logging() {
  env_logger::init();
}

pub fn init_window_attributes(window_attributes: WindowAttributes) -> WindowAttributes {
  window_attributes
}

pub fn run_future<F>(future: F)
where
  F: Future<Output = ()> + 'static,
{
  pollster::block_on(future);
}

// Note that, as per `run_future`, futures on desktop platforms block so using a
// blocking sleep is not a problem.
pub async fn sleep(duration: Duration) {
  thread::sleep(duration)
}

pub async fn read_resource(path: &str) -> Result<Vec<u8>> {
  Ok(fs::read(path)?)
}
