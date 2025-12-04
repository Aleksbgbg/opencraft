use crate::platform::ResourceReader;
use anyhow::Result;
use image::DynamicImage;
use image::codecs::png::PngDecoder;
use std::io::Cursor;

impl ResourceReader {
  pub async fn decode_png(&self, path: &str) -> Result<DynamicImage> {
    let image_data = Cursor::new(self.read(path).await?);
    let decoder = PngDecoder::new(image_data)?;

    Ok(DynamicImage::from_decoder(decoder)?)
  }
}
