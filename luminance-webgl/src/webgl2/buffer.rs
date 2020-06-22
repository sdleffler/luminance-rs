//! WebGL2 buffer implementation.

use std::cell::RefCell;
use std::cmp::Ordering;
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::slice;
use web_sys::{WebGl2RenderingContext, WebGlBuffer};

use crate::webgl2::state::{Bind, WebGL2State};
use crate::webgl2::WebGL2;
use luminance::backend::buffer::{Buffer as BufferBackend, BufferSlice as BufferSliceBackend};
use luminance::buffer::BufferError;

/// Wrapped WebGL buffer.
///
/// Used to drop the buffer.
#[derive(Clone, Debug)]
struct BufferWrapper {
  handle: WebGlBuffer,
  state: Rc<RefCell<WebGL2State>>,
}

impl Drop for BufferWrapper {
  fn drop(&mut self) {
    let mut state = self.state.borrow_mut();

    state.unbind_buffer(&self.handle);
    state.ctx.delete_buffer(Some(&self.handle));
  }
}

/// WebGL buffer.
#[derive(Clone, Debug)]
pub struct Buffer<T> {
  /// A cached version of the GPU buffer; emulate persistent mapping.
  pub(crate) buf: Vec<T>,
  gl_buf: BufferWrapper,
}

impl<T> Buffer<T> {
  /// Create a new buffer from a length and a type. This is needed to implement repeat without Default.
  fn new(webgl2: &mut WebGL2, len: usize, clear_value: T) -> Result<Self, BufferError>
  where
    T: Copy,
  {
    let mut state = webgl2.state.borrow_mut();

    let mut buf = Vec::new();
    buf.resize_with(len, || clear_value);

    // generate a buffer and force binding the handle; this prevent side-effects from previous bound
    // resources to prevent binding the buffer
    let handle = state
      .create_buffer()
      .ok_or_else(|| BufferError::CannotCreate)?;
    state.bind_array_buffer(Some(&handle), Bind::Forced);

    let bytes = mem::size_of::<T>() * len;
    state.ctx.buffer_data_with_i32(
      WebGl2RenderingContext::ARRAY_BUFFER,
      bytes as i32,
      WebGl2RenderingContext::STREAM_DRAW,
    );

    let gl_buf = BufferWrapper {
      handle,
      state: webgl2.state.clone(),
    };

    Ok(Buffer { buf, gl_buf })
  }

  pub(crate) fn handle(&self) -> &WebGlBuffer {
    &self.gl_buf.handle
  }

  pub(crate) fn into_raw(self) -> Buffer<u8> {
    let boxed = self.buf.into_boxed_slice();
    let len = boxed.len();
    let ptr = Box::into_raw(boxed) as _;
    let buf = unsafe { Vec::from_raw_parts(ptr, len, len) };

    Buffer {
      buf,
      gl_buf: self.gl_buf,
    }
  }
}

unsafe impl<T> BufferBackend<T> for WebGL2
where
  T: Copy,
{
  type BufferRepr = Buffer<T>;

  unsafe fn new_buffer(&mut self, len: usize) -> Result<Self::BufferRepr, BufferError>
  where
    T: Default,
  {
    Buffer::<T>::new(self, len, T::default())
  }

  unsafe fn len(buffer: &Self::BufferRepr) -> usize {
    buffer.buf.len()
  }

  unsafe fn from_vec(&mut self, vec: Vec<T>) -> Result<Self::BufferRepr, BufferError> {
    let mut state = self.state.borrow_mut();
    let len = vec.len();

    let handle = state
      .create_buffer()
      .ok_or_else(|| BufferError::CannotCreate)?;
    state.bind_array_buffer(Some(&handle), Bind::Forced);

    let bytes = mem::size_of::<T>() * len;
    let data = slice::from_raw_parts(vec.as_ptr() as *const _, bytes);
    state.ctx.buffer_data_with_u8_array(
      WebGl2RenderingContext::ARRAY_BUFFER,
      data,
      WebGl2RenderingContext::STREAM_DRAW,
    );

    let gl_buf = BufferWrapper {
      handle,
      state: self.state.clone(),
    };

    Ok(Buffer { gl_buf, buf: vec })
  }

  unsafe fn repeat(&mut self, len: usize, value: T) -> Result<Self::BufferRepr, BufferError> {
    Buffer::<T>::new(self, len, value)
  }

  unsafe fn at(buffer: &Self::BufferRepr, i: usize) -> Option<T> {
    buffer.buf.get(i).copied()
  }

  unsafe fn whole(buffer: &Self::BufferRepr) -> Vec<T> {
    buffer.buf.iter().copied().collect()
  }

  unsafe fn set(buffer: &mut Self::BufferRepr, i: usize, x: T) -> Result<(), BufferError> {
    let buffer_len = buffer.buf.len();

    if i >= buffer_len {
      Err(BufferError::Overflow {
        index: i,
        buffer_len,
      })
    } else {
      // update the cache first
      buffer.buf[i] = x;

      // then update the WebGL buffer
      let mut state = buffer.gl_buf.state.borrow_mut();
      let bytes = mem::size_of::<T>() * buffer_len;
      update_webgl_buffer(
        &mut state,
        &buffer.gl_buf.handle,
        buffer.buf.as_ptr() as *const u8,
        bytes,
        i,
      );

      Ok(())
    }
  }

  unsafe fn write_whole(buffer: &mut Self::BufferRepr, values: &[T]) -> Result<(), BufferError> {
    let len = values.len();
    let buffer_len = buffer.buf.len();

    // error if we don’t pass the right number of items
    match len.cmp(&buffer_len) {
      Ordering::Less => {
        return Err(BufferError::TooFewValues {
          provided_len: len,
          buffer_len,
        })
      }

      Ordering::Greater => {
        return Err(BufferError::TooManyValues {
          provided_len: len,
          buffer_len,
        })
      }

      _ => (),
    }

    // update the internal representation of the vector; we clear it first then we extend with
    // the input slice to re-use the allocated region
    buffer.buf.clear();
    buffer.buf.extend_from_slice(values);

    // update the data on GPU
    let mut state = buffer.gl_buf.state.borrow_mut();
    let bytes = mem::size_of::<T>() * buffer_len;
    update_webgl_buffer(
      &mut state,
      &buffer.gl_buf.handle,
      buffer.buf.as_ptr() as *const u8,
      bytes,
      0,
    );

    Ok(())
  }

  unsafe fn clear(buffer: &mut Self::BufferRepr, x: T) -> Result<(), BufferError> {
    // copy the value everywhere in the buffer, then simply update the WebGL buffer
    for item in &mut buffer.buf {
      *item = x;
    }

    let mut state = buffer.gl_buf.state.borrow_mut();
    let bytes = buffer.buf.len() * mem::size_of::<T>();
    update_webgl_buffer(
      &mut state,
      &buffer.gl_buf.handle,
      buffer.buf.as_ptr() as *const u8,
      bytes,
      0,
    );

    Ok(())
  }
}

pub struct BufferSlice<T> {
  handle: WebGlBuffer,
  ptr: *const T,
  len: usize,
  state: Rc<RefCell<WebGL2State>>,
}

impl BufferSlice<u8> {
  /// Transmute to another type.
  ///
  /// This method is highly unsafe and should only be used when certain the target type is the
  /// one actually represented by the raw bytes.
  pub(crate) unsafe fn transmute<T>(self) -> BufferSlice<T> {
    let handle = self.handle;
    let ptr = self.len as *const T;
    let len = self.len / mem::size_of::<T>();
    let state = self.state;

    BufferSlice {
      handle,
      ptr,
      len,
      state,
    }
  }
}

impl<T> Deref for BufferSlice<T> {
  type Target = [T];

  fn deref(&self) -> &Self::Target {
    unsafe { slice::from_raw_parts(self.ptr, self.len) }
  }
}

/// Buffer mutable slice wrapper.
///
/// When a buffer is mapped, we are the only owner of it. We can then read or write from/to the
/// mapped buffer, and then update the GPU buffer on the [`Drop`] implementation.
pub struct BufferSliceMutWrapper {
  handle: WebGlBuffer,
  ptr: *mut u8,
  bytes: usize,
  state: Rc<RefCell<WebGL2State>>,
}

impl Drop for BufferSliceMutWrapper {
  fn drop(&mut self) {
    let mut state = self.state.borrow_mut();
    update_webgl_buffer(&mut state, &self.handle, self.ptr, self.bytes, 0);
  }
}

pub struct BufferSliceMut<T> {
  raw: BufferSliceMutWrapper,
  _phantom: PhantomData<T>,
}

impl BufferSliceMut<u8> {
  /// Transmute to another type.
  ///
  /// This method is highly unsafe and should only be used when certain the target type is the
  /// one actually represented by the raw bytes.
  pub(crate) unsafe fn transmute<T>(self) -> BufferSliceMut<T> {
    BufferSliceMut {
      raw: self.raw,
      _phantom: PhantomData,
    }
  }
}

impl<T> Deref for BufferSliceMut<T> {
  type Target = [T];

  fn deref(&self) -> &Self::Target {
    unsafe {
      slice::from_raw_parts(
        self.raw.ptr as *const T,
        self.raw.bytes * mem::size_of::<T>(),
      )
    }
  }
}

impl<T> DerefMut for BufferSliceMut<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe {
      slice::from_raw_parts_mut(self.raw.ptr as *mut T, self.raw.bytes * mem::size_of::<T>())
    }
  }
}

unsafe impl<T> BufferSliceBackend<T> for WebGL2
where
  T: Copy,
{
  type SliceRepr = BufferSlice<T>;

  type SliceMutRepr = BufferSliceMut<T>;

  unsafe fn slice_buffer(buffer: &Self::BufferRepr) -> Result<Self::SliceRepr, BufferError> {
    let slice = BufferSlice {
      handle: buffer.gl_buf.handle.clone(),
      ptr: buffer.buf.as_ptr(),
      len: buffer.buf.len(),
      state: buffer.gl_buf.state.clone(),
    };

    Ok(slice)
  }

  unsafe fn slice_buffer_mut(
    buffer: &mut Self::BufferRepr,
  ) -> Result<Self::SliceMutRepr, BufferError> {
    let raw = BufferSliceMutWrapper {
      handle: buffer.gl_buf.handle.clone(),
      ptr: buffer.buf.as_mut_ptr() as *mut u8,
      bytes: buffer.buf.len() / mem::size_of::<T>(),
      state: buffer.gl_buf.state.clone(),
    };
    let slice = BufferSliceMut {
      raw,
      _phantom: PhantomData,
    };

    Ok(slice)
  }

  unsafe fn obtain_slice(slice: &Self::SliceRepr) -> Result<&[T], BufferError> {
    Ok(slice::from_raw_parts(slice.ptr, slice.len))
  }

  unsafe fn obtain_slice_mut(slice: &mut Self::SliceMutRepr) -> Result<&mut [T], BufferError> {
    Ok(slice::from_raw_parts_mut(
      slice.raw.ptr as *mut T,
      slice.raw.bytes * mem::size_of::<T>(),
    ))
  }
}

/// Update a WebGL buffer by copying an input slice.
fn update_webgl_buffer(
  state: &mut WebGL2State,
  gl_buf: &WebGlBuffer,
  data: *const u8,
  bytes: usize,
  offset: usize,
) {
  state.bind_array_buffer(Some(gl_buf), Bind::Cached);

  let data = unsafe { slice::from_raw_parts(data as _, bytes) };
  state
    .ctx
    .buffer_sub_data_with_i32_and_u8_array_and_src_offset(
      WebGl2RenderingContext::ARRAY_BUFFER,
      offset as _,
      data,
      0,
    );
}
