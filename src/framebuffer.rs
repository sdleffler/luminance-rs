//! Framebuffers and utility types and functions.
//!
//! Framebuffers are at the core of rendering. They’re the support of rendering operation and can
//! be used to highly enhance the visual aspect of a render. You’re always provided with at least
//! one framebuffer, `default_framebuffer()`. That function returns a framebuffer that represents –
//! for short – your screen’s back framebuffer. You can render to that framebuffer and when you
//! *swap* the window’s buffers, your render appears at the screen.
//!
//! # Framebuffers
//!
//! A framebuffer is an object maintaining the required GPU state to hold images your render to.
//! It gathers two important concepts:
//!
//! - *color buffers*;
//! - *depth buffers*.
//!
//! The *color buffers* hold the color images you render to. A framebuffers can hold several of them
//! with different color formats. The *depth buffers* hold the depth images you render to.
//! Framebuffers can hold only one depth buffer.
//!
//! # Framebuffer slots
//!
//! A framebuffer slot contains either its color buffers or its depth buffer. Sometimes, you might
//! find it handy to have no slot at all for a given type of buffer. In that case, we use `()`.
//!
//! The slots are a way to convert the different formats you use for your framebuffers’ buffers into
//! their respective texture representation so that you can handle the corresponding texels.
//!
//! Color buffers are abstracted by `ColorSlot` and the depth buffer by `DepthSlot`.

use chain::Chain;
use core::marker::PhantomData;
use rw::RW;
use pixel::{ColorPixel, DepthPixel, Pixel, PixelFormat};
use std::vec::Vec;
use texture::{Dim2, Dimensionable, Flat, HasTexture, Layerable, Texture};

pub trait HasFramebuffer {
  type Framebuffer;

  /// Create a new framebuffer.
  fn new_framebuffer<D>(size: D::Size, mipmaps: u32, color_formats: &Vec<PixelFormat>, depth_format: Option<PixelFormat>) -> Result<Self::Framebuffer, FramebufferError> where D: Dimensionable;
  /// Default framebuffer.
  fn default_framebuffer() -> Self::Framebuffer;
}

pub enum FramebufferError {
  Incomplete(String)
}

pub struct Framebuffer<C, L, D, A, CS, DS>
    where C: HasTexture + HasFramebuffer,
          L: Layerable,
          D: Dimensionable,
          CS: ColorSlot,
          DS: DepthSlot {
  pub repr: C::Framebuffer,
  pub color_slot: PhantomData<CS>,
  pub depth_slot: PhantomData<DS>,
  _l: PhantomData<L>,
  _d: PhantomData<D>,
  _a: PhantomData<A>
}

/*
impl<C, L, D, A, CS, DS> Framebuffer<C, L, D, A, CS, DS>
    where C: HasTexture + HasFramebuffer,
          L: Layerable,
          D: Dimensionable,
          CS: ColorSlot,
          DS: DepthSlot {
  fn new(size: D::Size, mipmaps: u32) -> Result<Framebuffer<C, L, D, A, CS, DS>, FramebufferError> {
    C::new_framebuffer(size, mipmaps, &CS::color_formats(), DS::depth_format()).map(|framebuffer| Framebuffer {
      repr: framebuffer,
      color_slot: PhantomData,
      depth_slot: PhantomData,
      _l: PhantomData,
      _d: PhantomData,
      _a: PhantomData
    })
  }
}
*/

/*
pub fn default_framebuffer<C>() -> Framebuffer<C, Flat, Dim2, RW, (), ()> where C: HasTexture + HasFramebuffer {
  Framebuffer {
    repr: C::default_framebuffer(),
    color_slot: (),
    depth_slot: (),
    _a: PhantomData
  }
}
*/

/// Slot type; used to create color and depth slots for framebuffers.
pub struct Slot<C, L, D, P>
    where C: HasTexture,
          L: Layerable,
          D: Dimensionable,
          P: Pixel {
  pub texture_slot: Texture<C, L, D, P>
}

/// A framebuffer has a color slot. A color slot can either be empty (the *unit* type is used,`()`)
/// or several color formats.
pub trait ColorSlot {
  fn color_formats() -> Vec<PixelFormat>;
}

impl ColorSlot for () {
  fn color_formats() -> Vec<PixelFormat> { Vec::new() }
}

impl<C, L, D, P> ColorSlot for Slot<C, L, D, P>
    where C: HasTexture,
          L: Layerable,
          D: Dimensionable,
          P: ColorPixel {
  fn color_formats() -> Vec<PixelFormat> { vec![P::pixel_format()] }
}

impl<A, B> ColorSlot for Chain<A, B> where A: ColorSlot, B: ColorSlot {
  fn color_formats() -> Vec<PixelFormat> {
    let mut a = A::color_formats();
    a.extend(B::color_formats());
    a
  }
}

/// A framebuffer has a depth slot. A depth slot can either be empty (the *unit* type is used, `()`)
/// or a single depth format.
pub trait DepthSlot {
  fn depth_format() -> Option<PixelFormat>;
}

impl DepthSlot for () {
  fn depth_format() -> Option<PixelFormat> { None }
}

impl<C, L, D, P> DepthSlot for Slot<C, L, D, P>
    where C: HasTexture,
          L: Layerable,
          D: Dimensionable,
          P: DepthPixel {
  fn depth_format() -> Option<PixelFormat> { Some(P::pixel_format()) }
}
