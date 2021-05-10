//! Platform services implementation.

use std::{env, error::Error, fmt};

use image::ImageError;
use luminance_examples::PlatformServices;

/// Desktop implementation of the [`PlatformService`] API.
#[derive(Debug)]
pub struct DesktopPlatformServices {}

impl DesktopPlatformServices {
  pub fn new() -> Self {
    Self {}
  }
}

#[derive(Debug)]
pub enum DesktopFetchError {
  ImageError(ImageError),
}

impl fmt::Display for DesktopFetchError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      DesktopFetchError::ImageError(ref e) => write!(f, "cannot fetch texture: {}", e),
    }
  }
}

impl Error for DesktopFetchError {}

impl From<ImageError> for DesktopFetchError {
  fn from(source: ImageError) -> Self {
    Self::ImageError(source)
  }
}

impl PlatformServices for DesktopPlatformServices {
  type FetchError = DesktopFetchError;

  fn fetch_user_input_args(&mut self) -> Vec<String> {
    // we skip two because the 1st is the executable name and the 2nd is the example name
    env::args().skip(2).collect()
  }

  fn fetch_texture(&mut self, name: impl AsRef<str>) -> Result<image::RgbImage, Self::FetchError> {
    let path = name.as_ref();
    let img = image::open(path).map(|img| img.flipv().to_rgb8())?;
    Ok(img)
  }
}
