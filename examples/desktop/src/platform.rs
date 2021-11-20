//! Platform services implementation.

use crate::CLIOpts;
use image::ImageError;
use luminance_examples::PlatformServices;
use std::{error::Error, fmt};

/// Desktop implementation of the [`PlatformServices`] API.
#[derive(Debug)]
pub struct DesktopPlatformServices {
  textures: Vec<image::RgbImage>,
}

impl DesktopPlatformServices {
  pub fn new(cli_opts: CLIOpts) -> Self {
    let textures = cli_opts.textures;

    if textures.is_empty() {
      Self {
        textures: Vec::new(),
      }
    } else {
      let textures = textures
        .into_iter()
        .map(|path| {
          image::open(&path)
            .map(|img| img.flipv().to_rgb8())
            .expect(&format!("image {}", path))
        })
        .collect();

      Self { textures }
    }
  }
}

#[derive(Debug)]
pub enum DesktopFetchError {
  NoMoreTexture,
  ImageError(ImageError),
}

impl fmt::Display for DesktopFetchError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      DesktopFetchError::NoMoreTexture => f.write_str("no more texture, sorry"),
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

  fn fetch_texture(&mut self) -> Result<image::RgbImage, Self::FetchError> {
    if self.textures.is_empty() {
      Err(DesktopFetchError::NoMoreTexture)
    } else {
      Ok(self.textures.remove(0)) // bit of a cost but for small textures who cares?
    }
  }
}
