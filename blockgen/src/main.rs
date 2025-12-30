use base::texture::block::{PANE_DIMENSION_PX_U32, PANE_PIXELS};
use base::texture::{Srgb, Srgba};
use clap::Parser;
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{ExtendedColorType, ImageEncoder};
use itertools::Itertools;
use rand::SeedableRng;
use rand::seq::SliceRandom;
use rand_chacha::ChaCha8Rng;
use serde::Deserialize;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use zerocopy::IntoBytes;

const SEED_BYTES: usize = 32;

#[derive(Parser)]
struct Args {
  /// Path to the TOML file which declares the parameters of the block texture
  /// to generate.
  #[arg(short, long)]
  specification: PathBuf,

  /// Path to the PNG file where to store the generated block texture.
  #[arg(short, long)]
  destination: PathBuf,
}

#[derive(Deserialize)]
struct Seed {
  value: [u8; SEED_BYTES],
}

#[derive(Clone, Deserialize)]
struct PaletteEntry {
  colour: Srgb,
  frequency: u32,
}

#[derive(Deserialize)]
struct PlainSpecification {
  palette: Vec<PaletteEntry>,
}

#[derive(Deserialize)]
struct Specification {
  /// Seed to be used for the random number generator.
  seed: Option<Seed>,

  /// Parameters to generate a plain block, which has the same texture on all
  /// faces.
  ///
  /// Examples include dirt and sand.
  plain: Option<PlainSpecification>,
}

fn main() {
  let args = Args::parse();
  let spec: Specification = toml::from_str(
    &fs::read_to_string(&args.specification).expect("could not read specification file"),
  )
  .expect("could not parse specification");

  let plain = spec
    .plain
    .as_ref()
    .expect("plain block specification is required");
  let palette = &plain.palette;

  let palette_pixels: usize = palette
    .iter()
    .map(|pixel| pixel.frequency)
    .sum::<u32>()
    .try_into()
    .unwrap();
  if palette_pixels != PANE_PIXELS {
    eprintln!(
      "Palette should provide exactly {} pixels but provided {} pixels.",
      PANE_PIXELS, palette_pixels
    );
    return;
  }

  let mut pane_src = [Srgb::default(); PANE_PIXELS];
  let mut pixel_index = 0;
  // Sort the palette to ensure that rearranging the TOML file layout without
  // changing the colours, frequency, or seed will still generate the same
  // image.
  for entry in palette.iter().sorted_by_key(|entry| entry.colour) {
    let frequency = entry.frequency.try_into().unwrap();

    for pixel_offset in 0..frequency {
      pane_src[pixel_index + pixel_offset] = entry.colour;
    }

    pixel_index += frequency;
  }

  let seed = spec.seed.map(|seed| seed.value).unwrap_or_else(|| {
    let mut bytes = [0; SEED_BYTES];
    rand::fill(&mut bytes);
    bytes
  });
  let mut rng = ChaCha8Rng::from_seed(seed);
  pane_src.shuffle(&mut rng);

  let mut pane_dst = [Srgba::default(); PANE_PIXELS];
  for index in 0..pane_src.len() {
    pane_dst[index] = pane_src[index].into();
  }

  let mut file = File::create(&args.destination).expect("could not create destination file");
  PngEncoder::new_with_quality(&mut file, CompressionType::Level(9), FilterType::default())
    .write_image(
      pane_dst.as_bytes(),
      PANE_DIMENSION_PX_U32,
      PANE_DIMENSION_PX_U32,
      ExtendedColorType::Rgba8,
    )
    .expect("could not save texture");

  println!(
    "Generated block texture with seed {:x?} and saved to file {}.",
    seed,
    args.destination.display()
  );
}
