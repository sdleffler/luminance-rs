use crate::{Example, InputAction, LoopFeedback, PlatformServices};
use luminance_front::{
  context::GraphicsContext, framebuffer::Framebuffer, tess::TessError, texture::Dim2, Backend,
};

pub struct LocalExample;

impl Example for LocalExample {
  fn bootstrap(
    _: &mut impl PlatformServices,
    context: &mut impl GraphicsContext<Backend = Backend>,
  ) -> Self {
    let tess = context.new_tess().build();
    assert!(matches!(tess, Err(TessError::NoData)));
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
