//! Platform services implementation.

use image::ImageError;
use luminance_examples::PlatformServices;
use std::{error::Error, fmt};

/// Web implementation of the [`PlatformService`] API.
#[derive(Debug)]
pub struct WebPlatformServices {
  textures: Vec<image::RgbImage>,
}

impl WebPlatformServices {
  pub fn new() -> Self {
    let textures = Vec::new();
    Self { textures }
  }

  pub fn add_texture(&mut self, blob: Vec<u8>) {
    match image::load_from_memory(&blob) {
      Err(err) => log::error!("cannot read texture {}", err),
      Ok(img) => {
        log::info!("added a new texture");
        self.textures.push(img.flipv().into_rgb8());
      }
    }
  }
}

#[derive(Debug)]
pub enum WebFetchError {
  NoMoreTexture,
  ImageError(ImageError),
}

impl fmt::Display for WebFetchError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      WebFetchError::NoMoreTexture => f.write_str("no more texture, sorry"),
      WebFetchError::ImageError(ref e) => write!(f, "cannot fetch texture: {}", e),
    }
  }
}

impl Error for WebFetchError {}

impl From<ImageError> for WebFetchError {
  fn from(source: ImageError) -> Self {
    Self::ImageError(source)
  }
}

impl PlatformServices for WebPlatformServices {
  type FetchError = WebFetchError;

  fn fetch_texture(&mut self) -> Result<image::RgbImage, Self::FetchError> {
    if self.textures.is_empty() {
      Err(WebFetchError::NoMoreTexture)
    } else {
      Ok(self.textures.remove(0)) // bit of a cost but for small textures who cares?
    }
  }
}
