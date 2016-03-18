use core::marker::PhantomData;
use pixel::Pixel;
use texture::*;

pub trait HasFramebuffer {
  type AFramebuffer;

  fn new<'a, D>(size: D::Size, mipmaps: u32) -> Result<Self::AFramebuffer, FramebufferError<'a>> where D: Dimensionable;
}

pub enum FramebufferError<'a> {
  Incomplete(&'a str)
}

/// A framebuffer with `L` being the layering, `D` the dimension, `A` the access, `Color` the color
/// format and `Depth` the depth format.
pub struct Framebuffer<C, L, D, A, Color, Depth>
    where C: HasTexture,
          L: Layerable,
          D: Dimensionable,
          Color: ColorPixel,
          Depth: DepthPixel {
  color_tex: Option<Tex<C, L, D, Color>>,
  depth_tex: Option<Tex<C, L, D, Depth>>,
  _c: PhantomData<C>,
  _a: PhantomData<A>,
}
