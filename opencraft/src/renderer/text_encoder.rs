use crate::core::type_conversions::Coerce;
use crate::renderer::font_atlas::{FontAtlas, TextVertex};
use winit::dpi::PhysicalSize;

enum Alignment {
  Left,
  Right,
}

#[derive(Default)]
pub struct Anchor {
  pub left: Option<u32>,
  pub right: Option<u32>,
  pub top: Option<u32>,
  pub bottom: Option<u32>,
}

pub struct TextEncoder<'a> {
  font_atlas: &'a FontAtlas,
  screen_size: PhysicalSize<u32>,

  vertices: Vec<TextVertex>,
}

impl<'a> TextEncoder<'a> {
  pub fn new(font_atlas: &'a FontAtlas, screen_size: PhysicalSize<u32>) -> Self {
    Self {
      font_atlas,
      screen_size,
      vertices: Vec::new(),
    }
  }

  pub fn push_text_block(&mut self, block: &[&str], anchor: Anchor) {
    assert!(!block.is_empty());

    let line_widths: Vec<_> = block
      .iter()
      .map(|line| self.font_atlas.measure_text_width(line))
      .collect();

    let total_width = line_widths.iter().copied().max().unwrap();
    let total_height = self.font_atlas.line_height() * Coerce::<u32>::coerce(block.len());

    let offset = {
      let x = if let Some(left) = anchor.left {
        left.coerce()
      } else if let Some(right) = anchor.right {
        Coerce::<i32>::coerce(self.screen_size.width)
          - Coerce::<i32>::coerce(right)
          - Coerce::<i32>::coerce(total_width)
      } else {
        0
      };
      let y = if let Some(top) = anchor.top {
        top.coerce()
      } else if let Some(bottom) = anchor.bottom {
        Coerce::<i32>::coerce(self.screen_size.height)
          - Coerce::<i32>::coerce(bottom)
          - Coerce::<i32>::coerce(total_height)
      } else {
        0
      };

      PhysicalSize::new(x, y)
    };
    let alignment = anchor.left.map_or(Alignment::Right, |_| Alignment::Left);

    for (index, line) in block.iter().enumerate() {
      let alignment_offset = match alignment {
        Alignment::Left => 0,
        Alignment::Right => total_width - line_widths[index],
      };

      self.font_atlas.push_text_vertices(
        line,
        PhysicalSize::new(
          offset.width + Coerce::<i32>::coerce(alignment_offset),
          offset.height
            + (Coerce::<i32>::coerce(index) * Coerce::<i32>::coerce(self.font_atlas.line_height())),
        ),
        self.screen_size,
        &mut self.vertices,
      );
    }
  }

  pub fn finish(self) -> Vec<TextVertex> {
    self.vertices
  }
}
