//! Platform services implementation.

use image::ImageError;
use luminance_examples::PlatformServices;
use std::{error::Error, fmt};

/// Web implementation of the [`PlatformService`] API.
#[derive(Debug)]
pub struct WebPlatformServices {}

impl WebPlatformServices {
  pub fn new() -> Self {
    Self {}
  }
}

#[derive(Debug)]
pub enum WebFetchError {
  ImageError(ImageError),
}

impl fmt::Display for WebFetchError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
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

  fn fetch_user_input_args(&mut self) -> Vec<String> {
    todo!()
  }

  fn fetch_texture(&mut self, name: impl AsRef<str>) -> Result<image::RgbImage, Self::FetchError> {
    todo!()
  }
}
