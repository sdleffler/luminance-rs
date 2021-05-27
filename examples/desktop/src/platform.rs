//! Platform services implementation.

use crate::CLIOpts;
use image::ImageError;
use luminance_examples::{Features, PlatformServices};
use std::{collections::HashMap, error::Error, fmt};

/// Desktop implementation of the [`PlatformService`] API.
#[derive(Debug)]
pub struct DesktopPlatformServices {
  cli_opts: CLIOpts,
  textures: HashMap<String, image::RgbImage>,
}

impl DesktopPlatformServices {
  pub fn new(cli_opts: CLIOpts, features: Features) -> Self {
    let texture_root = cli_opts.textures.as_ref().expect("no texture root");
    let textures = features
      .textures()
      .iter()
      .map(|name| {
        let path = texture_root.join(name);
        let texture = image::open(&path)
          .map(|img| img.flipv().to_rgb8())
          .expect(&format!("image {}", path.display()));
        (name.clone(), texture)
      })
      .collect();

    Self { cli_opts, textures }
  }
}

#[derive(Debug)]
pub enum DesktopFetchError {
  UnknownTexture(String),
  ImageError(ImageError),
}

impl fmt::Display for DesktopFetchError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      DesktopFetchError::UnknownTexture(ref name) => write!(f, "unknown texture to load: {}", name),
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

  fn fetch_texture(&mut self, name: impl AsRef<str>) -> Result<&image::RgbImage, Self::FetchError> {
    let path = name.as_ref();
    self
      .textures
      .get(path)
      .ok_or_else(|| DesktopFetchError::UnknownTexture(path.to_owned()))
  }
}
