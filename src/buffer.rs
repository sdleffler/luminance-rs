use core::marker::PhantomData;
use core::mem;
use std::vec::Vec;
use std::ops::Index;

/// Implement this trait to provide buffers.
pub trait HasBuffer {
  /// A type representing minimal information to operate on a buffer. For instance, a size, a
  /// pointer, a method to retrieve data, a handle, whatever.
  type ABuffer;

  /// Create a new buffer with a given size.
  fn new(size: usize) -> Self::ABuffer;
  /// Write values into the buffer.
  fn write_whole<T>(buffer: &Self::ABuffer, values: &Vec<T>);
  /// Write a single value in the buffer at a given offset.
  ///
  /// # Failures
  ///
  /// `Err(BufferError::Overflow)` if you provide an offset that doesn’t lie in the allocated GPU
  /// region.
  fn write<T>(buffer: &Self::ABuffer, offset: usize, x: &T) -> Result<(), BufferError>;
  /// Read all values from the buffer.
  fn read_whole<T>(buffer: &Self::ABuffer) -> Vec<T>;
  /// Read a single value from the buffer at a given offset.
  ///
  /// # Failures
  ///
  /// `None` if you provide an offset that doesn’t lie in the allocated GPU region.
  fn read<T>(buffer: &Self::ABuffer, offset: usize) -> Option<&T>;
}

/// Buffer errors.
#[derive(Debug)]
pub enum BufferError {
    Overflow
  , TooManyValues
}

/// A `Buffer` is a GPU region you can picture as an array. It has a static size and cannot be
/// resized. The size is expressed in number of elements lying in the buffer, not in bytes.
#[derive(Debug)]
pub struct Buffer<C, A, T> where C: HasBuffer {
    repr: C::ABuffer
  , size: usize // FIXME: should be compile-time, not runtime
  , _a: PhantomData<A>
  , _t: PhantomData<T>
}

impl<C, A, T> Buffer<C, A, T> where C: HasBuffer {
  pub fn new(_: A, size: u32) -> Buffer<C, A, T> {
    let size = size as usize;
    let buffer = C::new(size * mem::size_of::<T>());
    Buffer { repr: buffer, size: size, _a: PhantomData, _t: PhantomData }
  }

  pub fn get(&self, i: u32) -> Option<&T> {
    C::read(&self.repr, i as usize * mem::size_of::<T>())
  }

  pub fn set(&mut self, i: u32, x: &T) -> Result<(), BufferError> {
    C::write(&self.repr, i as usize * mem::size_of::<T>(), x)
  }

}

impl<C, A, T> Buffer<C, A, T> where C: HasBuffer, T: Clone {
  /// Fill a `Buffer` with a single value.
  pub fn clear(&self, x: T) {
    C::write_whole(&self.repr, &vec![x; self.size]);
  }
}

impl<C, A, T> Index<u32> for Buffer<C, A, T> where C: HasBuffer {
  type Output = T;

  fn index(&self, i: u32) -> &T {
		self.get(i).unwrap()
  }
}

//impl<C, A, T> IndexMut<u32> for Buffer<C, A, T> where C: HasBuffer {
//  fn index_mut(&mut self, i: u32) -> &mut T {
//  }
//}
