use crate::Backend;

pub type Buffer<T> = luminance::buffer::Buffer<Backend, T>;
pub use luminance::buffer::{BufferError, BufferSlice, BufferSliceMut};
