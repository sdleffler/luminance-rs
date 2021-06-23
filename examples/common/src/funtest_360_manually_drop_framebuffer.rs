//! <https://github.com/phaazon/luminance-rs/issues/360>
//!
//! When a framebuffer is created, dropped and a new one is created, the new framebuffer doesn’t
//! seem to behave correctly. At the time of #360, it was detected on Windows.
//!
//! Because this example simply creates a framebuffer, drops it and creates another one, nothing is
//! displayed and the window flashes on the screen. Run the application in a tool such as apitrace
//! or renderdoc to analyze it.

use crate::{Example, InputAction, LoopFeedback, PlatformServices};
use luminance_front::{
  context::GraphicsContext,
  framebuffer::Framebuffer,
  pixel::{Depth32F, RGBA32F},
  texture::{Dim2, Sampler},
  Backend,
};

pub struct LocalExample;

impl Example for LocalExample {
  fn bootstrap(
    _: &mut impl PlatformServices,
    context: &mut impl GraphicsContext<Backend = Backend>,
  ) -> Self {
    let framebuffer =
      context.new_framebuffer::<Dim2, RGBA32F, Depth32F>([1024, 1024], 0, Sampler::default());

    std::mem::drop(framebuffer);

    // #360 occurs here after the drop
    let _ = context.new_framebuffer::<Dim2, RGBA32F, Depth32F>([1024, 1024], 0, Sampler::default());

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
