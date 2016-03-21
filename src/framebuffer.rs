use chain::Chain;
use core::marker::PhantomData;
use pixel::{ColorPixel, DepthPixel, Pixel, PixelFormat};
use std::vec::Vec;
use texture::{Dimensionable, HasTexture, Layerable, Tex};

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

pub struct Framebuffer<C, A, CS, DS>
    where C: HasTexture + HasFramebuffer,
          CS: ColorSlot,
          DS: DepthSlot {
  pub repr: C::AFramebuffer,
  pub color_slot: CS,
  pub depth_slot: DS,
  _a: PhantomData<A>
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
pub struct Slot<C, L, D, P>
    where C: HasTexture,
          L: Layerable,
          D: Dimensionable,
          P: Pixel {
  pub texture_slot: Tex<C, L, D, P>
}

/// A framebuffer has a color slot. A color slot can either be empty (the *unit* type is used,`()`)
/// or several color formats.
pub trait ColorSlot {
  fn color_slots() -> Vec<PixelFormat>;
}

impl ColorSlot for () {
  fn color_slots() -> Vec<PixelFormat> { Vec::new() }
}

impl<C, L, D, P> ColorSlot for Slot<C, L, D, P>
    where C: HasTexture,
          L: Layerable,
          D: Dimensionable,
          P: ColorPixel {
  fn color_slots() -> Vec<PixelFormat> { vec![P::pixel_format()] }
}

impl<A, B> ColorSlot for Chain<A,B> where A: ColorSlot, B: ColorSlot {
  fn color_slots() -> Vec<PixelFormat> {
    let mut a = A::color_slots();
    a.extend(B::color_slots());
    a
  }
}

/// A framebuffer has a depth slot. A depth slot can either be empty (the *unit* type is used, `()`)
/// or a single depth format.
pub trait DepthSlot {
  fn depth_slot() -> Vec<PixelFormat>;
}

impl DepthSlot for () {
  fn depth_slot() -> Vec<PixelFormat> { Vec::new() }
}

impl<C, L, D, P> DepthSlot for Slot<C, L, D, P>
    where C: HasTexture,
          L: Layerable,
          D: Dimensionable,
          P: DepthPixel {
  fn depth_slot() -> Vec<PixelFormat> { vec![P::pixel_format()] }
}
