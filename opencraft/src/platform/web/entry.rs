use wasm_bindgen::UnwrapThrowExt;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(start)]
fn main() {
  console_error_panic_hook::set_once();

  crate::start().expect_throw("error during app startup");
}
