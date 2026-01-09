use crate::core::math::intersect::BoxFace;
use crate::core::type_conversions::{Coerce, CoerceLossy};
use crate::model::block::Block;
use crate::platform::ResourceReader;
use crate::renderer::Quad;
use crate::resources::Texture;
use anyhow::Result;
use base::texture;
use base::texture::Srgba;
use base::texture::block::{PANE_DIMENSION_PX, PANE_DIMENSION_PX_F32};
use image::GenericImageView;
use std::collections::HashMap;
use zerocopy::FromBytes;

const BLOCK_TEXTURE_PANES: &[(Block, Texture, usize)] = &[
  (Block::Bedrock, Texture::Bedrock, 1),
  (Block::Stone, Texture::Stone, 1),
  (Block::Dirt, Texture::Dirt, 1),
  (Block::Grass, Texture::Grass, 3),
];

fn pane_to_faces(pane: usize, panes: usize) -> &'static [BoxFace] {
  match panes {
    1 => &[
      BoxFace::XPos,
      BoxFace::XNeg,
      BoxFace::YPos,
      BoxFace::YNeg,
      BoxFace::ZPos,
      BoxFace::ZNeg,
    ],
    3 => match pane {
      0 => &[BoxFace::YPos],
      1 => &[BoxFace::XPos, BoxFace::XNeg, BoxFace::ZPos, BoxFace::ZNeg],
      2 => &[BoxFace::YNeg],
      _ => unreachable!("pane index {} is out of bounds (panes = {})", pane, panes),
    },
    _ => unreachable!("textures consisting of {} panes are invalid", panes),
  }
}

pub struct TextureAtlasImage {
  pub rgba: Vec<Srgba>,
  pub width: u32,
  pub height: u32,
}

pub struct TextureAtlas {
  coordinates: HashMap<(Block, BoxFace), Quad>,
}

impl TextureAtlas {
  // Used in benchmarks
  pub fn grass_only() -> Self {
    Self {
      coordinates: HashMap::from([
        ((Block::Grass, BoxFace::XPos), Quad::default()),
        ((Block::Grass, BoxFace::XNeg), Quad::default()),
        ((Block::Grass, BoxFace::YPos), Quad::default()),
        ((Block::Grass, BoxFace::YNeg), Quad::default()),
        ((Block::Grass, BoxFace::ZPos), Quad::default()),
        ((Block::Grass, BoxFace::ZNeg), Quad::default()),
      ]),
    }
  }

  pub async fn load(assets: &ResourceReader) -> Result<(Self, TextureAtlasImage)> {
    let panes: usize = BLOCK_TEXTURE_PANES.iter().map(|(_, _, panes)| panes).sum();

    let width = panes * PANE_DIMENSION_PX;
    let height = PANE_DIMENSION_PX;

    let mut atlas_rgba = vec![Srgba::default(); width * height];

    let mut coordinates = HashMap::with_capacity(BLOCK_TEXTURE_PANES.len() * 6);

    let width_f32 = width.coerce_lossy();

    let mut current_pane = 0;
    for &(block, texture, panes) in BLOCK_TEXTURE_PANES {
      let image = assets.load_texture(texture).await?;
      let image_rgba = image.to_rgba8();
      let (image_width, image_height) = image.dimensions();

      for pane in 0..panes {
        let atlas_offset_x = current_pane * PANE_DIMENSION_PX;
        texture::transfer_block(
          &mut atlas_rgba,
          (atlas_offset_x, 0),
          width,
          <[Srgba]>::ref_from_bytes_with_elems(&image_rgba, (image_width * image_height).coerce())
            .unwrap(),
          (0, pane * PANE_DIMENSION_PX),
          image_width.coerce(),
          PANE_DIMENSION_PX,
        );

        let atlas_offset_x_f32 = atlas_offset_x.coerce_lossy();
        let quad = Quad {
          left: atlas_offset_x_f32 / width_f32,
          right: (atlas_offset_x_f32 + PANE_DIMENSION_PX_F32) / width_f32,
          top: 0.0,
          bot: 1.0,
        };
        for &face in pane_to_faces(pane, panes) {
          coordinates.insert((block, face), quad);
        }

        current_pane += 1;
      }
    }

    Ok((
      Self { coordinates },
      TextureAtlasImage {
        rgba: atlas_rgba,
        width: width.coerce(),
        height: height.coerce(),
      },
    ))
  }

  pub fn generate_texture_coordinates(&self, block: Block, face: BoxFace) -> Quad {
    self.coordinates.get(&(block, face)).copied().unwrap()
  }
}
