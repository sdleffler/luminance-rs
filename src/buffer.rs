//! GPU *buffers*.

use core::marker::PhantomData;
use core::mem;
use std::vec::Vec;

/// Implement this trait to provide buffers.
pub trait HasBuffer {
  /// A type representing minimal information to operate on a buffer. For instance, a size, a
  /// pointer, a method to retrieve data, a handle, whatever.
  type ABuffer;

  /// Create a new buffer with a given size.
  fn new(size: usize) -> Self::ABuffer;
  /// Write values into the buffer.
  ///
  /// # Warnings
  ///
  ///  Those warnings are just **hints**. The behavior for each warning is specific and should be
  ///  accounted.
  ///
  /// `Err(BufferError::TooManyValues)` if you provide more values than the buffer’s size. In that
  /// case, the extra items are just ignored and all others are written; that is, the `values`
  /// argument is considered having the same size as `buffer`.
  ///
  /// `Err(BufferError::TooFewValues)` if you provide less avlues than the buffer’s size. In that
  /// case, all `values` are written and the missing ones remain the same in `buffer`.
  fn write_whole<T>(buffer: &Self::ABuffer, values: &Vec<T>) -> Result<(),BufferError>;
  /// Write a single value in the buffer at a given offset.
  ///
  /// # Failures
  ///
  /// `Err(BufferError::Overflow)` if you provide an offset that doesn’t lie in the allocated GPU
  /// region.
  fn write<T>(buffer: &Self::ABuffer, offset: usize, x: &T) -> Result<(), BufferError> where T: Clone;
  /// Read all values from the buffer.
  fn read_whole<T>(buffer: &Self::ABuffer) -> Vec<T>;
  /// Read a single value from the buffer at a given offset.
  ///
  /// # Failures
  ///
  /// `None` if you provide an offset that doesn’t lie in the allocated GPU region.
  fn read<T>(buffer: &Self::ABuffer, offset: usize) -> Option<T> where T: Clone;
}

/// Buffer errors.
#[derive(Debug)]
pub enum BufferError {
  Overflow,
  TooFewValues,
  TooManyValues
}

/// A `Buffer` is a GPU region you can picture as an array. It has a static size and cannot be
/// resized. The size is expressed in number of elements lying in the buffer, not in bytes.
#[derive(Debug)]
pub struct Buffer<C, A, T> where C: HasBuffer {
  pub repr: C::ABuffer,
  pub size: usize, // FIXME: should be compile-time, not runtime
  _a: PhantomData<A>,
  _t: PhantomData<T>
}

impl<C, A, T> Buffer<C, A, T> where C: HasBuffer {
  /// Create a new `Buffer` with a given number of elements.
  pub fn new(_: A, size: u32) -> Buffer<C, A, T> {
    let size = size as usize;
    let buffer = C::new(size * mem::size_of::<T>());
    Buffer { repr: buffer, size: size, _a: PhantomData, _t: PhantomData }
  }

  /// Retrieve an element from the `Buffer`.
  ///
  /// Checks boundaries.
  pub fn get(&self, i: u32) -> Option<T> where T: Clone {
    C::read(&self.repr, i as usize * mem::size_of::<T>())
  }

  /// Retrieve the whole content of the `Buffer`.
  pub fn whole(&self) -> Vec<T> {
    C::read_whole(&self.repr)
  }

  /// Set a value at a given index in the `Buffer`.
  ///
  /// Checks boundaries.
  pub fn set(&mut self, i: u32, x: &T) -> Result<(), BufferError> where T: Clone {
    C::write(&self.repr, i as usize * mem::size_of::<T>(), x)
  }

  /// Fill the `Buffer` with a single value.
  pub fn clear(&self, x: &T) {
    let _ = C::write_whole(&self.repr, &vec![x; self.size]);
  }
}
