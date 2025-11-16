#[cfg(not(target_family = "wasm"))]
mod desktop;
#[cfg(target_family = "wasm")]
mod web;

#[cfg(not(target_family = "wasm"))]
pub use desktop::*;
#[cfg(target_family = "wasm")]
pub use web::*;
