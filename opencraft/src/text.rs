use crate::core::math;
use crate::core::math::vec2::Vec2;
use crate::core::type_conversions::{Coerce, CoerceLossy};
use crate::platform::ResourceReader;
use anyhow::Result;
use rusttype::Scale;
use std::collections::HashMap;
use winit::dpi::PhysicalSize;
use zerocopy::{Immutable, IntoBytes};

const SPACE: char = ' ';
const FALLBACK_CHARACTER: char = '\0';
const ALPHABET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789.:()\0";

#[repr(C)]
#[derive(Immutable, IntoBytes)]
pub struct TextVertex {
  screen_position: Vec2,
  texture_position: Vec2,
}

struct GlyphMapping {
  texture_offset: Vec2,
  pixel_width: f32,
}

pub struct FontAtlas {
  glyphs: HashMap<char, GlyphMapping>,
  glyph_pixel_height: f32,
  space_pixel_width: f32,
  texture_size: Vec2,
}

impl FontAtlas {
  pub async fn load(reader: &ResourceReader, scale: f32) -> Result<(Self, Vec<u8>)> {
    let scale = Scale::uniform(scale);

    let font = reader.load_font().await?;

    let pixel_width_float: f32 = font
      .glyphs_for(ALPHABET.chars())
      .map(|glyph| glyph.scaled(scale).h_metrics().advance_width.ceil())
      .sum();
    let v_metrics = font.v_metrics(scale);
    let ascent_abs = v_metrics.ascent.abs();
    let descent_abs = v_metrics.descent.abs();
    let pixel_height_float = (ascent_abs + 1.0 + descent_abs).ceil();

    let pixel_width: usize = pixel_width_float.coerce_lossy();
    let pixel_height: usize = pixel_height_float.coerce_lossy();

    let mut texture = vec![0; pixel_width * pixel_height];
    let texture_size = Vec2::new(pixel_width_float, pixel_height_float);

    let mut glyphs = HashMap::new();
    let mut x_position = 0.0;
    for char in ALPHABET.chars() {
      let scaled = font.glyph(char).scaled(scale);
      let glyph_pixel_width = scaled.h_metrics().advance_width.ceil();

      let positioned = scaled.positioned(rusttype::point(x_position, ascent_abs));
      let bounding_box = positioned.pixel_bounding_box().unwrap();
      positioned.draw(|x, y, alpha| {
        let x = bounding_box.min.x.coerce() + x.coerce();
        let y = bounding_box.min.y.coerce() + y.coerce();

        texture[(y * pixel_width) + x] = math::normalized_f32_to_u8(alpha);
      });

      glyphs.insert(
        char,
        GlyphMapping {
          texture_offset: Vec2::new(x_position, 0.0).normalise_components_to(texture_size),
          pixel_width: glyph_pixel_width,
        },
      );

      x_position += glyph_pixel_width;
    }

    let space_pixel_width = font
      .glyph(SPACE)
      .scaled(scale)
      .h_metrics()
      .advance_width
      .ceil();

    Ok((
      Self {
        glyphs,
        glyph_pixel_height: pixel_height_float,
        space_pixel_width,
        texture_size,
      },
      texture,
    ))
  }

  fn get_glyph_mapping(&self, char: char) -> &GlyphMapping {
    self
      .glyphs
      .get(&char)
      .unwrap_or_else(|| self.glyphs.get(&FALLBACK_CHARACTER).unwrap())
  }

  pub fn dimensions(&self) -> (u32, u32) {
    (
      self.texture_size.x().coerce_lossy(),
      self.texture_size.y().coerce_lossy(),
    )
  }

  pub fn push_text_vertices(
    &self,
    text: &str,
    offset: PhysicalSize<u32>,
    screen_size: PhysicalSize<u32>,
    vertices: &mut Vec<TextVertex>,
  ) {
    let offset = Vec2::new(offset.width.coerce_lossy(), offset.height.coerce_lossy());
    let screen_size = Vec2::new(
      screen_size.width.coerce_lossy(),
      screen_size.height.coerce_lossy(),
    );

    let mut x_position = 0.0;
    for char in text.chars() {
      if char == SPACE {
        x_position += self.space_pixel_width;
        continue;
      }

      let mapping = self.get_glyph_mapping(char);

      let size = Vec2::new(mapping.pixel_width, self.glyph_pixel_height);

      let screen_offset = Vec2::new(
        math::affine_transform(offset.x() + x_position, 0.0..=screen_size.x(), -1.0..=1.0),
        -math::affine_transform(offset.y(), 0.0..=screen_size.y(), -1.0..=1.0),
      );
      let screen_size = Vec2::new(
        math::affine_transform(size.x(), 0.0..=screen_size.x(), 0.0..=2.0),
        -math::affine_transform(size.y(), 0.0..=screen_size.y(), 0.0..=2.0),
      );

      let screen_x1 = screen_offset.x();
      let screen_x2 = screen_offset.x() + screen_size.x();
      let screen_y1 = screen_offset.y();
      let screen_y2 = screen_offset.y() + screen_size.y();

      let texture_size = size.normalise_components_to(self.texture_size);

      let texture_x1 = mapping.texture_offset.x();
      let texture_x2 = mapping.texture_offset.x() + texture_size.x();
      let texture_y1 = mapping.texture_offset.y();
      let texture_y2 = mapping.texture_offset.y() + texture_size.y();

      vertices.extend([
        TextVertex {
          screen_position: Vec2::new(screen_x1, screen_y1),
          texture_position: Vec2::new(texture_x1, texture_y1),
        },
        TextVertex {
          screen_position: Vec2::new(screen_x1, screen_y2),
          texture_position: Vec2::new(texture_x1, texture_y2),
        },
        TextVertex {
          screen_position: Vec2::new(screen_x2, screen_y1),
          texture_position: Vec2::new(texture_x2, texture_y1),
        },
        TextVertex {
          screen_position: Vec2::new(screen_x2, screen_y1),
          texture_position: Vec2::new(texture_x2, texture_y1),
        },
        TextVertex {
          screen_position: Vec2::new(screen_x1, screen_y2),
          texture_position: Vec2::new(texture_x1, texture_y2),
        },
        TextVertex {
          screen_position: Vec2::new(screen_x2, screen_y2),
          texture_position: Vec2::new(texture_x2, texture_y2),
        },
      ]);

      x_position += size.x();
    }
  }
}
