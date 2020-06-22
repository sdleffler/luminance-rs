use crate::Backend;

pub use luminance::tess::{
  Deinterleaved, DeinterleavedData, Interleaved, Mode, TessError, TessIndexType, TessMapError,
  TessViewError,
};

pub type TessBuilder<'a, V, I, W, S> = luminance::tess::TessBuilder<'a, Backend, V, I, W, S>;
pub type Tess<V, I, W, S> = luminance::tess::Tess<Backend, V, I, W, S>;
pub type Vertices<V, I, W, S, T> = luminance::tess::Vertices<Backend, V, I, W, S, T>;
pub type VerticesMut<V, I, W, S, T> = luminance::tess::VerticesMut<Backend, V, I, W, S, T>;
pub type Indices<V, I, W, S> = luminance::tess::Indices<Backend, V, I, W, S>;
pub type IndicesMut<V, I, W, S> = luminance::tess::IndicesMut<Backend, V, I, W, S>;
pub type Instances<V, I, W, S, T> = luminance::tess::Instances<Backend, V, I, W, S, T>;
pub type InstancesMut<V, I, W, S, T> = luminance::tess::InstancesMut<Backend, V, I, W, S, T>;
pub type TessView<'a, V, I, W, S> = luminance::tess::TessView<'a, Backend, V, I, W, S>;
