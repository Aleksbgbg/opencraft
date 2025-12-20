use crate::platform::ResourceReader;
use anyhow::{Error, Result};
use image::DynamicImage;
use image::codecs::png::PngDecoder;
use rusttype::Font;
use std::io::Cursor;

const MONOCRAFT_WOFF2_PATH: &str = "fonts/monocraft_v4.2.1.woff2";

#[derive(Clone, Copy)]
pub enum Texture {
  Grass,
  Crosshair,
}

impl Texture {
  fn path(self) -> &'static str {
    match self {
      Texture::Grass => "textures/block/grass.png",
      Texture::Crosshair => "textures/ui/crosshair.png",
    }
  }
}

impl ResourceReader {
  async fn decode_png(&self, path: &str) -> Result<DynamicImage> {
    let image_data = Cursor::new(self.read(path).await?);
    let decoder = PngDecoder::new(image_data)?;

    Ok(DynamicImage::from_decoder(decoder)?)
  }

  pub async fn load_texture(&self, texture: Texture) -> Result<DynamicImage> {
    self.decode_png(texture.path()).await
  }

  pub async fn load_font(&self) -> Result<Font<'static>> {
    let woff2_data = self.read(MONOCRAFT_WOFF2_PATH).await?;
    let ttf_data = wuff::decompress_woff2(&woff2_data)?;

    Font::try_from_vec(ttf_data).ok_or_else(|| Error::msg("invalid font"))
  }
}
