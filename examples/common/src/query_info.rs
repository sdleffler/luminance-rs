//! This program demonstrates the Query API, allowing to get information about the GPU and the backend. This example
//! outputs the information via the `log` crate via `log::info`, so don’t forget to enable information level in the
//! executor you choose.
//!
//! <https://docs.rs/luminance>

use crate::{Example, InputAction, LoopFeedback, PlatformServices};
use luminance_front::{context::GraphicsContext, framebuffer::Framebuffer, texture::Dim2, Backend};

pub struct LocalExample;

impl Example for LocalExample {
  fn bootstrap(
    _: &mut impl PlatformServices,
    context: &mut impl GraphicsContext<Backend = Backend>,
  ) -> Self {
    let q = context.query();

    log::info!("Backend author: {:?}", q.backend_author());
    log::info!("Backend name: {:?}", q.backend_name());
    log::info!("Backend version: {:?}", q.backend_version());
    log::info!(
      "Backend shading language version: {:?}",
      q.backend_shading_lang_version()
    );
    log::info!(
      "Maximum number of elements in a texture array: {:?}",
      q.max_texture_array_elements()
    );

    LocalExample
  }

  fn render_frame(
    self,
    _: f32,
    _: Framebuffer<Dim2, (), ()>,
    _: impl Iterator<Item = InputAction>,
    _: &mut impl GraphicsContext<Backend = Backend>,
  ) -> LoopFeedback<Self> {
    LoopFeedback::Exit
  }
}
