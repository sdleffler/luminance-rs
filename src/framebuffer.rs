use core::marker::PhantomData;
use pixel::{ColorPixel, DepthPixel};
use texture::*;

pub trait HasFramebuffer {
  type AFramebuffer;

  /// Create a new framebuffer.
  fn new<'a, D>(size: D::Size, mipmaps: u32) -> Result<Self::AFramebuffer, FramebufferError<'a>> where D: Dimensionable;
  /// Default framebuffer.
  fn default_framebuffer() -> Self::AFramebuffer;
}

pub enum FramebufferError<'a> {
  Incomplete(&'a str)
}

pub struct Framebuffer<C, L, D, A, Color, Depth>
    where C: HasTexture + HasFramebuffer,
          L: Layerable,
          D: Dimensionable,
          Color: ColorPixel,
          Depth: DepthPixel {
  pub repr: C::AFramebuffer,
  pub color_tex: Option<Tex<C, L, D, Color>>,
  pub depth_tex: Option<Tex<C, L, D, Depth>>,
  _a: PhantomData<A>,
}

/*
impl<C, L, D, A, Color, Depth> Framebuffer<C, L, D, A, Color, Depth>
    where C: HasTexture + HasFramebuffer,
          L: Layerable,
          D: Dimensionable,
          Color: ColorPixel,
          Depth: DepthPixel {
  fn new<'a>(size: D::Size, mipmaps: u32) -> Result<Framebuffer<C, L, D, A, Color, Depth>, FramebufferError<'a>> {
  }
}
*/

/// A framebuffer has a color slot. A color slot can either be empty (the *unit* type is used,`()`)
/// or several color formats. You can have up
pub trait ColorSlot: ColorPixel {}

