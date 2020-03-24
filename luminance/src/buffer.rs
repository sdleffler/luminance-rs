//! Graphics buffers.
//!
//! A GPU buffer is a typed continuous region of data. It has a size and can hold several elements.
//!
//! Once the buffer is created, you can perform several operations on it:
//!
//! - Writing to it.
//! - Reading from it.
//! - Passing it around as uniforms.
//! - Etc.
//!
//! # Parametricity
//!
//! The [`Buffer`] type is parametric over the backend type and item type. For the backend type,
//! the [`backend::buffer::Buffer`] trait must be implemented to be able to use it with the buffe.
//!
//! # Buffer creation, reading, writing and getting information
//!
//! Buffers are created with the [`Buffer::new`], [`Buffer::from_slice`] and [`Buffer::repeat`]
//! methods. All these methods are fallible — they might fail with [`BufferError`].
//!
//! Once you have a [`Buffer`], you can read from it and write to it.
//! Writing is done with [`Buffer::set`] — which allows to set a value at a given index in the
//! buffer, [`Buffer::write_whole`] — which writes a whole slice to the buffer — and
//! [`Buffer::clear`] — which sets the same value to all items in a buffer. Reading is performed
//! with [`Buffer::at`] — which retrieves the item at a given index and [`Buffer::whole`] which
//! retrieves the whole buffer by copying it to a `Vec<T>`.
//!
//! It’s possible to get data via several methods, such as [`Buffer::len`] to get the number of
//! items in the buffer.
//!
//! # Buffer slicing
//!
//! Buffer slicing is the act of getting a [`BufferSlice`] out of a [`Buffer`]. That operation
//! allows to _map_ a GPU region onto a CPU one and access the underlying data as a regular slice.
//! Two methods exist to slice a buffer
//!
//! - Read-only: [`Buffer::slice`].
//! - Mutably: [`Buffer::slice_mut`].
//!
//! Both methods take a mutable reference on a buffer because even in the read-only case, exclusive
//! borrowing must be enforced.
//!
//! [`backend::buffer::Buffer`]: crate::backend::buffer::Buffer

use crate::backend::buffer::{Buffer as BufferBackend, BufferSlice as BufferSliceBackend};
use crate::context::GraphicsContext;

use std::fmt;
use std::marker::PhantomData;

/// A GPU buffer.
///
/// # Parametricity
///
/// `B` is the backend type. It must implement [`backend::buffer::Buffer`].
///
/// [`backend::buffer::Buffer`]: crate::backend::buffer::Buffer
///
/// `T` is the type of stored items. No restriction are currently enforced on that type, besides
/// the fact it must be [`Sized`].
#[derive(Debug)]
pub struct Buffer<B, T>
where
  B: ?Sized + BufferBackend<T>,
{
  pub(crate) repr: B::BufferRepr,
  _t: PhantomData<T>,
}

impl<B, T> Drop for Buffer<B, T>
where
  B: ?Sized + BufferBackend<T>,
{
  fn drop(&mut self) {
    unsafe { <B as BufferBackend<T>>::destroy_buffer(&mut self.repr) };
  }
}

impl<B, T> Buffer<B, T>
where
  B: ?Sized + BufferBackend<T>,
{
  /// Create a new buffer with a given length
  ///
  /// The buffer will be created on the GPU with a contiguous region large enough to fit `len`
  /// items.
  ///
  /// # Safety
  ///
  /// This function is `unsafe` because it allocates memory without initializing it. If you need
  /// a safe version, consider using [`Buffer::from_slice`] or [`Buffer::repeat`] instead.
  ///
  /// # Errors
  ///
  /// That function can fail creating the buffer for various reasons, in which case it returns
  /// `Err(BufferError::_)`. Feel free to read the documentation of [`BufferError`] for further
  /// information.
  ///
  /// # Notes
  ///
  /// You might be interested in the [`GraphicsContext::new_buffer`] function instead, which
  /// is the exact same function, but benefits from more type inference (based on `&mut C`).
  pub unsafe fn new<C>(ctx: &mut C, len: usize) -> Result<Self, BufferError>
  where
    C: GraphicsContext<Backend = B>,
  {
    let repr = ctx.backend().new_buffer(len)?;

    Ok(Buffer {
      repr,
      _t: PhantomData,
    })
  }

  /// Create a new buffer from a slice of items.
  ///
  /// The buffer will be created with a length equal to the length of the input size, and items
  /// will be copied from the slice inside the contiguous GPU region.
  ///
  /// # Errors
  ///
  /// That function can fail creating the buffer for various reasons, in which case it returns
  /// `Err(BufferError::_)`. Feel free to read the documentation of [`BufferError`] for further
  /// information.
  ///
  /// # Notes
  ///
  /// You might be interested in the [`GraphicsContext::new_buffer_from_slice`] function instead,
  /// which is the exact same function, but benefits from more type inference (based on `&mut C`).
  pub fn from_slice<C, X>(ctx: &mut C, slice: X) -> Result<Self, BufferError>
  where
    C: GraphicsContext<Backend = B>,
    X: AsRef<[T]>,
  {
    let repr = unsafe { ctx.backend().from_slice(slice)? };

    Ok(Buffer {
      repr,
      _t: PhantomData,
    })
  }

  /// Create a new buffer by repeating `len` times a `value`.
  ///
  /// The buffer will be comprised of `len` items, all equal to `value`.
  ///
  /// # Errors
  ///
  /// That function can fail creating the buffer for various reasons, in which case it returns
  /// `Err(BufferError::_)`. Feel free to read the documentation of [`BufferError`] for further
  /// information.
  ///
  /// # Notes
  ///
  /// You might be interested in the [`GraphicsContext::new_buffer_repeating`] function instead,
  /// which is the exact same function, but benefits from more type inference (based on `&mut C`).
  pub fn repeat<C>(ctx: &mut C, len: usize, value: T) -> Result<Self, BufferError>
  where
    C: GraphicsContext<Backend = B>,
    T: Copy,
  {
    let repr = unsafe { ctx.backend().repeat(len, value)? };

    Ok(Buffer {
      repr,
      _t: PhantomData,
    })
  }

  /// Get the item at the given index.
  pub fn at(&self, i: usize) -> Option<T>
  where
    T: Copy,
  {
    unsafe { B::at(&self.repr, i) }
  }

  /// Get the whole content of the buffer and store it inside a [`Vec`].
  pub fn whole(&self) -> Vec<T>
  where
    T: Copy,
  {
    unsafe { B::whole(&self.repr) }
  }

  /// Set a value `x` at index `i` in the buffer.
  ///
  /// # Errors
  ///
  /// That function returns [`BufferError::Overflow`] if `i` is bigger than the length of the
  /// buffer. Other errors are possible; please consider reading the documentation of
  /// [`BufferError`] for further information.
  pub fn set(&mut self, i: usize, x: T) -> Result<(), BufferError>
  where
    T: Copy,
  {
    unsafe { B::set(&mut self.repr, i, x) }
  }

  /// Set the content of the buffer by using a slice that will be copied at the buffer’s memory
  /// location.
  ///
  /// # Errors
  ///
  /// [`BufferError::TooFewValues`] is returned if the input slice has less items than the buffer.
  ///
  /// [`BufferError::TooManyValues`] is returned if the input slice has more items than the buffer.
  pub fn write_whole(&mut self, values: &[T]) -> Result<(), BufferError> {
    unsafe { B::write_whole(&mut self.repr, values) }
  }

  /// Clear the content of the buffer by copying the same value everywhere.
  pub fn clear(&mut self, x: T) -> Result<(), BufferError>
  where
    T: Copy,
  {
    unsafe { B::clear(&mut self.repr, x) }
  }

  /// Return the length of the buffer (i.e. the number of elements).
  #[inline(always)]
  pub fn len(&self) -> usize {
    unsafe { B::len(&self.repr) }
  }

  /// Check whether the buffer is empty (i.e. it has no elements).
  ///
  /// # Note
  ///
  /// Since right now, it is not possible to grow vectors, it is highly recommended not to create
  /// empty buffers. That function is there only for convenience and demonstration; you shouldn’t
  /// really have to use it.
  #[inline(always)]
  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }
}

impl<B, T> Buffer<B, T>
where
  B: ?Sized + BufferSliceBackend<T>,
{
  /// Create a new [`BufferSlice`] from a buffer, allowing to get `&[T]` out of it.
  ///
  /// # Errors
  ///
  /// That function might fail and return a [`BufferError::MapFailed`].
  pub fn slice(&mut self) -> Result<BufferSlice<B, T>, BufferError> {
    unsafe {
      B::slice_buffer(&mut self.repr).map(|slice| BufferSlice {
        slice,
        _a: PhantomData,
      })
    }
  }

  /// Create a new [`BufferSliceMut`] from a buffer, allowing to get `&mut [T]` out of it.
  ///
  /// # Errors
  ///
  /// That function might fail and return a [`BufferError::MapFailed`].
  pub fn slice_mut(&mut self) -> Result<BufferSliceMut<B, T>, BufferError> {
    unsafe {
      B::slice_buffer_mut(&mut self.repr).map(|slice| BufferSliceMut {
        slice,
        _a: PhantomData,
      })
    }
  }
}

/// Buffer errors.
///
/// Please keep in mind that this `enum` is _non exhaustive_; you will not be able to exhaustively
/// pattern-match against it.
#[non_exhaustive]
#[derive(Debug, Eq, PartialEq)]
pub enum BufferError {
  /// Overflow when setting a value with a specific index.
  ///
  /// Contains the index and the size of the buffer.
  Overflow { index: usize, buffer_len: usize },

  /// Too few values were passed to fill a buffer.
  ///
  /// Contains the number of passed value and the size of the buffer.
  TooFewValues {
    provided_len: usize,
    buffer_len: usize,
  },

  /// Too many values were passed to fill a buffer.
  ///
  /// Contains the number of passed value and the size of the buffer.
  TooManyValues {
    provided_len: usize,
    buffer_len: usize,
  },

  /// Buffer mapping failed.
  MapFailed,
}

impl fmt::Display for BufferError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      BufferError::Overflow { index, buffer_len } => write!(
        f,
        "buffer overflow (index = {}, size = {})",
        index, buffer_len
      ),

      BufferError::TooFewValues {
        provided_len,
        buffer_len,
      } => write!(
        f,
        "too few values passed to the buffer (nb = {}, size = {})",
        provided_len, buffer_len
      ),

      BufferError::TooManyValues {
        provided_len,
        buffer_len,
      } => write!(
        f,
        "too many values passed to the buffer (nb = {}, size = {})",
        provided_len, buffer_len
      ),

      BufferError::MapFailed => write!(f, "buffer mapping failed"),
    }
  }
}

/// A buffer slice, allowing to get `&[T]`.
#[derive(Debug)]
pub struct BufferSlice<'a, B, T>
where
  B: ?Sized + BufferSliceBackend<T>,
{
  slice: B::SliceRepr,
  _a: PhantomData<&'a mut ()>,
}

impl<'a, B, T> Drop for BufferSlice<'a, B, T>
where
  B: ?Sized + BufferSliceBackend<T>,
{
  fn drop(&mut self) {
    {
      unsafe { B::destroy_buffer_slice(&mut self.slice) };
    };
  }
}

impl<'a, B, T> BufferSlice<'a, B, T>
where
  B: ?Sized + BufferSliceBackend<T>,
{
  /// Obtain a `&[T]`.
  ///
  /// # Errors
  ///
  /// It is possible that obtaining a slice is not possible. In that case,
  /// [`BufferError::MapFailed`] is returned.
  pub fn as_slice(&self) -> Result<&[T], BufferError> {
    unsafe { B::obtain_slice(&self.slice) }
  }
}

/// A buffer mutable slice, allowing to get `&mut [T]`.
#[derive(Debug)]
pub struct BufferSliceMut<'a, B, T>
where
  B: ?Sized + BufferSliceBackend<T>,
{
  slice: B::SliceMutRepr,
  _a: PhantomData<&'a mut ()>,
}

impl<'a, B, T> Drop for BufferSliceMut<'a, B, T>
where
  B: ?Sized + BufferSliceBackend<T>,
{
  fn drop(&mut self) {
    unsafe { B::destroy_buffer_slice_mut(&mut self.slice) };
  }
}

impl<'a, B, T> BufferSliceMut<'a, B, T>
where
  B: ?Sized + BufferSliceBackend<T>,
{
  /// Obtain a `&mut [T]`.
  pub fn as_slice_mut(&mut self) -> Result<&mut [T], BufferError> {
    unsafe { B::obtain_slice_mut(&mut self.slice) }
  }
}
