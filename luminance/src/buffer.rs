//! Static GPU typed arrays.
//!
//! A GPU buffer is a typed continuous region of data. It has a size and can hold several elements.
//!
//! Buffers are created with the `new` associated function. You pass in the number of elements you
//! want in the buffer.
//!
//! ```
//! let buffer: Buffer<f32> = Buffer::new(5);
//! ```
//! Once the buffer is created, you can perform several operations on them:
//!
//! - writing to them ;
//! - reading from them ;
//! - passing them around as uniforms ;
//! - etc.
//!
//! # Writing to a buffer
//!
//! `Buffer`s support several write methods. The simple one is *clearing*. That is, replacing the
//! whole content of the buffer with a single value. Use the `fill` function to do so.
//!
//! ```
//! buffer.clear(0.);
//! ```
//!
//! If you want to clear the buffer by providing a value for each elements, you want *filling*. Use
//! the `fill` function:
//!
//! ```
//! buffer.fill([1, 2, 3, 4, 5]);
//! ```
//!
//! If you want to change a value at a given index, you can use the `set` function.
//!
//! ```
//! buffer.set(3, 3.14);
//! ```
//!
//! # Reading from the buffer
//!
//! You can either retrieve the `whole` content of the `Buffer` or `get` a value with an index.
//!
//! ```
//! // get the whole content
//! let all_elems = buffer.whole();
//! assert_eq!(all_elems.len(), 5);
//! assert_eq!(all_elemns, vec![1, 2, 3, 3.14, 5]); // admit floating equalities
//!
//! // get the element at index 3
//! assert_eq!(buffer.get(3), Some(3.14));
//! ```

use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::slice;
use std::vec::Vec;

use linear::{M22, M33, M44};
use texture::Unit;

/// Implement this trait to provide buffers.
pub unsafe trait HasBuffer {
  /// A type representing minimal information to operate on a buffer. For instance, a size, a
  /// pointer, a method to retrieve data, a handle, whatever.
  type ABuffer;

  /// Create a new buffer with a given size.
  fn new(size: usize) -> Self::ABuffer;
  /// Destroy a buffer.
  fn free(&mut Self::ABuffer);
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
  /// `Err(BufferError::TooFewValues)` if you provide less values than the buffer’s size. In that
  /// case, all `values` are written and the missing ones remain the same in `buffer`.
  fn write_whole<T>(buffer: &Self::ABuffer, values: &[T]) -> Result<(),BufferError>;
  /// Write a single value in the buffer at a given offset.
  ///
  /// # Failures
  ///
  /// `Err(BufferError::Overflow)` if you provide an offset that doesn’t lie in the allocated GPU
  /// region.
  fn write<T>(buffer: &Self::ABuffer, offset: usize, x: T) -> Result<(), BufferError> where T: Copy;
  /// Read all values from the buffer.
  fn read_whole<T>(buffer: &Self::ABuffer, nb: usize) -> Vec<T> where T: Copy;
  /// Read a single value from the buffer at a given offset.
  ///
  /// # Failures
  ///
  /// `None` if you provide an offset that doesn’t lie in the allocated GPU region.
  fn read<T>(buffer: &Self::ABuffer, offset: usize) -> Option<T> where T: Copy;
  fn map<T>(&mut Self::ABuffer) -> *const T;
  fn map_mut<T>(&mut Self::ABuffer) -> *mut T;
  fn unmap(&mut Self::ABuffer);
}

/// Buffer errors.
#[derive(Debug, Eq, PartialEq)]
pub enum BufferError {
  Overflow,
  TooFewValues,
  TooManyValues,
  MapFailed
}

/// A `Buffer` is a GPU region you can picture as an array. It has a static size and cannot be
/// resized. The size is expressed in number of elements lying in the buffer, not in bytes.
#[derive(Debug)]
pub struct Buffer<C, T> where C: HasBuffer {
  pub repr: C::ABuffer,
  pub size: usize,
  _t: PhantomData<T>
}

impl<C, T> Buffer<C, T> where C: HasBuffer {
  /// Create a new `Buffer` with a given number of elements.
  pub fn new(size: usize) -> Buffer<C, T> {
    let buffer = C::new(size * mem::size_of::<T>());
    Buffer {
      repr: buffer,
      size: size,
      _t: PhantomData
    }
  }

  /// Retrieve an element from the `Buffer`.
  ///
  /// Checks boundaries.
  pub fn at(&self, i: u32) -> Option<T> where T: Copy {
    C::read(&self.repr, i as usize * mem::size_of::<T>())
  }

  /// Retrieve the whole content of the `Buffer`.
  pub fn whole(&self) -> Vec<T> where T: Copy {
    C::read_whole(&self.repr, self.size)
  }

  /// Set a value at a given index in the `Buffer`.
  ///
  /// Checks boundaries.
  pub fn set(&mut self, i: u32, x: T) -> Result<(), BufferError> where T: Copy {
    C::write(&self.repr, i as usize * mem::size_of::<T>(), x)
  }

  /// Fill the `Buffer` with a single value.
  pub fn clear(&self, x: T) where T: Copy {
    let _ = C::write_whole(&self.repr, &vec![x; self.size]);
  }

  /// Fill the whole buffer with an array.
  pub fn fill(&self, values: &[T]) {
    let _ = C::write_whole(&self.repr, values);
  }

  pub fn get(&mut self) -> Result<BufferSlice<C, T>, BufferError> {
    let p = C::map(&mut self.repr);

    if p.is_null() {
      return Err(BufferError::MapFailed);
    }

    Ok(BufferSlice {
      buf: &mut self.repr,
      ptr: p,
      len: self.size
    })
  }

  pub fn get_mut(&mut self) -> Result<BufferSliceMut<C, T>, BufferError> {
    let p = C::map_mut(&mut self.repr);

    if p.is_null() {
      return Err(BufferError::MapFailed);
    }

    Ok(BufferSliceMut {
      buf: &mut self.repr,
      ptr: p,
      len: self.size
    })
  }
}

impl<C, T> Drop for Buffer<C, T> where C: HasBuffer {
  fn drop(&mut self) {
    C::free(&mut self.repr)
  }
}

pub struct BufferSlice<'a, C, T> where C: 'a + HasBuffer, T: 'a {
  /// Borrowed buffer.
  pub buf: &'a mut C::ABuffer,
  /// Raw pointer into the GPU memory.
  ptr: *const T,
  /// Number of elements in the GPU memory.
  len: usize
}

impl<'a, C, T> BufferSlice<'a, C, T> where C: 'a + HasBuffer, T: 'a {
  pub fn map(buf: &'a mut C::ABuffer, len: usize) -> Result<Self, BufferError> {
    let p = C::map(buf);

    if p.is_null() {
      return Err(BufferError::MapFailed);
    }

    Ok(BufferSlice {
      buf: buf,
      ptr: p,
      len: len
    })
  }
}

impl<'a, C, T> Drop for BufferSlice<'a, C, T> where C: 'a + HasBuffer, T: 'a {
  fn drop(&mut self) {
    C::unmap(self.buf)
  }
}

impl<'a, C, T> Deref for BufferSlice<'a, C, T> where C: 'a + HasBuffer, T: 'a {
  type Target = [T];

  fn deref(&self) -> &Self::Target {
    unsafe { slice::from_raw_parts(self.ptr, self.len) }
  }
}

pub struct BufferSliceMut<'a, C, T> where C: 'a + HasBuffer, T: 'a {
  /// Borrowed buffer.
  pub buf: &'a mut C::ABuffer,
  /// Raw pointer into the GPU memory.
  ptr: *mut T,
  /// Number of elements in the GPU memory.
  len: usize
}

impl<'a, C, T> BufferSliceMut<'a, C, T> where C: 'a + HasBuffer, T: 'a {
  pub fn map_mut(buf: &'a mut C::ABuffer, len: usize) -> Result<Self, BufferError> {
    let p = C::map_mut(buf);

    if p.is_null() {
      return Err(BufferError::MapFailed);
    }

    Ok(BufferSliceMut {
      buf: buf,
      ptr: p,
      len: len
    })
  }
}

impl<'a, C, T> Drop for BufferSliceMut<'a, C, T> where C: 'a + HasBuffer, T: 'a {
  fn drop(&mut self) {
    C::unmap(self.buf)
  }
}

impl<'a, C, T> Deref for BufferSliceMut<'a, C, T> where C: 'a + HasBuffer, T: 'a {
  type Target = [T];

  fn deref(&self) -> &Self::Target {
    unsafe { slice::from_raw_parts(self.ptr, self.len) }
  }
}

impl<'a, C, T> DerefMut for BufferSliceMut<'a, C, T> where C: 'a + HasBuffer, T: 'a {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { slice::from_raw_parts_mut(self.ptr, self.len) }
  }
}

/// Buffer binding.
pub struct Binding {
  index: u32
}

impl Binding {
  pub fn new(index: u32) -> Self {
    Binding {
      index: index
    }
  }
}

impl Deref for Binding {
  type Target = u32;

  fn deref(&self) -> &Self::Target {
    &self.index
  }
}

impl DerefMut for Binding {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.index
  }
}

/// An opaque type representing any uniform buffer.
pub struct UniformBufferProxy<'a, C> where C: HasBuffer + 'a {
  pub repr: &'a C::ABuffer
}

impl<'a, C, T> From<&'a Buffer<C, T>> for UniformBufferProxy<'a, C>
    where C: HasBuffer,
          T: UniformBlock {
  fn from(buffer: &'a Buffer<C, T>) -> Self {
    UniformBufferProxy {
      repr: &buffer.repr
    }
  }
}

pub trait UniformBlock {}

impl UniformBlock for u32 {}
impl UniformBlock for i32 {}
impl UniformBlock for f32 {}
impl UniformBlock for bool {}
impl UniformBlock for M22 {}
impl UniformBlock for M33 {}
impl UniformBlock for M44 {}
impl UniformBlock for [u32; 2] {}
impl UniformBlock for [i32; 2] {}
impl UniformBlock for [f32; 2] {}
impl UniformBlock for [bool; 2] {}
impl UniformBlock for [u32; 3] {}
impl UniformBlock for [i32; 3] {}
impl UniformBlock for [f32; 3] {}
impl UniformBlock for [bool; 3] {}
impl UniformBlock for [u32; 4] {}
impl UniformBlock for [i32; 4] {}
impl UniformBlock for [f32; 4] {}
impl UniformBlock for [bool; 4] {}
impl UniformBlock for Unit {}
impl UniformBlock for Binding {}
impl<T> UniformBlock for [T] where T: UniformBlock {}

macro_rules! impl_uniform_block_tuple {
  ($( $t:ident ),*) => {
    impl<$($t),*> UniformBlock for ($($t),*) where $($t: UniformBlock),* {}
  }
}

impl_uniform_block_tuple!(A, B);
impl_uniform_block_tuple!(A, B, C);
impl_uniform_block_tuple!(A, B, C, D);
impl_uniform_block_tuple!(A, B, C, D, E);
impl_uniform_block_tuple!(A, B, C, D, E, F);
impl_uniform_block_tuple!(A, B, C, D, E, F, G);
impl_uniform_block_tuple!(A, B, C, D, E, F, G, H);
impl_uniform_block_tuple!(A, B, C, D, E, F, G, H, I);
impl_uniform_block_tuple!(A, B, C, D, E, F, G, H, I, J);
