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
use rw::RW;
use pixel::{ColorPixel, DepthPixel, Pixel, PixelFormat};
use std::default::Default;
use std::vec::Vec;
use texture::{Dim2, Dimensionable, Flat, HasTexture, Layerable, Texture};

/// Trait to implement to provide framebuffer features.
///
/// When creating a new framebuffer with `new_framebuffer`, the color and depth formats are passed
/// and should be used to create internal textures and/or buffers to represent the slots.
pub trait HasFramebuffer: HasTexture {
	/// Framebuffer representation.
  type Framebuffer;

  /// Create a new framebuffer.
	///
	/// `size` represents the size of the color and depth slots. `mipmaps` is the number of levels
	/// required for those slots. `color_formats` and `depth_format` represent the color buffers and
	/// depth buffer, respectively.
	///
	/// This function should return a tuple containing the framebuffer, a list of textures to use in
	/// place of color buffers and zero or one texture for the depth buffer. On error, it should
	/// return the appropriate error.
  fn new_framebuffer<D>(size: D::Size, mipmaps: u32, color_formats: &Vec<PixelFormat>, depth_format: Option<PixelFormat>) -> Result<(Self::Framebuffer, Vec<Self::ATexture>, Option<Self::ATexture>), FramebufferError> where D: Dimensionable;
  /// Default framebuffer.
  fn default_framebuffer() -> Self::Framebuffer;
	/// Called when no color slot is required.
	fn disable_color_slot(framebuffer: &Self::Framebuffer);
	/// Called when no depth slot is required.
	fn disable_depth_slot(framebuffer: &Self::Framebuffer);
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
  pub texture: Texture<C, L, D, P>
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

impl<A, B> ColorSlot for (A, B) where A: ColorSlot, B: ColorSlot {
  fn color_formats() -> Vec<PixelFormat> {
		Chain::<A, B>::color_formats()
	}
}

impl<A, B, C> ColorSlot for (A, B, C) where A: ColorSlot, B: ColorSlot, C: ColorSlot {
  fn color_formats() -> Vec<PixelFormat> {
		Chain::<A, Chain<B, C>>::color_formats()
	}
}

impl<A, B, C, D> ColorSlot for (A, B, C, D) where A: ColorSlot, B: ColorSlot, C: ColorSlot, D: ColorSlot {
  fn color_formats() -> Vec<PixelFormat> {
		Chain::<A, Chain<B, Chain<C, D>>>::color_formats()
	}
}

impl<A, B, C, D, E> ColorSlot for (A, B, C, D, E) where A: ColorSlot, B: ColorSlot, C: ColorSlot, D: ColorSlot, E: ColorSlot {
  fn color_formats() -> Vec<PixelFormat> {
		Chain::<A, Chain<B, Chain<C, Chain<D, E>>>>::color_formats()
	}
}

impl<A, B, C, D, E, F> ColorSlot for (A, B, C, D, E, F) where A: ColorSlot, B: ColorSlot, C: ColorSlot, D: ColorSlot, E: ColorSlot, F: ColorSlot {
  fn color_formats() -> Vec<PixelFormat> {
		Chain::<A, Chain<B, Chain<C, Chain<D, Chain<E, F>>>>>::color_formats()
	}
}

impl<A, B, C, D, E, F, G> ColorSlot for (A, B, C, D, E, F, G) where A: ColorSlot, B: ColorSlot, C: ColorSlot, D: ColorSlot, E: ColorSlot, F: ColorSlot, G: ColorSlot {
  fn color_formats() -> Vec<PixelFormat> {
		Chain::<A, Chain<B, Chain<C, Chain<D, Chain<E, Chain<F, G>>>>>>::color_formats()
	}
}

impl<A, B, C, D, E, F, G, H> ColorSlot for (A, B, C, D, E, F, G, H) where A: ColorSlot, B: ColorSlot, C: ColorSlot, D: ColorSlot, E: ColorSlot, F: ColorSlot, G: ColorSlot, H: ColorSlot {
  fn color_formats() -> Vec<PixelFormat> {
		Chain::<A, Chain<B, Chain<C, Chain<D, Chain<E, Chain<F, Chain<G, H>>>>>>>::color_formats()
	}
}

impl<A, B, C, D, E, F, G, H, I> ColorSlot for (A, B, C, D, E, F, G, H, I) where A: ColorSlot, B: ColorSlot, C: ColorSlot, D: ColorSlot, E: ColorSlot, F: ColorSlot, G: ColorSlot, H: ColorSlot, I: ColorSlot {
  fn color_formats() -> Vec<PixelFormat> {
		Chain::<A, Chain<B, Chain<C, Chain<D, Chain<E, Chain<F, Chain<G, Chain<H, I>>>>>>>>::color_formats()
	}
}

impl<A, B, C, D, E, F, G, H, I, J> ColorSlot for (A, B, C, D, E, F, G, H, I, J) where A: ColorSlot, B: ColorSlot, C: ColorSlot, D: ColorSlot, E: ColorSlot, F: ColorSlot, G: ColorSlot, H: ColorSlot, I: ColorSlot, J: ColorSlot {
  fn color_formats() -> Vec<PixelFormat> {
		Chain::<A, Chain<B, Chain<C, Chain<D, Chain<E, Chain<F, Chain<G, Chain<H, Chain<I, J>>>>>>>>>::color_formats()
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


trait ToDepthSlot<C, L, D> where C: HasFramebuffer, L: Layerable, D: Dimensionable, D::Size: Copy {
	type Target;

	fn to_depth_slot(framebuffer: &C::Framebuffer, size: D::Size, mipmaps: u32) -> Self;
}

impl<C, L, D> ToDepthSlot<C, L, D> for () where C: HasFramebuffer, L: Layerable, D: Dimensionable, D::Size: Copy {
	type Target = ();

	fn to_depth_slot(framebuffer: &C::Framebuffer, size: D::Size, mipmaps: u32) -> Self {
		C::disable_depth_slot(framebuffer)
	}
}

