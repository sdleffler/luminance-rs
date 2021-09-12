//! WebGL2 buffer implementation.

use crate::webgl2::{
  state::{Bind, WebGL2State},
  WebGL2,
};
use core::fmt;
use luminance::tess::TessError;
use std::{
  cell::RefCell,
  error,
  marker::PhantomData,
  mem,
  ops::{Deref, DerefMut},
  rc::Rc,
  slice,
};
use web_sys::{WebGl2RenderingContext, WebGlBuffer};

/// Errors that can occur when dealing with buffers.
#[derive(Clone, Debug)]
pub enum BufferError {
  /// Cannot create the buffer on the backend.
  CannotCreate,
}

impl fmt::Display for BufferError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match self {
      BufferError::CannotCreate => f.write_str("cannot create buffer on the backend"),
    }
  }
}

impl error::Error for BufferError {}

impl From<BufferError> for TessError {
  fn from(e: BufferError) -> Self {
    TessError::cannot_create(e.to_string())
  }
}

/// Wrapped WebGL buffer.
///
/// Used to drop the buffer.
#[derive(Clone, Debug)]
struct BufferWrapper {
  handle: WebGlBuffer,
  /// Target the buffer was created with; WebGL2 doesn’t play well with rebinding a buffer to a different target (unlike
  /// OpenGL, in which that optimization is handy).
  ///
  /// See https://developer.mozilla.org/en-US/docs/Web/API/WebGLRenderingContext/bindBuffer#exceptions for further details
  target: u32,
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
  pub(crate) fn from_vec(
    webgl2: &mut WebGL2,
    vec: Vec<T>,
    target: u32,
  ) -> Result<Self, BufferError> {
    let mut state = webgl2.state.borrow_mut();
    let len = vec.len();

    let handle = state
      .create_buffer()
      .ok_or_else(|| BufferError::CannotCreate)?;

    bind_buffer(&mut state, &handle, target, Bind::Forced)?;

    let bytes = mem::size_of::<T>() * len;
    let data = unsafe { slice::from_raw_parts(vec.as_ptr() as *const _, bytes) };
    state
      .ctx
      .buffer_data_with_u8_array(target, data, WebGl2RenderingContext::STREAM_DRAW);

    let gl_buf = BufferWrapper {
      handle,
      target,
      state: webgl2.state.clone(),
    };

    Ok(Buffer { gl_buf, buf: vec })
  }

  pub(crate) fn handle(&self) -> &WebGlBuffer {
    &self.gl_buf.handle
  }

  pub(crate) fn slice_buffer(&self) -> BufferSlice<T> {
    BufferSlice {
      handle: self.gl_buf.handle.clone(),
      ptr: self.buf.as_ptr(),
      len: self.buf.len(),
      state: self.gl_buf.state.clone(),
    }
  }

  pub(crate) fn slice_buffer_mut(&mut self) -> BufferSliceMut<T> {
    let raw = BufferSliceMutWrapper {
      target: self.gl_buf.target,
      handle: self.gl_buf.handle.clone(),
      ptr: self.buf.as_mut_ptr() as *mut u8,
      bytes: self.buf.len() * mem::size_of::<T>(),
      state: self.gl_buf.state.clone(),
    };

    BufferSliceMut {
      raw,
      _phantom: PhantomData,
    }
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
    let ptr = self.ptr as *const T;
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
  target: u32,
  handle: WebGlBuffer,
  ptr: *mut u8,
  bytes: usize,
  state: Rc<RefCell<WebGL2State>>,
}

impl Drop for BufferSliceMutWrapper {
  fn drop(&mut self) {
    let mut state = self.state.borrow_mut();
    let _ = update_webgl_buffer(
      self.target,
      &mut state,
      &self.handle,
      self.ptr,
      self.bytes,
      0,
    );
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
        self.raw.bytes / mem::size_of::<T>(),
      )
    }
  }
}

impl<T> DerefMut for BufferSliceMut<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe {
      slice::from_raw_parts_mut(self.raw.ptr as *mut T, self.raw.bytes / mem::size_of::<T>())
    }
  }
}

/// Bind a buffer to a given state regarding the input target.
fn bind_buffer(
  state: &mut WebGL2State,
  handle: &WebGlBuffer,
  target: u32,
  bind_mode: Bind,
) -> Result<(), BufferError> {
  // depending on the buffer target, we are not going to bind it the same way, as the first bind
  // is actually meaningful in WebGL2
  match target {
    WebGl2RenderingContext::ARRAY_BUFFER => state.bind_array_buffer(Some(handle), bind_mode),
    WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER => {
      state.bind_element_array_buffer(Some(handle), bind_mode)
    }

    // a bit opaque but should never happen
    _ => return Err(BufferError::CannotCreate),
  }

  Ok(())
}

/// Update a WebGL buffer by copying an input slice.
fn update_webgl_buffer(
  target: u32,
  state: &mut WebGL2State,
  handle: &WebGlBuffer,
  data: *const u8,
  bytes: usize,
  offset: usize,
) -> Result<(), BufferError> {
  bind_buffer(state, handle, target, Bind::Cached)?;

  let data = unsafe { slice::from_raw_parts(data as _, bytes) };
  state
    .ctx
    .buffer_sub_data_with_i32_and_u8_array_and_src_offset(target, offset as _, data, 0);

  Ok(())
}
