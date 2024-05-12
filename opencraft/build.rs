fn main() {
  const ASSETS_PATH: &str = "../assets";
  println!("cargo:rerun-if-changed={}", ASSETS_PATH);
  omnicopy_to_output::copy_to_output(ASSETS_PATH).expect("could not copy assets to target");
}
