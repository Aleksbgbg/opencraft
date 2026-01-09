use crate::core::math::intersect::BoxFace;
use crate::platform::ResourceReader;
use crate::renderer::Quad;
use crate::resources::Texture;
use anyhow::Result;
use image::GenericImageView;

const TEX_WIDTH: f32 = 16.0;
const TEX_HEIGHT: f32 = 48.0;

const TEX_Y_POS_LEFT: f32 = 0.0 / TEX_WIDTH;
const TEX_Y_POS_RIGHT: f32 = 16.0 / TEX_WIDTH;
const TEX_Y_POS_TOP: f32 = 0.0 / TEX_HEIGHT;
const TEX_Y_POS_BOTTOM: f32 = 16.0 / TEX_HEIGHT;

const TEX_SIDE_LEFT: f32 = 0.0 / TEX_WIDTH;
const TEX_SIDE_RIGHT: f32 = 16.0 / TEX_WIDTH;
const TEX_SIDE_TOP: f32 = 16.0 / TEX_HEIGHT;
const TEX_SIDE_BOTTOM: f32 = 32.0 / TEX_HEIGHT;

const TEX_Y_NEG_LEFT: f32 = 0.0 / TEX_WIDTH;
const TEX_Y_NEG_RIGHT: f32 = 16.0 / TEX_WIDTH;
const TEX_Y_NEG_TOP: f32 = 32.0 / TEX_HEIGHT;
const TEX_Y_NEG_BOTTOM: f32 = 48.0 / TEX_HEIGHT;

pub struct TextureAtlasImage {
  pub rgba: Vec<u8>,
  pub width: u32,
  pub height: u32,
}

pub struct TextureAtlas {}

impl TextureAtlas {
  pub async fn load(assets: &ResourceReader) -> Result<(Self, TextureAtlasImage)> {
    let grass_image = assets.load_texture(Texture::Grass).await?;
    let grass_rgba = grass_image.to_rgba8();
    let (grass_width, grass_height) = grass_image.dimensions();

    Ok((
      Self {},
      TextureAtlasImage {
        rgba: grass_rgba.to_vec(),
        width: grass_width,
        height: grass_height,
      },
    ))
  }

  pub fn generate_texture_coordinates(&self, face: BoxFace) -> Quad {
    match face {
      BoxFace::YPos => Quad {
        left: TEX_Y_POS_LEFT,
        right: TEX_Y_POS_RIGHT,
        top: TEX_Y_POS_TOP,
        bot: TEX_Y_POS_BOTTOM,
      },
      BoxFace::YNeg => Quad {
        left: TEX_Y_NEG_LEFT,
        right: TEX_Y_NEG_RIGHT,
        top: TEX_Y_NEG_TOP,
        bot: TEX_Y_NEG_BOTTOM,
      },
      BoxFace::XPos | BoxFace::XNeg | BoxFace::ZPos | BoxFace::ZNeg => Quad {
        left: TEX_SIDE_LEFT,
        right: TEX_SIDE_RIGHT,
        top: TEX_SIDE_TOP,
        bot: TEX_SIDE_BOTTOM,
      },
    }
  }
}
