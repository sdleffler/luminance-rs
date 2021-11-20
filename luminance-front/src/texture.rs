use crate::Backend;

pub use luminance::texture::{
  CubeFace, Cubemap, Dim, Dim1, Dim1Array, Dim2, Dim2Array, Dim3, Dimensionable, MagFilter,
  MinFilter, Sampler, TexelUpload, TextureError, Wrap,
};

pub type Texture<D, P> = luminance::texture::Texture<Backend, D, P>;
