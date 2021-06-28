use crate::{shared::Vertex, Example, InputAction, LoopFeedback, PlatformServices};
use luminance_front::{context::GraphicsContext, framebuffer::Framebuffer, texture::Dim2, Backend};
use std::ops::Deref as _;

pub struct LocalExample;

impl Example for LocalExample {
  fn bootstrap(
    _: &mut impl PlatformServices,
    context: &mut impl GraphicsContext<Backend = Backend>,
  ) -> Self {
    let vertices = [
      Vertex::new([1., 2.].into(), [1., 1., 1.].into()),
      Vertex::new([-1., 2.].into(), [1., 0., 1.].into()),
      Vertex::new([1., -2.].into(), [1., 1., 0.].into()),
    ];
    let mut tess = context
      .new_tess()
      .set_vertices(&vertices[..])
      .set_indices([0u8, 1, 2])
      .set_mode(luminance_front::tess::Mode::Point)
      .build()
      .expect("tessellation");

    {
      let mut slice = tess.indices_mut().expect("sliced indices");
      log::info!("slice before mutation is: {:?}", slice.deref());

      slice[1] = 2;
      log::info!("slice after mutation is:  {:?}", slice.deref());
    }

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
