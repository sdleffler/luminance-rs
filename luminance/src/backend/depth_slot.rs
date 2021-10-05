//! Depth slot backend interface.
//!
//! This interface defines the low-level API depth slots must implement to be usable.
//!
//! # Note to backend contributors
//!
//! If you are implementing a backend, there is nothing here to implement, but you will likely want to use some traits /
//! types from this module to use as constraint.

use crate::{
  backend::{framebuffer::Framebuffer, texture::Texture as TextureBackend},
  context::GraphicsContext,
  framebuffer::FramebufferError,
  pixel::{DepthPixel, PixelFormat},
  texture::{Dimensionable, Sampler, Texture},
};

/// A depth slot.
///
/// A depth slot represents the associated _depth data_ within a [`Framebuffer`]. This type is entirely constructed at
/// compile-time to ensure type safety. Even though this trait lives on the backend side of luminance, no backend is
/// supposed to implement it, but instead use it.
///
/// Two types of depth slots exist:
///
/// - None, represented by the `()` implementor.
/// - A single depth [`Texture`]. This type of depth slot is often suitable for renderable framebuffer. The pixel format
///   must implement [`DepthPixel`].
///
/// Feel free to have a look at the list of implementors of this trait to know which types you can use as color slots.
pub trait DepthSlot<B, D>
where
  B: ?Sized + Framebuffer<D>,
  D: Dimensionable,
  D::Size: Copy,
{
  /// Data associated with this color slot. Either a `()` or a [`Texture`] depending on the [`Framebuffer`].
  type DepthTexture;

  /// Turn a depth slot into a pixel format.
  fn depth_format() -> Option<PixelFormat>;

  /// Reify a raw texture into a depth slot.
  fn reify_depth_texture<C>(
    ctx: &mut C,
    size: D::Size,
    mipmaps: usize,
    sampler: &Sampler,
    framebuffer: &mut B::FramebufferRepr,
  ) -> Result<Self::DepthTexture, FramebufferError>
  where
    C: GraphicsContext<Backend = B>;
}

impl<B, D> DepthSlot<B, D> for ()
where
  B: ?Sized + Framebuffer<D>,
  D::Size: Copy,
  D: Dimensionable,
{
  type DepthTexture = ();

  fn depth_format() -> Option<PixelFormat> {
    None
  }

  fn reify_depth_texture<C>(
    _: &mut C,
    _: D::Size,
    _: usize,
    _: &Sampler,
    _: &mut B::FramebufferRepr,
  ) -> Result<Self::DepthTexture, FramebufferError>
  where
    C: GraphicsContext<Backend = B>,
  {
    Ok(())
  }
}

impl<B, D, P> DepthSlot<B, D> for P
where
  B: ?Sized + Framebuffer<D> + TextureBackend<D, P>,
  D: Dimensionable,
  D::Size: Copy,
  P: DepthPixel,
{
  type DepthTexture = Texture<B, D, P>;

  fn depth_format() -> Option<PixelFormat> {
    Some(P::pixel_format())
  }

  fn reify_depth_texture<C>(
    ctx: &mut C,
    size: D::Size,
    mipmaps: usize,
    sampler: &Sampler,
    framebuffer: &mut B::FramebufferRepr,
  ) -> Result<Self::DepthTexture, FramebufferError>
  where
    C: GraphicsContext<Backend = B>,
  {
    let texture = Texture::new_no_texels(ctx, size, mipmaps, sampler.clone())?;
    unsafe { B::attach_depth_texture(framebuffer, &texture.repr)? };

    Ok(texture)
  }
}
