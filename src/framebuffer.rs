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

use chain::Chain;
use core::marker::PhantomData;
use pixel::{ColorPixel, DepthPixel, Pixel, PixelFormat};
use std::default::Default;
use texture::{Dim2, Dimensionable, Flat, HasTexture, Layerable, Texture};

/// Trait to implement to provide framebuffer features.
///
/// When creating a new framebuffer with `new_framebuffer`, the color and depth formats are passed
/// and should be used to create internal textures and/or buffers to represent the slots.
pub trait HasFramebuffer: HasTexture {
  /// Framebuffer representation.
  type Framebuffer;

  /// Create a new framebuffer.
  fn new_framebuffer() -> Result<Self::Framebuffer, FramebufferError>;
  /// Free a framebuffer.
  fn free_framebuffer(framebuffer: &mut Self::Framebuffer);
  /// Default framebuffer.
  fn default_framebuffer() -> Self::Framebuffer;
  /// Called when a color slot is created. The `index` parameter gives the stream index of the color
  /// slot.
  fn accept_color_slot<D>(framebuffer: &mut Self::Framebuffer, size: D::Size, color_texture: &Self::ATexture, index: u8) where D: Dimensionable, D::Size: Copy;
  /// Called when a depth slot is created.
  fn accept_depth_slot<D>(framebuffer: &mut Self::Framebuffer, size: D::Size, depth_texture: &Self::ATexture) where D: Dimensionable, D::Size: Copy;
  /// Called when no color slot is required.
  fn disable_color_slot(framebuffer: &mut Self::Framebuffer);
  /// Called when no depth slot is required.
  fn disable_depth_slot(framebuffer: &mut Self::Framebuffer);
}

/// Framebuffer error.
///
/// `Incomplete(reason)` occurs at framebuffer creation and `reason` gives a `String` explaination
/// of the failure.
pub enum FramebufferError {
  Incomplete(String)
}

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
  pub fn new(size: D::Size, mipmaps: u32) -> Result<Framebuffer<C, L, D, CS, DS>, FramebufferError> {
    C::new_framebuffer().map(|mut framebuffer| {
      let color_slot = CS::new_color_slot(&mut framebuffer, size, mipmaps, 0);
      let depth_slot = DS::new_depth_slot(&mut framebuffer, size, mipmaps);

      Framebuffer {
        repr: framebuffer,
        color_slot: color_slot,
        depth_slot: depth_slot,
        _l: PhantomData,
        _d: PhantomData,
      }
    })
  }
}

pub fn default_framebuffer<C>() -> Framebuffer<C, Flat, Dim2, (), ()> where C: HasTexture + HasFramebuffer {
  Framebuffer {
    repr: C::default_framebuffer(),
    color_slot: (),
    depth_slot: (),
    _l: PhantomData,
    _d: PhantomData,
  }
}

/// Slot type; used to create color and depth slots for framebuffers.
pub struct Slot<C, L, D, P>
    where C: HasTexture,
          L: Layerable,
          D: Dimensionable,
          P: Pixel {
  pub texture: Texture<C, L, D, P>
}

/// A framebuffer has a color slot. A color slot can either be empty (the *unit* type is used,`()`)
/// or several color formats.
pub trait ColorSlot<C, L, D> where C: HasFramebuffer + HasTexture, L: Layerable, D: Dimensionable, D::Size: Copy {
  fn new_color_slot(framebuffer: &mut C::Framebuffer, size: D::Size, mipmaps: u32, index: u8) -> Self;
}

impl<C, L, D> ColorSlot<C, L, D> for () where C: HasFramebuffer + HasTexture, L: Layerable, D: Dimensionable, D::Size: Copy {
  fn new_color_slot(framebuffer: &mut C::Framebuffer, _: D::Size, _: u32, _: u8) -> Self {
    C::disable_color_slot(framebuffer)
  }
}

impl<C, L, D, P> ColorSlot<C, L, D> for Slot<C, L, D, P>
    where C: HasFramebuffer + HasTexture,
          L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P: ColorPixel {
  fn new_color_slot(framebuffer: &mut C::Framebuffer, size: D::Size, mipmaps: u32, index: u8) -> Self {
    let color_slot = create_slot(size, mipmaps);

    C::accept_color_slot::<D>(framebuffer, size, &color_slot.texture.repr, index);

    color_slot
  }
}

impl<C, L, D, P, B> ColorSlot<C, L, D> for Chain<Slot<C, L, D, P>, B>
    where C: HasFramebuffer + HasTexture,
          L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P: ColorPixel,
          B: ColorSlot<C, L, D> {
  fn new_color_slot(framebuffer: &mut C::Framebuffer, size: D::Size, mipmaps: u32, index: u8) -> Self {
    let a = Slot::<C, L, D, P>::new_color_slot(framebuffer, size, mipmaps, index);
    let b = B::new_color_slot(framebuffer, size, mipmaps, index + 1);
    Chain(a, b)
  }
}

impl<C, L, D, P0, P1> ColorSlot<C, L, D> for (Slot<C, L, D, P0>, Slot<C, L, D, P1>)
    where C: HasFramebuffer + HasTexture,
          L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P0: ColorPixel,
          P1: ColorPixel {
  fn new_color_slot(framebuffer: &mut C::Framebuffer, size: D::Size, mipmaps: u32, index: u8) -> Self {
    let Chain(a, b) = Chain::<Slot<C, L, D, P0>, Slot<C, L, D, P1>>::new_color_slot(framebuffer, size, mipmaps, index);
    (a, b)
  }
}

impl<C, L, D, P0, P1, P2> ColorSlot<C, L, D> for (Slot<C, L, D, P0>, Slot<C, L, D, P1>, Slot<C, L, D, P2>)
    where C: HasFramebuffer + HasTexture,
          L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P0: ColorPixel,
          P1: ColorPixel,
          P2: ColorPixel {
  fn new_color_slot(framebuffer: &mut C::Framebuffer, size: D::Size, mipmaps: u32, index: u8) -> Self {
    let Chain(a, Chain(b, c)) = Chain::<Slot<C, L, D, P0>, Chain<Slot<C, L, D, P1>, Slot<C, L, D, P2>>>::new_color_slot(framebuffer, size, mipmaps, index);
    (a, b, c)
  }
}

impl<C, L, D, P0, P1, P2, P3> ColorSlot<C, L, D> for (Slot<C, L, D, P0>, Slot<C, L, D, P1>, Slot<C, L, D, P2>, Slot<C, L, D, P3>)
    where C: HasFramebuffer + HasTexture,
          L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P0: ColorPixel,
          P1: ColorPixel,
          P2: ColorPixel,
          P3: ColorPixel {
  fn new_color_slot(framebuffer: &mut C::Framebuffer, size: D::Size, mipmaps: u32, index: u8) -> Self {
    let Chain(a, Chain(b, Chain(c, d))) = Chain::<Slot<C, L, D, P0>, Chain<Slot<C, L, D, P1>, Chain<Slot<C, L, D, P2>, Slot<C, L, D, P3>>>>::new_color_slot(framebuffer, size, mipmaps, index);
    (a, b, c, d)
  }
}

impl<C, L, D, P0, P1, P2, P3, P4> ColorSlot<C, L, D> for (Slot<C, L, D, P0>, Slot<C, L, D, P1>, Slot<C, L, D, P2>, Slot<C, L, D, P3>, Slot<C, L, D, P4>)
    where C: HasFramebuffer + HasTexture,
          L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P0: ColorPixel,
          P1: ColorPixel,
          P2: ColorPixel,
          P3: ColorPixel,
          P4: ColorPixel {
  fn new_color_slot(framebuffer: &mut C::Framebuffer, size: D::Size, mipmaps: u32, index: u8) -> Self {
    let Chain(a, Chain(b, Chain(c, Chain(d, e)))) = Chain::<Slot<C, L, D, P0>, Chain<Slot<C, L, D, P1>, Chain<Slot<C, L, D, P2>, Chain<Slot<C, L, D, P3>, Slot<C, L, D, P4>>>>>::new_color_slot(framebuffer, size, mipmaps, index);
    (a, b, c, d, e)
  }
}

impl<C, L, D, P0, P1, P2, P3, P4, P5> ColorSlot<C, L, D> for (Slot<C, L, D, P0>, Slot<C, L, D, P1>, Slot<C, L, D, P2>, Slot<C, L, D, P3>, Slot<C, L, D, P4>, Slot<C, L, D, P5>)
    where C: HasFramebuffer + HasTexture,
          L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P0: ColorPixel,
          P1: ColorPixel,
          P2: ColorPixel,
          P3: ColorPixel,
          P4: ColorPixel,
          P5: ColorPixel {
  fn new_color_slot(framebuffer: &mut C::Framebuffer, size: D::Size, mipmaps: u32, index: u8) -> Self {
    let Chain(a, Chain(b, Chain(c, Chain(d, Chain(e, f))))) = Chain::<Slot<C, L, D, P0>, Chain<Slot<C, L, D, P1>, Chain<Slot<C, L, D, P2>, Chain<Slot<C, L, D, P3>, Chain<Slot<C, L, D, P4>, Slot<C, L, D, P5>>>>>>::new_color_slot(framebuffer, size, mipmaps, index);
    (a, b, c, d, e, f)
  }
}

impl<C, L, D, P0, P1, P2, P3, P4, P5, P6> ColorSlot<C, L, D> for (Slot<C, L, D, P0>, Slot<C, L, D, P1>, Slot<C, L, D, P2>, Slot<C, L, D, P3>, Slot<C, L, D, P4>, Slot<C, L, D, P5>, Slot<C, L, D, P6>)
    where C: HasFramebuffer + HasTexture,
          L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P0: ColorPixel,
          P1: ColorPixel,
          P2: ColorPixel,
          P3: ColorPixel,
          P4: ColorPixel,
          P5: ColorPixel,
          P6: ColorPixel {
  fn new_color_slot(framebuffer: &mut C::Framebuffer, size: D::Size, mipmaps: u32, index: u8) -> Self {
    let Chain(a, Chain(b, Chain(c, Chain(d, Chain(e, Chain(f, g)))))) = Chain::<Slot<C, L, D, P0>, Chain<Slot<C, L, D, P1>, Chain<Slot<C, L, D, P2>, Chain<Slot<C, L, D, P3>, Chain<Slot<C, L, D, P4>, Chain<Slot<C, L, D, P5>, Slot<C, L, D, P6>>>>>>>::new_color_slot(framebuffer, size, mipmaps, index);
    (a, b, c, d, e, f, g)
  }
}

impl<C, L, D, P0, P1, P2, P3, P4, P5, P6, P7> ColorSlot<C, L, D> for (Slot<C, L, D, P0>, Slot<C, L, D, P1>, Slot<C, L, D, P2>, Slot<C, L, D, P3>, Slot<C, L, D, P4>, Slot<C, L, D, P5>, Slot<C, L, D, P6>, Slot<C, L, D, P7>)
    where C: HasFramebuffer + HasTexture,
          L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P0: ColorPixel,
          P1: ColorPixel,
          P2: ColorPixel,
          P3: ColorPixel,
          P4: ColorPixel,
          P5: ColorPixel,
          P6: ColorPixel,
          P7: ColorPixel {
  fn new_color_slot(framebuffer: &mut C::Framebuffer, size: D::Size, mipmaps: u32, index: u8) -> Self {
    let Chain(a, Chain(b, Chain(c, Chain(d, Chain(e, Chain(f, Chain(g, h))))))) = Chain::<Slot<C, L, D, P0>, Chain<Slot<C, L, D, P1>, Chain<Slot<C, L, D, P2>, Chain<Slot<C, L, D, P3>, Chain<Slot<C, L, D, P4>, Chain<Slot<C, L, D, P5>, Chain<Slot<C, L, D, P6>, Slot<C, L, D, P7>>>>>>>>::new_color_slot(framebuffer, size, mipmaps, index);
    (a, b, c, d, e, f, g, h)
  }
}

impl<C, L, D, P0, P1, P2, P3, P4, P5, P6, P7, P8> ColorSlot<C, L, D> for (Slot<C, L, D, P0>, Slot<C, L, D, P1>, Slot<C, L, D, P2>, Slot<C, L, D, P3>, Slot<C, L, D, P4>, Slot<C, L, D, P5>, Slot<C, L, D, P6>, Slot<C, L, D, P7>, Slot<C, L, D, P8>)
    where C: HasFramebuffer + HasTexture,
          L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P0: ColorPixel,
          P1: ColorPixel,
          P2: ColorPixel,
          P3: ColorPixel,
          P4: ColorPixel,
          P5: ColorPixel,
          P6: ColorPixel,
          P7: ColorPixel,
          P8: ColorPixel {
  fn new_color_slot(framebuffer: &mut C::Framebuffer, size: D::Size, mipmaps: u32, index: u8) -> Self {
    let Chain(a, Chain(b, Chain(c, Chain(d, Chain(e, Chain(f, Chain(g, Chain(h, i)))))))) = Chain::<Slot<C, L, D, P0>, Chain<Slot<C, L, D, P1>, Chain<Slot<C, L, D, P2>, Chain<Slot<C, L, D, P3>, Chain<Slot<C, L, D, P4>, Chain<Slot<C, L, D, P5>, Chain<Slot<C, L, D, P6>, Chain<Slot<C, L, D, P7>, Slot<C, L, D, P8>>>>>>>>>::new_color_slot(framebuffer, size, mipmaps, index);
    (a, b, c, d, e, f, g, h, i)
  }
}

impl<C, L, D, P0, P1, P2, P3, P4, P5, P6, P7, P8, P9> ColorSlot<C, L, D> for (Slot<C, L, D, P0>, Slot<C, L, D, P1>, Slot<C, L, D, P2>, Slot<C, L, D, P3>, Slot<C, L, D, P4>, Slot<C, L, D, P5>, Slot<C, L, D, P6>, Slot<C, L, D, P7>, Slot<C, L, D, P8>, Slot<C, L, D, P9>)
    where C: HasFramebuffer + HasTexture,
          L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P0: ColorPixel,
          P1: ColorPixel,
          P2: ColorPixel,
          P3: ColorPixel,
          P4: ColorPixel,
          P5: ColorPixel,
          P6: ColorPixel,
          P7: ColorPixel,
          P8: ColorPixel,
          P9: ColorPixel {
  fn new_color_slot(framebuffer: &mut C::Framebuffer, size: D::Size, mipmaps: u32, index: u8) -> Self {
    let Chain(a, Chain(b, Chain(c, Chain(d, Chain(e, Chain(f, Chain(g, Chain(h, Chain(i, j))))))))) = Chain::<Slot<C, L, D, P0>, Chain<Slot<C, L, D, P1>, Chain<Slot<C, L, D, P2>, Chain<Slot<C, L, D, P3>, Chain<Slot<C, L, D, P4>, Chain<Slot<C, L, D, P5>, Chain<Slot<C, L, D, P6>, Chain<Slot<C, L, D, P7>, Chain<Slot<C, L, D, P8>, Slot<C, L, D, P9>>>>>>>>>>::new_color_slot(framebuffer, size, mipmaps, index);
    (a, b, c, d, e, f, g, h, i, j)
  }
}

/// A framebuffer has a depth slot. A depth slot can either be empty (the *unit* type is used, `()`)
/// or a single depth format.
pub trait DepthSlot<C, L, D> where C: HasFramebuffer + HasTexture, L: Layerable, D: Dimensionable, D::Size: Copy {
  fn depth_format() -> Option<PixelFormat>;
  fn new_depth_slot(framebuffer: &mut C::Framebuffer, size: D::Size, mipmaps: u32) -> Self;
}

impl<C, L, D> DepthSlot<C, L, D> for () where C: HasFramebuffer + HasTexture, L: Layerable, D: Dimensionable, D::Size: Copy {
  fn depth_format() -> Option<PixelFormat> { None }

  fn new_depth_slot(framebuffer: &mut C::Framebuffer, _: D::Size, _: u32) -> Self {
    C::disable_depth_slot(framebuffer)
  }
}

impl<C, L, D, P> DepthSlot<C, L, D> for Slot<C, L, D, P>
    where C: HasFramebuffer + HasTexture,
          L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P: DepthPixel {
  fn depth_format() -> Option<PixelFormat> { Some(P::pixel_format()) }

  fn new_depth_slot(framebuffer: &mut C::Framebuffer, size: D::Size, mipmaps: u32) -> Self {
    let depth_slot = create_slot(size, mipmaps);

    C::accept_depth_slot::<D>(framebuffer, size, &depth_slot.texture.repr);

    depth_slot
  }
}

/// Create a new slot from a size, mipmaps and static properties of the target texture.
fn create_slot<C, L, D, P>(size: D::Size, mipmaps: u32) -> Slot<C, L, D, P>
    where C: HasTexture,
          L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P: Pixel {
  Slot {
    texture: Texture::new(size, mipmaps, &Default::default())
  }
}
