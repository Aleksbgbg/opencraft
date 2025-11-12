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
