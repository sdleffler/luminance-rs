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
//! The *color buffers* hold the color images you render to. A framebuffer can hold several of them
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

use std::marker::PhantomData;

use chain::Chain;
use pixel::{ColorPixel, DepthPixel, PixelFormat, RenderablePixel};
use texture::{Dim2, Dimensionable, Flat, HasTexture, Layerable, Texture, TextureError};

/// Trait to implement to provide framebuffer features.
///
/// When creating a new framebuffer with `new_framebuffer`, the color and depth formats are passed
/// and should be used to create internal textures and/or buffers to represent the slots.
pub trait HasFramebuffer: HasTexture + Sized {
  /// Framebuffer representation.
  type Framebuffer;

  /// Create a new framebuffer.
  fn new_framebuffer<L, D, CS, DS>(size: D::Size, mipmaps: usize) -> Result<(Self::Framebuffer, Vec<Self::ATexture>, Option<Self::ATexture>)>
    where L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          CS: ColorSlot<Self, L, D>,
          DS: DepthSlot<Self, L, D>;
  /// Free a framebuffer.
  fn free_framebuffer(framebuffer: &mut Self::Framebuffer);
  /// Default framebuffer.
  fn default_framebuffer<D>(size: D::Size) -> Self::Framebuffer
    where D: Dimensionable,
          D::Size: Copy;
}

/// Framebuffer error.
///
/// `Incomplete(reason)` occurs at framebuffer creation and `reason` gives a `String` explaination
/// of the failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FramebufferError {
  TextureError(TextureError),
  Incomplete(String)
}

pub type Result<T> = ::std::result::Result<T, FramebufferError>;

/// Framebuffer with static layering, dimension, access and slots formats.
///
/// A `Framebuffer` is a *GPU* special object used to render to. Because framebuffers have a
/// *layering* property, it’s possible to have regular render and *layered rendering*. The dimension
/// of a framebuffer makes it possible to render to 1D, 2D, 3D and cubemaps.
///
/// A framebuffer has two kind of slots:
///
/// - **color slot** ;
/// - **depth slot**.
///
/// A framebuffer can have zero or several color slots and it can have zero or one depth slot. If
/// you use several color slots, you’ll be performing what’s called *MRT* (*M* ultiple *R* ender
/// *T* argets), enabling to render to several textures at once.
#[derive(Debug)]
pub struct Framebuffer<C, L, D, CS, DS>
    where C: HasTexture + HasFramebuffer,
          L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          CS: ColorSlot<C, L, D>,
          DS: DepthSlot<C, L, D> {
  pub repr: C::Framebuffer,
  pub color_slot: CS,
  pub depth_slot: DS,
  _l: PhantomData<L>,
  _d: PhantomData<D>,
}

impl<C> Framebuffer<C, Flat, Dim2, (), ()> where C: HasTexture + HasFramebuffer {
  pub fn default(size: <Dim2 as Dimensionable>::Size) -> Self {
    Framebuffer {
      repr: C::default_framebuffer::<Dim2>(size),
      color_slot: (),
      depth_slot: (),
      _l: PhantomData,
      _d: PhantomData,
    }
  }
}

impl<C, L, D, CS, DS> Drop for Framebuffer<C, L, D, CS, DS>
    where C: HasTexture + HasFramebuffer,
          L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          CS: ColorSlot<C, L, D>,
          DS: DepthSlot<C, L, D> {
  fn drop(&mut self) {
    C::free_framebuffer(&mut self.repr)
  }
}

impl<C, L, D, CS, DS> Framebuffer<C, L, D, CS, DS>
    where C: HasTexture + HasFramebuffer,
          L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          CS: ColorSlot<C, L, D>,
          DS: DepthSlot<C, L, D> {
  pub fn new(size: D::Size, mipmaps: usize) -> Result<Framebuffer<C, L, D, CS, DS>> {
    let mipmaps = mipmaps + 1;

    C::new_framebuffer::<L, D, CS, DS>(size, mipmaps).map(|(framebuffer, mut color_textures, depth_texture)| {
      Framebuffer {
        repr: framebuffer,
        color_slot: CS::reify_textures(size, mipmaps, &mut color_textures),
        depth_slot: DS::reify_texture(size, mipmaps, depth_texture),
        _l: PhantomData,
        _d: PhantomData,
      }
    })
  }
}

/// A framebuffer has a color slot. A color slot can either be empty (the *unit* type is used,`()`)
/// or several color formats.
pub trait ColorSlot<C, L, D> where C: HasFramebuffer + HasTexture, L: Layerable, D: Dimensionable, D::Size: Copy {
  /// Turn a color slot into a list of pixel formats.
  fn color_formats() -> Vec<PixelFormat>;
  /// Reify a list of raw textures into a color slot.
  fn reify_textures(size: D::Size, mipmaps: usize, textures: &mut Vec<C::ATexture>) -> Self;
}

impl<C, L, D> ColorSlot<C, L, D> for () where C: HasFramebuffer + HasTexture, L: Layerable, D: Dimensionable, D::Size: Copy {
  fn color_formats() -> Vec<PixelFormat> { Vec::new() }

  fn reify_textures(_: D::Size, _: usize, _: &mut Vec<C::ATexture>) -> Self { () }
}

impl<C, L, D, P> ColorSlot<C, L, D> for Texture<C, L, D, P>
    where C: HasFramebuffer + HasTexture,
          L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P: ColorPixel + RenderablePixel {
  fn color_formats() -> Vec<PixelFormat> { vec![P::pixel_format()] }

  fn reify_textures(size: D::Size, mipmaps: usize, textures: &mut Vec<C::ATexture>) -> Self {
    let color_texture = textures.swap_remove(0);

    Texture::from_raw(color_texture, size, mipmaps)
  }
}

impl<C, L, D, P, B> ColorSlot<C, L, D> for Chain<Texture<C, L, D, P>, B>
    where C: HasFramebuffer + HasTexture,
          L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P: ColorPixel + RenderablePixel,
          B: ColorSlot<C, L, D> {
  fn color_formats() -> Vec<PixelFormat> {
    let mut a = Texture::<C, L, D, P>::color_formats();
    a.extend(B::color_formats());
    a
  }

  fn reify_textures(size: D::Size, mipmaps: usize, textures: &mut Vec<C::ATexture>) -> Self {
    let a = Texture::<C, L, D, P>::reify_textures(size, mipmaps, textures);
    let b = B::reify_textures(size, mipmaps, textures);

    Chain(a, b)
  }
}

impl<C, L, D, P0, P1> ColorSlot<C, L, D> for (Texture<C, L, D, P0>, Texture<C, L, D, P1>)
    where C: HasFramebuffer + HasTexture,
          L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P0: ColorPixel + RenderablePixel,
          P1: ColorPixel + RenderablePixel {
  fn color_formats() -> Vec<PixelFormat> {
    Chain::<Texture<C, L, D, P0>, Texture<C, L, D, P1>>::color_formats()
  }

  fn reify_textures(size: D::Size, mipmaps: usize, textures: &mut Vec<C::ATexture>) -> Self {
    let Chain(a, b) = Chain::<Texture<C, L, D, P0>, Texture<C, L, D, P1>>::reify_textures(size, mipmaps, textures);
    (a, b)
  }
}

impl<C, L, D, P0, P1, P2> ColorSlot<C, L, D> for (Texture<C, L, D, P0>, Texture<C, L, D, P1>, Texture<C, L, D, P2>)
    where C: HasFramebuffer + HasTexture,
          L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P0: ColorPixel + RenderablePixel,
          P1: ColorPixel + RenderablePixel,
          P2: ColorPixel + RenderablePixel {
  fn color_formats() -> Vec<PixelFormat> {
    Chain::<Texture<C, L, D, P0>, Chain<Texture<C, L, D, P1>, Texture<C, L, D, P2>>>::color_formats()
  }

  fn reify_textures(size: D::Size, mipmaps: usize, textures: &mut Vec<C::ATexture>) -> Self {
    let Chain(a, Chain(b, c)) = Chain::<Texture<C, L, D, P0>, Chain<Texture<C, L, D, P1>, Texture<C, L, D, P2>>>::reify_textures(size, mipmaps, textures);
    (a, b, c)
  }
}

impl<C, L, D, P0, P1, P2, P3> ColorSlot<C, L, D> for (Texture<C, L, D, P0>, Texture<C, L, D, P1>, Texture<C, L, D, P2>, Texture<C, L, D, P3>)
    where C: HasFramebuffer + HasTexture,
          L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P0: ColorPixel + RenderablePixel,
          P1: ColorPixel + RenderablePixel,
          P2: ColorPixel + RenderablePixel,
          P3: ColorPixel + RenderablePixel {
  fn color_formats() -> Vec<PixelFormat> {
    Chain::<Texture<C, L, D, P0>, Chain<Texture<C, L, D, P1>, Chain<Texture<C, L, D, P2>, Texture<C, L, D, P3>>>>::color_formats()
  }

  fn reify_textures(size: D::Size, mipmaps: usize, textures: &mut Vec<C::ATexture>) -> Self {
    let Chain(a, Chain(b, Chain(c, d))) = Chain::<Texture<C, L, D, P0>, Chain<Texture<C, L, D, P1>, Chain<Texture<C, L, D, P2>, Texture<C, L, D, P3>>>>::reify_textures(size, mipmaps, textures);
    (a, b, c, d)
  }
}

impl<C, L, D, P0, P1, P2, P3, P4> ColorSlot<C, L, D> for (Texture<C, L, D, P0>, Texture<C, L, D, P1>, Texture<C, L, D, P2>, Texture<C, L, D, P3>, Texture<C, L, D, P4>)
    where C: HasFramebuffer + HasTexture,
          L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P0: ColorPixel + RenderablePixel,
          P1: ColorPixel + RenderablePixel,
          P2: ColorPixel + RenderablePixel,
          P3: ColorPixel + RenderablePixel,
          P4: ColorPixel + RenderablePixel {
  fn color_formats() -> Vec<PixelFormat> {
    Chain::<Texture<C, L, D, P0>, Chain<Texture<C, L, D, P1>, Chain<Texture<C, L, D, P2>, Chain<Texture<C, L, D, P3>, Texture<C, L, D, P4>>>>>::color_formats()
  }

  fn reify_textures(size: D::Size, mipmaps: usize, textures: &mut Vec<C::ATexture>) -> Self {
    let Chain(a, Chain(b, Chain(c, Chain(d, e)))) = Chain::<Texture<C, L, D, P0>, Chain<Texture<C, L, D, P1>, Chain<Texture<C, L, D, P2>, Chain<Texture<C, L, D, P3>, Texture<C, L, D, P4>>>>>::reify_textures(size, mipmaps, textures);
    (a, b, c, d, e)
  }
}

impl<C, L, D, P0, P1, P2, P3, P4, P5> ColorSlot<C, L, D> for (Texture<C, L, D, P0>, Texture<C, L, D, P1>, Texture<C, L, D, P2>, Texture<C, L, D, P3>, Texture<C, L, D, P4>, Texture<C, L, D, P5>)
    where C: HasFramebuffer + HasTexture,
          L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P0: ColorPixel + RenderablePixel,
          P1: ColorPixel + RenderablePixel,
          P2: ColorPixel + RenderablePixel,
          P3: ColorPixel + RenderablePixel,
          P4: ColorPixel + RenderablePixel,
          P5: ColorPixel + RenderablePixel {
  fn color_formats() -> Vec<PixelFormat> {
    Chain::<Texture<C, L, D, P0>, Chain<Texture<C, L, D, P1>, Chain<Texture<C, L, D, P2>, Chain<Texture<C, L, D, P3>, Chain<Texture<C, L, D, P4>, Texture<C, L, D, P5>>>>>>::color_formats()
  }

  fn reify_textures(size: D::Size, mipmaps: usize, textures: &mut Vec<C::ATexture>) -> Self {
    let Chain(a, Chain(b, Chain(c, Chain(d, Chain(e, f))))) = Chain::<Texture<C, L, D, P0>, Chain<Texture<C, L, D, P1>, Chain<Texture<C, L, D, P2>, Chain<Texture<C, L, D, P3>, Chain<Texture<C, L, D, P4>, Texture<C, L, D, P5>>>>>>::reify_textures(size, mipmaps, textures);
    (a, b, c, d, e, f)
  }
}

impl<C, L, D, P0, P1, P2, P3, P4, P5, P6> ColorSlot<C, L, D> for (Texture<C, L, D, P0>, Texture<C, L, D, P1>, Texture<C, L, D, P2>, Texture<C, L, D, P3>, Texture<C, L, D, P4>, Texture<C, L, D, P5>, Texture<C, L, D, P6>)
    where C: HasFramebuffer + HasTexture,
          L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P0: ColorPixel + RenderablePixel,
          P1: ColorPixel + RenderablePixel,
          P2: ColorPixel + RenderablePixel,
          P3: ColorPixel + RenderablePixel,
          P4: ColorPixel + RenderablePixel,
          P5: ColorPixel + RenderablePixel,
          P6: ColorPixel + RenderablePixel {
  fn color_formats() -> Vec<PixelFormat> {
    Chain::<Texture<C, L, D, P0>, Chain<Texture<C, L, D, P1>, Chain<Texture<C, L, D, P2>, Chain<Texture<C, L, D, P3>, Chain<Texture<C, L, D, P4>, Chain<Texture<C, L, D, P5>, Texture<C, L, D, P6>>>>>>>::color_formats()
  }

  fn reify_textures(size: D::Size, mipmaps: usize, textures: &mut Vec<C::ATexture>) -> Self {
    let Chain(a, Chain(b, Chain(c, Chain(d, Chain(e, Chain(f, g)))))) = Chain::<Texture<C, L, D, P0>, Chain<Texture<C, L, D, P1>, Chain<Texture<C, L, D, P2>, Chain<Texture<C, L, D, P3>, Chain<Texture<C, L, D, P4>, Chain<Texture<C, L, D, P5>, Texture<C, L, D, P6>>>>>>>::reify_textures(size, mipmaps, textures);
    (a, b, c, d, e, f, g)
  }
}

impl<C, L, D, P0, P1, P2, P3, P4, P5, P6, P7> ColorSlot<C, L, D> for (Texture<C, L, D, P0>, Texture<C, L, D, P1>, Texture<C, L, D, P2>, Texture<C, L, D, P3>, Texture<C, L, D, P4>, Texture<C, L, D, P5>, Texture<C, L, D, P6>, Texture<C, L, D, P7>)
    where C: HasFramebuffer + HasTexture,
          L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P0: ColorPixel + RenderablePixel,
          P1: ColorPixel + RenderablePixel,
          P2: ColorPixel + RenderablePixel,
          P3: ColorPixel + RenderablePixel,
          P4: ColorPixel + RenderablePixel,
          P5: ColorPixel + RenderablePixel,
          P6: ColorPixel + RenderablePixel,
          P7: ColorPixel + RenderablePixel {
  fn color_formats() -> Vec<PixelFormat> {
    Chain::<Texture<C, L, D, P0>, Chain<Texture<C, L, D, P1>, Chain<Texture<C, L, D, P2>, Chain<Texture<C, L, D, P3>, Chain<Texture<C, L, D, P4>, Chain<Texture<C, L, D, P5>, Chain<Texture<C, L, D, P6>, Texture<C, L, D, P7>>>>>>>>::color_formats()
  }

  fn reify_textures(size: D::Size, mipmaps: usize, textures: &mut Vec<C::ATexture>) -> Self {
    let Chain(a, Chain(b, Chain(c, Chain(d, Chain(e, Chain(f, Chain(g, h))))))) = Chain::<Texture<C, L, D, P0>, Chain<Texture<C, L, D, P1>, Chain<Texture<C, L, D, P2>, Chain<Texture<C, L, D, P3>, Chain<Texture<C, L, D, P4>, Chain<Texture<C, L, D, P5>, Chain<Texture<C, L, D, P6>, Texture<C, L, D, P7>>>>>>>>::reify_textures(size, mipmaps, textures);
    (a, b, c, d, e, f, g, h)
  }
}

impl<C, L, D, P0, P1, P2, P3, P4, P5, P6, P7, P8> ColorSlot<C, L, D> for (Texture<C, L, D, P0>, Texture<C, L, D, P1>, Texture<C, L, D, P2>, Texture<C, L, D, P3>, Texture<C, L, D, P4>, Texture<C, L, D, P5>, Texture<C, L, D, P6>, Texture<C, L, D, P7>, Texture<C, L, D, P8>)
    where C: HasFramebuffer + HasTexture,
          L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P0: ColorPixel + RenderablePixel,
          P1: ColorPixel + RenderablePixel,
          P2: ColorPixel + RenderablePixel,
          P3: ColorPixel + RenderablePixel,
          P4: ColorPixel + RenderablePixel,
          P5: ColorPixel + RenderablePixel,
          P6: ColorPixel + RenderablePixel,
          P7: ColorPixel + RenderablePixel,
          P8: ColorPixel + RenderablePixel {
  fn color_formats() -> Vec<PixelFormat> {
    Chain::<Texture<C, L, D, P0>, Chain<Texture<C, L, D, P1>, Chain<Texture<C, L, D, P2>, Chain<Texture<C, L, D, P3>, Chain<Texture<C, L, D, P4>, Chain<Texture<C, L, D, P5>, Chain<Texture<C, L, D, P6>, Chain<Texture<C, L, D, P7>, Texture<C, L, D, P8>>>>>>>>>::color_formats()
  }

  fn reify_textures(size: D::Size, mipmaps: usize, textures: &mut Vec<C::ATexture>) -> Self {
    let Chain(a, Chain(b, Chain(c, Chain(d, Chain(e, Chain(f, Chain(g, Chain(h, i)))))))) = Chain::<Texture<C, L, D, P0>, Chain<Texture<C, L, D, P1>, Chain<Texture<C, L, D, P2>, Chain<Texture<C, L, D, P3>, Chain<Texture<C, L, D, P4>, Chain<Texture<C, L, D, P5>, Chain<Texture<C, L, D, P6>, Chain<Texture<C, L, D, P7>, Texture<C, L, D, P8>>>>>>>>>::reify_textures(size, mipmaps, textures);
    (a, b, c, d, e, f, g, h, i)
  }
}

impl<C, L, D, P0, P1, P2, P3, P4, P5, P6, P7, P8, P9> ColorSlot<C, L, D> for (Texture<C, L, D, P0>, Texture<C, L, D, P1>, Texture<C, L, D, P2>, Texture<C, L, D, P3>, Texture<C, L, D, P4>, Texture<C, L, D, P5>, Texture<C, L, D, P6>, Texture<C, L, D, P7>, Texture<C, L, D, P8>, Texture<C, L, D, P9>)
    where C: HasFramebuffer + HasTexture,
          L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P0: ColorPixel + RenderablePixel,
          P1: ColorPixel + RenderablePixel,
          P2: ColorPixel + RenderablePixel,
          P3: ColorPixel + RenderablePixel,
          P4: ColorPixel + RenderablePixel,
          P5: ColorPixel + RenderablePixel,
          P6: ColorPixel + RenderablePixel,
          P7: ColorPixel + RenderablePixel,
          P8: ColorPixel + RenderablePixel,
          P9: ColorPixel + RenderablePixel {
  fn color_formats() -> Vec<PixelFormat> {
    Chain::<Texture<C, L, D, P0>, Chain<Texture<C, L, D, P1>, Chain<Texture<C, L, D, P2>, Chain<Texture<C, L, D, P3>, Chain<Texture<C, L, D, P4>, Chain<Texture<C, L, D, P5>, Chain<Texture<C, L, D, P6>, Chain<Texture<C, L, D, P7>, Chain<Texture<C, L, D, P8>, Texture<C, L, D, P9>>>>>>>>>>::color_formats()
  }

  fn reify_textures(size: D::Size, mipmaps: usize, textures: &mut Vec<C::ATexture>) -> Self {
    let Chain(a, Chain(b, Chain(c, Chain(d, Chain(e, Chain(f, Chain(g, Chain(h, Chain(i, j))))))))) = Chain::<Texture<C, L, D, P0>, Chain<Texture<C, L, D, P1>, Chain<Texture<C, L, D, P2>, Chain<Texture<C, L, D, P3>, Chain<Texture<C, L, D, P4>, Chain<Texture<C, L, D, P5>, Chain<Texture<C, L, D, P6>, Chain<Texture<C, L, D, P7>, Chain<Texture<C, L, D, P8>, Texture<C, L, D, P9>>>>>>>>>>::reify_textures(size, mipmaps, textures);
    (a, b, c, d, e, f, g, h, i, j)
  }
}

/// A framebuffer has a depth slot. A depth slot can either be empty (the *unit* type is used, `()`)
/// or a single depth format.
pub trait DepthSlot<C, L, D> where C: HasFramebuffer + HasTexture, L: Layerable, D: Dimensionable, D::Size: Copy {
  /// Turn a depth slot into a pixel format.
  fn depth_format() -> Option<PixelFormat>;
  /// Reify a raw textures into a depth slot.
  fn reify_texture(size: D::Size, mipmaps: usize, texture: Option<C::ATexture>) -> Self; }

impl<C, L, D> DepthSlot<C, L, D> for () where C: HasFramebuffer + HasTexture, L: Layerable, D: Dimensionable, D::Size: Copy {
  fn depth_format() -> Option<PixelFormat> { None }

  fn reify_texture(_: D::Size, _: usize, _: Option<C::ATexture>) -> Self { () }
}

impl<C, L, D, P> DepthSlot<C, L, D> for Texture<C, L, D, P>
    where C: HasFramebuffer + HasTexture,
          L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P: DepthPixel {
  fn depth_format() -> Option<PixelFormat> { Some(P::pixel_format()) }

  fn reify_texture(size: D::Size, mipmaps: usize, texture: Option<C::ATexture>) -> Self {
    Texture::from_raw(texture.unwrap(), size, mipmaps)
  }
}
