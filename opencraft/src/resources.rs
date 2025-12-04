use crate::platform::ResourceReader;
use anyhow::Result;
use image::DynamicImage;
use image::codecs::png::PngDecoder;
use std::io::Cursor;

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
}
