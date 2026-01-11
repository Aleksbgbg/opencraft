use base::texture;
use base::texture::block::{PANE_DIMENSION_PX, PANE_PIXELS};
use base::texture::{Srgb, Srgba};
use clap::Parser;
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{ExtendedColorType, ImageEncoder};
use itertools::Itertools;
use rand::seq::{IteratorRandom, SliceRandom};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::Deserialize;
use std::collections::HashMap;
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

/// Seed to be used for a random number generator.
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
  seed: Option<Seed>,
  palette: Vec<PaletteEntry>,
}

#[derive(Deserialize)]
struct BaseStoneSpecification {
  seed: Option<Seed>,
  palette: Vec<PaletteEntry>,
}

#[derive(Deserialize)]
struct Specification {
  /// Parameters to generate a plain block, which has the same texture on all
  /// faces.
  ///
  /// Examples include dirt and sand.
  plain: Option<PlainSpecification>,

  /// Parameters to generate a base stone block, which has the same texture on
  /// all faces. Base stones are distinguished by the distinct regular lines
  /// in their textures.
  ///
  /// Examples include stone and bedrock.
  base_stone: Option<BaseStoneSpecification>,
}

impl Specification {
  fn extract_generator(&self) -> &dyn Generator {
    if let Some(plain) = &self.plain {
      plain
    } else {
      self
        .base_stone
        .as_ref()
        .expect("exactly one block specification must be provided")
    }
  }
}

fn main() {
  let args = Args::parse();
  let spec: Specification = toml::from_str(
    &fs::read_to_string(&args.specification).expect("could not read specification file"),
  )
  .expect("could not parse specification");

  let generator = spec.extract_generator();
  if !generator.validate() {
    return;
  }

  let texture = generator.generate();

  let mut file = File::create(&args.destination).expect("could not create destination file");
  PngEncoder::new_with_quality(&mut file, CompressionType::Level(9), FilterType::default())
    .write_image(
      texture.rgba.as_bytes(),
      texture.width.try_into().unwrap(),
      texture.height.try_into().unwrap(),
      ExtendedColorType::Rgba8,
    )
    .expect("could not save texture");

  println!(
    "Generated block texture with seed(s) {} and saved to file {}.",
    texture
      .seeds
      .iter()
      .map(|seed| format!("{:x?}", seed))
      .join(", "),
    args.destination.display()
  );
}

fn validate_palette_specifies_all_pixels(palette: &[PaletteEntry]) -> bool {
  let palette_pixels: usize = palette
    .iter()
    .map(|pixel| pixel.frequency)
    .sum::<u32>()
    .try_into()
    .unwrap();
  if palette_pixels == PANE_PIXELS {
    true
  } else {
    eprintln!(
      "Palette should provide exactly {} pixels but provided {} pixels.",
      PANE_PIXELS, palette_pixels
    );
    false
  }
}

fn create_rng(seed: Option<&Seed>) -> (ChaCha8Rng, [u8; SEED_BYTES]) {
  let seed = seed.map(|seed| seed.value).unwrap_or_else(|| {
    let mut bytes = [0; SEED_BYTES];
    rand::fill(&mut bytes);
    bytes
  });

  (ChaCha8Rng::from_seed(seed), seed)
}

fn randomise_pane(palette: &[PaletteEntry], rng: &mut ChaCha8Rng) -> [Srgb; PANE_PIXELS] {
  let mut pane = [Srgb::default(); PANE_PIXELS];
  let mut pixel_index = 0;
  // Sort the palette to ensure that rearranging the TOML file layout without
  // changing the colours, frequency, or seed will still generate the same
  // image.
  for entry in palette.iter().sorted_by_key(|entry| entry.colour) {
    let frequency = entry.frequency.try_into().unwrap();

    for pixel_offset in 0..frequency {
      pane[pixel_index + pixel_offset] = entry.colour;
    }

    pixel_index += frequency;
  }

  pane.shuffle(rng);

  pane
}

struct GeneratedImage {
  rgba: Vec<Srgba>,
  width: usize,
  height: usize,

  seeds: Vec<[u8; SEED_BYTES]>,
}

trait Generator {
  fn validate(&self) -> bool;

  fn generate(&self) -> GeneratedImage;
}

impl Generator for PlainSpecification {
  fn validate(&self) -> bool {
    validate_palette_specifies_all_pixels(&self.palette)
  }

  fn generate(&self) -> GeneratedImage {
    let (mut rng, seed) = create_rng(self.seed.as_ref());
    let pane_src = randomise_pane(&self.palette, &mut rng);

    let mut pane_dst = vec![Srgba::default(); PANE_PIXELS];
    for index in 0..pane_src.len() {
      pane_dst[index] = pane_src[index].into();
    }

    GeneratedImage {
      rgba: pane_dst,
      width: PANE_DIMENSION_PX,
      height: PANE_DIMENSION_PX,
      seeds: vec![seed],
    }
  }
}

impl Generator for BaseStoneSpecification {
  fn validate(&self) -> bool {
    validate_palette_specifies_all_pixels(&self.palette)
  }

  fn generate(&self) -> GeneratedImage {
    let (mut rng, seed) = create_rng(self.seed.as_ref());
    let mut pane_src = randomise_pane(&self.palette, &mut rng);

    // Group colours together in random run lengths to produce the distinct lines on
    // base stone textures.
    for line in 0..PANE_DIMENSION_PX {
      let mut frequency_map = HashMap::new();
      for pixel in 0..PANE_DIMENSION_PX {
        let colour = pane_src
          .get(texture::offset_2d((pixel, line), PANE_DIMENSION_PX))
          .unwrap();

        *frequency_map.entry(colour).or_insert(0_usize) += 1;
      }

      let mut new_line = [Srgb::default(); PANE_DIMENSION_PX];
      let mut offset = 0;
      while !frequency_map.is_empty() {
        // HashMap keys must be sorted before picking a random key, as their order is
        // otherwise non-deterministic and results cannot be reproduced from the
        // random seed.
        let colour = **frequency_map.keys().sorted().choose(&mut rng).unwrap();
        let colour_remaining_frequency = frequency_map.get_mut(&colour).unwrap();

        let repeats = rng.random_range(1..=*colour_remaining_frequency);

        for _ in 0..repeats {
          new_line[offset] = colour;
          offset += 1;
        }

        *colour_remaining_frequency -= repeats;

        if *colour_remaining_frequency == 0 {
          frequency_map.remove(&colour);
        }
      }

      let line_start = texture::offset_2d((0, line), PANE_DIMENSION_PX);
      pane_src[line_start..line_start + PANE_DIMENSION_PX].copy_from_slice(&new_line);
    }

    let mut pane_dst = vec![Srgba::default(); PANE_PIXELS];
    for index in 0..pane_src.len() {
      pane_dst[index] = pane_src[index].into();
    }

    GeneratedImage {
      rgba: pane_dst,
      width: PANE_DIMENSION_PX,
      height: PANE_DIMENSION_PX,
      seeds: vec![seed],
    }
  }
}
