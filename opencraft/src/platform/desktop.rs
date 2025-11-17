use anyhow::{Result, bail};
use std::path::PathBuf;
use std::time::Duration;
use std::{env, fs, thread};
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

pub type Instant = std::time::Instant;

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

pub struct ResourceReader {
  assets: PathBuf,
}

impl ResourceReader {
  pub fn new() -> Result<Self> {
    let mut path = env::current_exe()?.parent().unwrap().to_owned();

    if cfg!(debug_assertions) {
      loop {
        path.push("assets");

        if fs::exists(&path)? {
          break;
        } else {
          assert!(path.pop());
          if !path.pop() {
            bail!("no assets folder found in any parent directories on the path to the executable");
          }
        }
      }
    } else {
      path.push("assets");

      if !fs::exists(&path)? {
        bail!("assets folder ({}) does not exist", path.display());
      }
    }

    Ok(Self { assets: path })
  }

  pub async fn read(&self, path: &str) -> Result<Vec<u8>> {
    Ok(fs::read(self.assets.join(path))?)
  }
}
