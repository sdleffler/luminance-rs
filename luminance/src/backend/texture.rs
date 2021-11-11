//! Texture backend interface.
//!
//! This interface defines the low-level API textures must implement to be usable.
//!
//! In order to add support for textures, you have to implement two traits:
//!
//! - [`TextureBase`], which is a _type family_ providing the backend representation of a texture. This is needed so
//!   that other part of the crate don’t have to rely on a too abstraction.
//! - The rest of the abstraction, bigger, is [`Texture`].
//!
//! You will have to implement both traits to be able to use textures.

use crate::{
  pixel::Pixel,
  texture::{Dimensionable, Sampler, TexelUpload, TextureError},
};

/// Type family giving the backend representation type.
///
/// This type family is type-erased: it doesn’t know whether the texture is a 2D texture or a 3D one or a cubemap.
pub unsafe trait TextureBase {
  /// Backend representation of a texture.
  type TextureRepr;
}

/// Texture interface.
///
/// Implementing this trait requires implementing [`TextureBase`].
///
/// `D` is the _dimension_ of the texture, and must then implement [`Dimensionable`]. `P` is the format of the carried
/// pixels and must then implement [`Pixel`].
pub unsafe trait Texture<D, P>: TextureBase
where
  D: Dimensionable,
  P: Pixel,
{
  /// Create a new texture.
  unsafe fn new_texture(
    &mut self,
    size: D::Size,
    sampler: Sampler,
    texels: TexelUpload<[P::Encoding]>,
  ) -> Result<Self::TextureRepr, TextureError>;

  /// Create a new texture from raw texels.
  unsafe fn new_texture_raw(
    &mut self,
    size: D::Size,
    sampler: Sampler,
    texels: TexelUpload<[P::RawEncoding]>,
  ) -> Result<Self::TextureRepr, TextureError>;

  /// Get the number of mimaps associated with the texture.
  unsafe fn mipmaps(texture: &Self::TextureRepr) -> usize;

  /// Clear a part to texture using a pixel as clear value.
  ///
  /// This method will use the input pixel and will copy it everywhere in the part formed with `offset` and `size`. For
  /// instance, for 2D textures, `offset` and `size` form a rectangle: that rectangle of pixels will be cleared with the
  /// input pixel.
  unsafe fn clear_part(
    texture: &mut Self::TextureRepr,
    offset: D::Offset,
    size: D::Size,
    texel: TexelUpload<P::Encoding>,
  ) -> Result<(), TextureError>;

  /// Clear a texture using a pixel as clear value.
  ///
  /// This method is similar to [`Texture::clear_part`] but instead of clearing a part of it, it will clear the whole
  /// texture at once. The size will match the size of the texture so you do not have to cache it and simply can use the
  /// input `size` value.
  unsafe fn clear(
    texture: &mut Self::TextureRepr,
    size: D::Size,
    texel: TexelUpload<P::Encoding>,
  ) -> Result<(), TextureError>;

  /// Upload texels to a part of a texture.
  ///
  /// This method will use the input texels and will copy them everywhere in the part formed with `offset` and `size`. For
  /// instance, for 2D textures, `offset` and `size` form a rectangle: that rectangle of pixels will be filled with the
  /// provided input texels.
  unsafe fn upload_part(
    texture: &mut Self::TextureRepr,
    offset: D::Offset,
    size: D::Size,
    texels: TexelUpload<[P::Encoding]>,
  ) -> Result<(), TextureError>;

  /// Upload texels to a whole texture.
  ///
  /// This method is similar to [`Texture::upload_part`] but instead of uploading a part of it, it will upload to the
  /// whole texture at once. The size will match the size of the texture so you do not have to cache it and simply can use
  /// the input `size` value.
  unsafe fn upload(
    texture: &mut Self::TextureRepr,
    size: D::Size,
    texels: TexelUpload<[P::Encoding]>,
  ) -> Result<(), TextureError>;

  /// Upload texels to a part of a texture.
  ///
  /// This method will use the input texels and will copy them everywhere in the part formed with `offset` and `size`. For
  /// instance, for 2D textures, `offset` and `size` form a rectangle: that rectangle of pixels will be filled with the
  /// provided input texels.
  ///
  /// > This is very similar to [`Texture::upload_part_raw`], but the key difference is that this method works with the
  /// > _raw encoding_ of the texels, which is often the case with crates that provide you with a contiguous array of raw
  /// > data instead of rich texels.
  unsafe fn upload_part_raw(
    texture: &mut Self::TextureRepr,
    offset: D::Offset,
    size: D::Size,
    texels: TexelUpload<[P::RawEncoding]>,
  ) -> Result<(), TextureError>;

  /// Upload texels to a whole texture.
  ///
  /// This method is similar to [`Texture::upload_part`] but instead of uploading a part of it, it will upload to the
  /// whole texture at once. The size will match the size of the texture so you do not have to cache it and simply can use
  /// the input `size` value.
  ///
  /// > This is very similar to [`Texture::upload`], but the key difference is that this method works with the _raw
  /// > encoding_ of the texels, which is often the case with crates that provide you with a contiguous array of raw
  /// > data instead of rich texels.
  unsafe fn upload_raw(
    texture: &mut Self::TextureRepr,
    size: D::Size,
    texels: TexelUpload<[P::RawEncoding]>,
  ) -> Result<(), TextureError>;

  /// Get a copy of the raw texels stored in the texture.
  ///
  /// `size` will match the actual size of the texture, you do not need to cache it.
  unsafe fn get_raw_texels(
    texture: &Self::TextureRepr,
    size: D::Size,
  ) -> Result<Vec<P::RawEncoding>, TextureError>
  where
    P::RawEncoding: Copy + Default;

  /// Resize the texture.
  ///
  /// Once the texture is resized, pixels are left in an unknown state. Depending on the implementation of the backend,
  /// it is likely that texels will either be old ones, or completely random data.
  unsafe fn resize(
    texture: &mut Self::TextureRepr,
    size: D::Size,
    texel: TexelUpload<[P::Encoding]>,
  ) -> Result<(), TextureError>;

  /// Resize the texture with raw texels.
  ///
  /// Once the texture is resized, pixels are left in an unknown state. Depending on the implementation of the backend,
  /// it is likely that texels will either be old ones, or completely random data.
  unsafe fn resize_raw(
    texture: &mut Self::TextureRepr,
    size: D::Size,
    texel: TexelUpload<[P::RawEncoding]>,
  ) -> Result<(), TextureError>;
}
