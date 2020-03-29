//! Graphics context.
//!
//! # Graphics context and backends
//!
//! A graphics context is an external type typically implemented by other crates and which provides
//! support for backends. Its main scope is to unify all possible implementations of backends
//! behind a single trait: [`GraphicsContext`]. A [`GraphicsContext`] really only requires two items
//! to be implemented:
//!
//! - The type of the backend to use — [`GraphicsContext::Backend`]. That type will often be used
//!   to access the GPU, cache costly operations, etc.
//! - A method to get a mutable access to the underlying backend — [`GraphicsContext::backend`].
//!
//! Most of the time, if you want to work with _any_ windowing implementation, you will want to
//! use a type variable such as `C: GraphicsContext`. If you want to work with any context
//! supporting a specific backend, use `C: GraphicsContext<Backend = YourBackendType`. Etc.
//!
//! This crate doesn’t provide you with creating such contexts. Instead, you must do it yourself
//! or rely on crates doing it for you.
//!
//! # Default implementation of helper functions
//!
//! By default, graphics contexts automatically get several methods implemented on them. Those
//! methods are helper functions available to write code in a more elegant and compact way than
//! passing around mutable references on the context. Often, it will help you not having to
//! use type ascription, too, since the [`GraphicsContext::Backend`] type is known when calling
//! those functions.
//!
//! Instead of:
//!
//! ```ignore
//! use luminance::context::GraphicsContext as _;
//! use luminance::buffer::Buffer;
//!
//! let buffer: Buffer<SomeBackendType, u8> = Buffer::from_slice(&mut context, slice).unwrap();
//! ```
//!
//! You can simply do:
//!
//! ```ignore
//! use luminance::context::GraphicsContext as _;
//!
//! let buffer = context.new_buffer_from_slice(slice).unwrap();
//! ```

use crate::backend::buffer::Buffer as BufferBackend;
use crate::buffer::{Buffer, BufferError};
use crate::pipeline::PipelineGate;

/// Class of graphics context.
pub unsafe trait GraphicsContext: Sized {
  type Backend: ?Sized;

  fn backend(&mut self) -> &mut Self::Backend;

  /// Create a new pipeline gate
  fn pipeline_gate(&mut self) -> PipelineGate<Self> {
    PipelineGate::new(self)
  }

  /// Create a new buffer.
  ///
  /// See the documentation of [`Buffer::new`] for further details.
  unsafe fn new_buffer<T>(&mut self, len: usize) -> Result<Buffer<Self::Backend, T>, BufferError>
  where
    Self::Backend: BufferBackend<T>,
  {
    Buffer::new(self, len)
  }

  /// Create a new buffer from a slice.
  ///
  /// See the documentation of [`Buffer::from_slice`] for further details.
  fn new_buffer_from_slice<T, X>(
    &mut self,
    slice: X,
  ) -> Result<Buffer<Self::Backend, T>, BufferError>
  where
    Self::Backend: BufferBackend<T>,
    X: AsRef<[T]>,
  {
    Buffer::from_slice(self, slice)
  }

  /// Create a new buffer by repeating a value.
  ///
  /// See the documentation of [`Buffer::repeat`] for further details.
  fn new_buffer_repeating<T>(
    &mut self,
    len: usize,
    value: T,
  ) -> Result<Buffer<Self::Backend, T>, BufferError>
  where
    Self::Backend: BufferBackend<T>,
    T: Copy,
  {
    Buffer::repeat(self, len, value)
  }
}
