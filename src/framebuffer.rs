use chain::Chain;
use core::marker::PhantomData;
use pixel::{ColorPixel, DepthPixel, PixelFormat};
use std::vec::Vec;
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

/// Slot type; used to create color and depth slots for framebuffers.
pub struct Slot<T> {
  _t: PhantomData<T>
}

/// A framebuffer has a color slot. A color slot can either be empty (the *unit* type is used,`()`)
/// or several color formats. You can have up
pub trait ColorSlot {
  fn color_slots() -> Vec<PixelFormat>;
}

impl ColorSlot for () {
  fn color_slots() -> Vec<PixelFormat> { Vec::new() }
}

impl<P> ColorSlot for Slot<P> where P: ColorPixel {
  fn color_slots() -> Vec<PixelFormat> { vec![P::pixel_format()] }
}

impl<A, B> ColorSlot for Chain<A,B> where A: ColorSlot, B: ColorSlot {
  fn color_slots() -> Vec<PixelFormat> {
    let mut a = A::color_slots();
    a.extend(B::color_slots());
    a
  }
}
