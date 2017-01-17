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
//! However, you cannot change their size.
//!
//! # Writing to a buffer
//!
//! `Buffer`s support several write methods. The simple one is *clearing*. That is, replacing the
//! whole content of the buffer with a single value. Use the `clear` function to do so.
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
//! assert_eq!(all_elems, vec![1, 2, 3, 3.14, 5]); // admit floating equalities
//!
//! // get the element at index 3
//! assert_eq!(buffer.get(3), Some(3.14));
//! ```
//!
//! # Uniform buffer
//!
//! It’s possible to use buffers as *uniform buffers*. That is, buffers that will be in bound at
//! rendering time and which content will be available for a shader to read (no write).
//!
//! In order to use your buffers in a uniform context, the inner type has to implement
//! `UniformBlock`. Keep in mind alignment must be respected and is a bit peculiar. TODO: explain
//! std140 here.

use gl;
use gl::types::*;
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::os::raw::c_void;
use std::ptr;
use std::slice;
use std::vec::Vec;

use linear::{M22, M33, M44};

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
pub struct Buffer<T> {
  handle: GLuint,
  bytes: usize,
  len: usize,
  _t: PhantomData<T>
}

impl<T> Buffer<T> {
  /// Create a new `Buffer` with a given number of elements.
  pub fn new(len: usize) -> Buffer<T> {
    let mut buffer: GLuint = 0;
    let bytes = mem::size_of::<T> * len;

    unsafe {
      gl::GenBuffers(1, &mut buffer);
      gl::BindBuffer(gl::ARRAY_BUFFER, buffer);
      gl::BufferData(gl::ARRAY_BUFFER, bytes as isize, ptr::null(), gl::STREAM_DRAW);
      gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    }

    Buffer {
      handle: buffer,
      bytes: bytes,
      len: len,
      _t: PhantomData
    }
  }

  /// Get the length of the buffer.
  pub fn len(&self) -> usize {
    self.len
  }

  /// Retrieve an element from the `Buffer`.
  ///
  /// Checks boundaries.
  fn at<T>(&self, i: usize) -> Option<T> where T: Copy {
    if i >= self.len {
      return None;
    }

    unsafe {
      gl::BindBuffer(gl::ARRAY_BUFFER, self.handle);
      let ptr = gl::MapBuffer(gl::ARRAY_BUFFER, gl::READ_ONLY) as *const T;

      let x = *ptr.offset(i as isize);

      let _ = gl::UnmapBuffer(gl::ARRAY_BUFFER);
      gl::BindBuffer(gl::ARRAY_BUFFER, 0);

      Some(x)
    }
  }

  /// Retrieve the whole content of the `Buffer`.
  fn whole<T>(&self) -> Vec<T> where T: Copy {
    unsafe {
      gl::BindBuffer(gl::ARRAY_BUFFER, self.handle);
      let ptr = gl::MapBuffer(gl::ARRAY_BUFFER, gl::READ_ONLY) as *const T;

      let values = Vec::from_raw_parts(ptr, self.len, self.len);

      let _ = gl::UnmapBuffer(gl::ARRAY_BUFFER);
      gl::BindBuffer(gl::ARRAY_BUFFER, 0);

      values
    }
  }

  /// Set a value at a given index in the `Buffer`.
  ///
  /// Checks boundaries.
  pub fn set(&mut self, i: u32, x: T) -> Result<(), BufferError> where T: Copy {
    if i >= self.len {
      return Err(BufferError::Overflow);
    }

    unsafe {
      gl::BindBuffer(gl::ARRAY_BUFFER, self.handle);
      let ptr = gl::MapBuffer(gl::ARRAY_BUFFER, gl::WRITE_ONLY) as *mut T;

      *ptr.offset(i as isize) = x;

      let _ = gl::UnmapBuffer(gl::ARRAY_BUFFER);
      gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    }

    Ok(())
  }

  fn write_whole<T>(&self, values: &[T]) -> Result<(), BufferError> {
    let in_bytes = values.len() * mem::size_of::<T>();

    // generate warning and recompute the proper number of bytes to copy
    let (warning, real_bytes) = match in_bytes.cmp(&self.bytes) {
      Less => (Some(BufferError::TooFewValues), in_bytes),
      Greater => (Some(BufferError::TooManyValues), self.bytes),
      _ => (None, in_bytes)
    };

    unsafe {
      gl::BindBuffer(gl::ARRAY_BUFFER, self.handle);
      let ptr = gl::MapBuffer(gl::ARRAY_BUFFER, gl::WRITE_ONLY);

      ptr::copy_nonoverlapping(values.as_ptr() as *const c_void, ptr, real_bytes);

      let _ = gl::UnmapBuffer(gl::ARRAY_BUFFER);
      gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    }

    match warning {
      Some(w) => Err(w),
      None => Ok(())
    }
  }

  /// Fill the `Buffer` with a single value.
  pub fn clear(&self, x: T) where T: Copy {
    let _ = self.write_whole(&self.repr, &vec![x; self.size]);
  }

  /// Fill the whole buffer with an array.
  pub fn fill(&self, values: &[T]) {
    let _ = self.write_whole(&self.repr, values);
  }

  // TODO: see whether we can re-implement the above function with a call to this
  /// Obtain an immutable slice over the range handled by the buffer.
  pub fn get(&mut self) -> Result<BufferSlice<T>, BufferError> {
    let ptr = unsafe {
      gl::BindBuffer(gl::ARRAY_BUFFER, self.handle);
      gl::MapBuffer(gl::ARRAY_BUFFER, gl::READ_ONLY) as *const T
    };

    if ptr.is_null() {
      return Err(BufferError::MapFailed);
    }

    Ok(BufferSlice {
      buf: &mut self,
      ptr: ptr,
    })
  }

  // TODO: see whether we can re-implement the above function with a call to this
  pub fn get_mut(&mut self) -> Result<BufferSliceMut<T>, BufferError> {
    let ptr = unsafe {
      gl::BindBuffer(gl::ARRAY_BUFFER, self.handle);
      gl::MapBuffer(gl::ARRAY_BUFFER, gl::READ_WRITE) as *mut T
    };

    if ptr.is_null() {
      return Err(BufferError::MapFailed);
    }

    Ok(BufferSliceMut {
      buf: &mut self,
      ptr: ptr,
    })
  }
}

impl<T> Drop for Buffer<T> {
  fn drop(&mut self) {
    unsafe { gl::DeleteBuffers(1, &self.handle) }
  }
}

pub struct BufferSlice<'a, T> where T: 'a {
  /// Borrowed buffer.
  buf: &'a mut Buffer<T>,
  /// Raw pointer into the GPU memory.
  ptr: *const T,
}

impl<'a, T> Drop for BufferSlice<'a, T> where T: 'a {
  fn drop(&mut self) {
    unsafe {
      gl::BindBuffer(gl::ARRAY_BUFFER, self.buf.handle);
      gl::UnmapBuffer(gl::ARRAY_BUFFER);
    }
  }
}

impl<'a, T> Deref for BufferSlice<'a, T> where T: 'a {
  type Target = [T];

  fn deref(&self) -> &Self::Target {
    unsafe { slice::from_raw_parts(self.ptr, self.buf.len) }
  }
}

impl<'a, 'b, T> IntoIterator for &'b BufferSlice<'a, T> where T: 'a {
  type Item = &'b T;
  type IntoIter = slice::Iter<'b, T>;

  fn into_iter(self) -> Self::IntoIter {
    self.deref().into_iter()
  }
}

pub struct BufferSliceMut<'a, T> where T: 'a {
  /// Borrowed buffer.
  pub buf: &'a mut Buffer<T>,
  /// Raw pointer into the GPU memory.
  ptr: *mut T,
}

impl<'a, 'b, T> IntoIterator for &'b BufferSliceMut<'a, T> where T: 'a {
  type Item = &'b T;
  type IntoIter = slice::Iter<'b, T>;

  fn into_iter(self) -> Self::IntoIter {
    self.deref().into_iter()
  }
}

impl<'a, 'b, T> IntoIterator for &'b mut BufferSliceMut<'a, T> where T: 'a {
  type Item = &'b mut T;
  type IntoIter = slice::IterMut<'b, T>;

  fn into_iter(self) -> Self::IntoIter {
    self.deref_mut().into_iter()
  }
}

impl<'a, T> Drop for BufferSliceMut<'a, T> where T: 'a {
  fn drop(&mut self) {
    unsafe {
      gl::BindBuffer(gl::ARRAY_BUFFER, self.buf.handle);
      gl::UnmapBuffer(gl::ARRAY_BUFFER);
    }
  }
}

impl<'a, T> Deref for BufferSliceMut<'a, T> where T: 'a {
  type Target = [T];

  fn deref(&self) -> &Self::Target {
    unsafe { slice::from_raw_parts(self.ptr, self.buf.len) }
  }
}

impl<'a, T> DerefMut for BufferSliceMut<'a, T> where T: 'a {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { slice::from_raw_parts_mut(self.ptr, self.buf.len) }
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
pub struct UniformBufferProxy<'a> {
  handle: GLuint
}

impl<'a, T> From<&'a Buffer<T>> for UniformBufferProxy<'a> where T: UniformBlock {
  fn from(buffer: &'a Buffer<T>) -> Self {
    UniformBufferProxy {
      handle: &buffer.handle
    }
  }
}

/// Typeclass of types that can be used inside a uniform block. You have to be extra careful when
/// using uniform blocks and ensure you respect the OpenGL *std140* alignment / size rules. This
/// will be fixed in a coming release.
pub trait UniformBlock {}

impl UniformBlock for u8 {}
impl UniformBlock for u16 {}
impl UniformBlock for u32 {}

impl UniformBlock for i8 {}
impl UniformBlock for i16 {}
impl UniformBlock for i32 {}

impl UniformBlock for f32 {}
impl UniformBlock for f64 {}

impl UniformBlock for bool {}

impl UniformBlock for M22 {}
impl UniformBlock for M33 {}
impl UniformBlock for M44 {}

impl UniformBlock for [u8; 2] {}
impl UniformBlock for [u16; 2] {}
impl UniformBlock for [u32; 2] {}

impl UniformBlock for [i8; 2] {}
impl UniformBlock for [i16; 2] {}
impl UniformBlock for [i32; 2] {}

impl UniformBlock for [f32; 2] {}
impl UniformBlock for [f64; 2] {}

impl UniformBlock for [bool; 2] {}

impl UniformBlock for [u8; 3] {}
impl UniformBlock for [u16; 3] {}
impl UniformBlock for [u32; 3] {}

impl UniformBlock for [i8; 3] {}
impl UniformBlock for [i16; 3] {}
impl UniformBlock for [i32; 3] {}

impl UniformBlock for [f32; 3] {}
impl UniformBlock for [f64; 3] {}

impl UniformBlock for [bool; 3] {}

impl UniformBlock for [u8; 4] {}
impl UniformBlock for [u16; 4] {}
impl UniformBlock for [u32; 4] {}

impl UniformBlock for [i8; 4] {}
impl UniformBlock for [i16; 4] {}
impl UniformBlock for [i32; 4] {}

impl UniformBlock for [f32; 4] {}
impl UniformBlock for [f64; 4] {}

impl UniformBlock for [bool; 4] {}

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
