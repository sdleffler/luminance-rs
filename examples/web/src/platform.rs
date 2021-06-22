//! Platform services implementation.

use image::ImageError;
use luminance_examples::PlatformServices;
use std::{collections::HashMap, error::Error, fmt};

/// Web implementation of the [`PlatformService`] API.
#[derive(Debug)]
pub struct WebPlatformServices {
  textures: HashMap<String, image::RgbImage>,
}

impl WebPlatformServices {
  pub fn new() -> Self {
    let textures = HashMap::new();
    Self { textures }
  }

  pub fn add_texture(&mut self, name: impl Into<String>, blob: Vec<u8>) {
    let name = name.into();

    match image::load_from_memory(&blob) {
      Err(err) => log::error!("cannot read texture {}: {}", name, err),
      Ok(img) => {
        log::info!("loaded texture {}", name);
        self.textures.insert(name, img.flipv().into_rgb8());
      }
    }
  }
}

#[derive(Debug)]
pub enum WebFetchError {
  ImageError(ImageError),
  UnknownTexture(String),
}

impl fmt::Display for WebFetchError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      WebFetchError::ImageError(ref e) => write!(f, "cannot fetch texture: {}", e),
      WebFetchError::UnknownTexture(ref name) => write!(f, "unknown texture: {}", name),
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

  fn fetch_texture(&mut self, name: impl AsRef<str>) -> Result<&image::RgbImage, Self::FetchError> {
    let path = name.as_ref();
    self
      .textures
      .get(path)
      .ok_or_else(|| WebFetchError::UnknownTexture(path.to_owned()))
  }
}
