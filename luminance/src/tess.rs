//! Tessellation features.
//!
//! # Tessellation mode
//!
//! Tessellation is geometric information. Currently, several kinds of tessellation are supported:
//!
//! - *point clouds*;
//! - *lines*;
//! - *line strips*;
//! - *triangles*;
//! - *triangle fans*;
//! - *triangle strips*.
//!
//! Those kinds of tessellation are designated by the `Mode` type.
//!
//! # Tessellation creation
//!
//! Creation is done via the [`Tess::new`] function. This function is polymorphing in the type of
//! vertices you send. See the [`TessVertices`] type for further details.
//!
//! ## On interleaved and deinterleaved vertices
//!
//! Because [`Tess::new`] uses your user-defined vertex type, it uses interleaved memory. That
//! means that all vertices are spread out in a single GPU memory region (a single buffer). This
//! behavior is fine for most applications that will want their shaders to use all vertex attributes
//! but sometimes you want a more specific memory strategy. For instance, some shaders won’t use all
//! of the available vertex attributes.
//!
//! [`Tess`] supports such situations with the [`Tess::new_deinterleaved`] method, that creates a
//! tessellation by lying vertex attributes out in their own respective buffers. The implication is
//! that the interface requires you to pass already deinterleaved vertices. Those are most of the
//! time isomorphic to tuples of slices.
//!
//! # Tessellation vertices CPU mapping
//!
//! It’s possible to map `Tess`’ vertices into your code. You’re provided with two types to do so:
//!
//! - [`BufferSlice`], which gives you an immutable access to the vertices.
//! - [`BufferSliceMut`], which gives you a mutable access to the vertices.
//!
//! You can retrieve those slices with the [`Tess::as_slice`] and [`Tess::as_slice_mut`] methods.
//!
//! # Tessellation render
//!
//! In order to render a [`Tess`], you have to use a [`TessSlice`] object. You’ll be able to use
//! that object in *pipelines*. See the `pipeline` module for further details.
//!
//! [`BufferSlice`]: crate/buffer/struct.BufferSlice.html
//! [`BufferSliceMut`]: crate/buffer/struct.BufferSliceMut.html
//! [`Tess`]: struct.Tess.html
//! [`Tess::as_slice`]: struct.Tess.html#method.as_slice
//! [`Tess::as_slice_mut`]: struct.Tess.html#method.as_slice_mut
//! [`Tess::new`]: struct.Tess.html#method.new
//! [`Tess::new_deinterleaved`]: struct.Tess.html#method.new_deinterleaved
//! [`TessSlice`]: struct.TessSlice.html

#[cfg(feature = "std")]
use std::fmt;
#[cfg(feature = "std")]
use std::mem::size_of;
#[cfg(feature = "std")]
use std::ops::{Range, RangeFrom, RangeFull, RangeTo};
#[cfg(feature = "std")]
use std::os::raw::c_void;
#[cfg(feature = "std")]
use std::ptr;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(not(feature = "std"))]
use core::fmt;
#[cfg(not(feature = "std"))]
use core::mem::size_of;
#[cfg(not(feature = "std"))]
use core::ops::{Range, RangeFrom, RangeFull, RangeTo};
#[cfg(not(feature = "std"))]
use core::ptr;

use buffer::{Buffer, BufferError, BufferSlice, BufferSliceMut, RawBuffer};
use context::GraphicsContext;
use metagl::*;
use vertex::{IndexedVertexAttribFmt, Vertex, VertexAttribDim, VertexAttribFmt, VertexAttribType, VertexFmt};

/// Vertices can be connected via several modes.
#[derive(Copy, Clone, Debug)]
pub enum Mode {
  /// A single point.
  Point,
  /// A line, defined by two points.
  Line,
  /// A strip line, defined by at least two points and zero or many other ones.
  LineStrip,
  /// A triangle, defined by three points.
  Triangle,
  /// A triangle fan, defined by at least three points and zero or many other ones.
  TriangleFan,
  /// A triangle strip, defined by at least three points and zero or many other ones.
  TriangleStrip,
}

/// Error that can occur while trying to map GPU tessellation to host code.
#[derive(Debug, Eq, PartialEq)]
pub enum TessMapError {
  /// The CPU mapping failed due to buffer errors.
  VertexBufferMapFailed(BufferError),
  /// Target type is not the same as the one stored in the buffer.
  TypeMismatch(VertexFmt, VertexFmt),
  /// The CPU mapping failed because you cannot map an attributeless tessellation since it doesn’t
  /// have any vertex attribute.
  ForbiddenAttributelessMapping,
  /// The CPU mapping failed because currently, mapping deinterleaved buffers is not supported via
  /// a single slice.
  ForbiddenDeinterleavedMapping,
}

impl fmt::Display for TessMapError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      TessMapError::VertexBufferMapFailed(ref e) => write!(f, "cannot map tessellation buffer: {}", e),
      TessMapError::TypeMismatch(ref a, ref b) =>
       write!(f, "cannot map tessellation: type mismatch between {:?} and {:?}", a, b),
      TessMapError::ForbiddenAttributelessMapping => f.write_str("cannot map an attributeless buffer"),
      TessMapError::ForbiddenDeinterleavedMapping => {
        f.write_str("cannot map a deinterleaved buffer as interleaved")
      }
    }
  }
}

struct VertexBuffer {
  /// Indexed format of the buffer.
  fmt: VertexFmt,
  /// Internal buffer.
  buf: RawBuffer,
}

/// Build tessellations the easy way.
pub struct TessBuilder<'a, C> {
  ctx: &'a mut C,
  vertex_buffers: Vec<VertexBuffer>,
  index_buffer: Option<RawBuffer>,
  mode: Mode,
  vert_nb: usize
}

impl<'a, C> TessBuilder<'a, C> {
  pub fn new(ctx: &'a mut C) -> Self {
    TessBuilder {
      ctx,
      vertex_buffers: Vec::new(),
      index_buffer: None,
      mode: Mode::Point,
      vert_nb: 0
    }
  }
}

impl<'a, C> TessBuilder<'a, C>
where
  C: GraphicsContext,
{
  /// Add vertices to be part of the tessellation.
  ///
  /// This method can be used in several ways. First, you can decide to use interleaved memory, in
  /// which case you will call this method only once by providing an interleaved slice / borrowed
  /// buffer. Second, you can opt-in to use deinterleaved memory, in which case you will have
  /// several, smaller buffers of borrowed data and you will issue a call to this method for all of
  /// them.
  pub fn add_vertices<V, W>(mut self, vertices: W) -> Self
  where
    W: AsRef<[V]>,
    V: Vertex<'a>
  {
    let vertices = vertices.as_ref();

    // create a new interleaved raw buffer and turn it into a vertex buffer
    let vb = VertexBuffer {
      fmt: V::vertex_fmt(),
      buf: Buffer::from_slice(self.ctx, vertices).to_raw(),
    };

    self.vertex_buffers.push(vb);

    self
  }

  /// Set vertex indices in order to specify how vertices should be picked by the GPU pipeline.
  pub fn set_indices<T>(mut self, indices: T) -> Self
  where
    T: AsRef<[u32]>,
  {
    let indices = indices.as_ref();

    // create a new raw buffer containing the indices and turn it into a vertex buffer
    let buf = Buffer::from_slice(self.ctx, indices).to_raw();

    self.index_buffer = Some(buf);

    self
  }

  pub fn set_mode(mut self, mode: Mode) -> Self {
    self.mode = mode;
    self
  }

  pub fn set_vertex_nb(mut self, nb: usize) -> Self {
    self.vert_nb = nb;
    self
  }

  pub fn build(self) -> Result<Tess, TessError> {
    // try to deduce the number of vertices to render if it’s not specified
    let vert_nb = self.guess_vert_nb_or_fail()?;
    self.build_tess(vert_nb)
  }

  /// Build a tessellation based on a given number of vertices to render by default.
  fn build_tess(self, vert_nb: usize) -> Result<Tess, TessError> {
    let mut vao: GLuint = 0;

    unsafe {
      let mut gfx_st = self.ctx.state().borrow_mut();

      gl::GenVertexArrays(1, &mut vao);

      gfx_st.bind_vertex_array(vao);

      // add the vertex buffers into the vao
      for vb in &self.vertex_buffers {
        gfx_st.bind_array_buffer(vb.buf.handle());
        set_vertex_pointers(&vb.fmt)
      }

      // in case of indexed render, create an index buffer
      if let Some(ref index_buffer) = self.index_buffer {
        gfx_st.bind_element_array_buffer(index_buffer.handle());
      }

      // convert to OpenGL-friendly internals and return
      Ok(Tess {
        mode: opengl_mode(self.mode),
        vert_nb,
        vao,
        vertex_buffers: self.vertex_buffers,
        index_buffer: self.index_buffer
      })
    }
  }

  /// Guess how many vertices there is to render based on the current configuration or fail if
  /// incorrectly configured.
  fn guess_vert_nb_or_fail(&self) -> Result<usize, TessError> {
    if self.vert_nb == 0 {
      // we don’t have an explicit vertex number to render; go and guess!
      if let Some(ref index_buffer) = self.index_buffer {
        // we have an index buffer: just use its size
        Ok(index_buffer.len())
      } else {
        // deduce the number of vertices based on the vertex buffers; they all
        // must be of the same length, otherwise it’s an error
        match self.vertex_buffers.len() {
          0 => {
            Err(TessError::AttributelessError("attributeless render with no vertex number".to_owned()))
          }

          1 => {
            Ok(self.vertex_buffers[0].buf.len())
          }

          _ => {
            let vert_nb = self.vertex_buffers[0].buf.len();
            let incoherent = self.check_incoherent_vertex_buffers(vert_nb);

            if incoherent {
              Err(TessError::LengthIncoherency(vert_nb))
            } else {
              Ok(vert_nb)
            }
          }
        }
      }
    } else {
      // we have an explicit number of vertices to render, but we’re gonna check that number actually
      // makes sense
      if let Some(ref index_buffer) = self.index_buffer {
        // we have indices (indirect draw); so we’ll compare to them
        if index_buffer.len() < self.vert_nb {
          return Err(TessError::Overflow(index_buffer.len(), self.vert_nb));
        }
      } else {
        let incoherent = self.check_incoherent_vertex_buffers(self.vert_nb);

        if incoherent {
          return Err(TessError::LengthIncoherency(self.vert_nb));
        } else if !self.vertex_buffers.is_empty() && self.vertex_buffers[0].buf.len() < self.vert_nb {
          return Err(TessError::Overflow(self.vertex_buffers[0].buf.len(), self.vert_nb));
        }
      }

      Ok(self.vert_nb)
    }
  }

  /// Check whether any vertex buffer is incoherent in its length according to the input length.
  fn check_incoherent_vertex_buffers(&self, len: usize) -> bool {
    !self.vertex_buffers.iter().all(|vb| vb.buf.len() == len)
  }
}

#[derive(Debug)]
pub enum TessError {
  AttributelessError(String),
  LengthIncoherency(usize),
  Overflow(usize, usize)
}

pub struct Tess {
  mode: GLenum,
  vert_nb: usize,
  vao: GLenum,
  vertex_buffers: Vec<VertexBuffer>,
  index_buffer: Option<RawBuffer>,
}

impl Tess {
  fn render<C>(
    &self,
    ctx: &mut C,
    start_index: usize,
    vert_nb: usize,
    inst_nb: usize
  )
  where C: GraphicsContext {
    let vert_nb = vert_nb as GLsizei;
    let inst_nb = inst_nb as GLsizei;

    unsafe {
      ctx.state().borrow_mut().bind_vertex_array(self.vao);

      if self.index_buffer.is_some() {
        // indexed render
        let first = (size_of::<u32>() * start_index) as *const c_void;

        if inst_nb <= 1 {
          gl::DrawElements(self.mode, vert_nb, gl::UNSIGNED_INT, first);
        } else {
          gl::DrawElementsInstanced(self.mode, vert_nb, gl::UNSIGNED_INT, first, inst_nb);
        }
      } else {
        // direct render
        let first = start_index as GLint;

        if inst_nb <= 1 {
          gl::DrawArrays(self.mode, first, vert_nb);
        } else {
          gl::DrawArraysInstanced(self.mode, first, vert_nb, inst_nb);
        }
      }
    }
  }

  pub fn as_slice<'a, V>(&'a self) -> Result<BufferSlice<V>, TessMapError> where V: Vertex<'a> {
    match self.vertex_buffers.len() {
      0 => Err(TessMapError::ForbiddenAttributelessMapping),

      1 => {
        let vb = &self.vertex_buffers[0];
        let target_fmt = V::vertex_fmt(); // costs a bit

        if vb.fmt != target_fmt {
          Err(TessMapError::TypeMismatch(vb.fmt.clone(), target_fmt))
        } else {
          vb.buf.as_slice().map_err(TessMapError::VertexBufferMapFailed)
        }
      }

      _ => Err(TessMapError::ForbiddenDeinterleavedMapping),
    }
  }

  pub fn as_slice_mut<'a, V>(&mut self) -> Result<BufferSliceMut<V>, TessMapError> where V: Vertex<'a> {
    match self.vertex_buffers.len() {
      0 => Err(TessMapError::ForbiddenAttributelessMapping),

      1 => {
        let vb = &mut self.vertex_buffers[0];
        let target_fmt = V::vertex_fmt(); // costs a bit

        if vb.fmt != target_fmt {
          Err(TessMapError::TypeMismatch(vb.fmt.clone(), target_fmt))
        } else {
          vb.buf.as_slice_mut().map_err(TessMapError::VertexBufferMapFailed)
        }
      }

      _ => Err(TessMapError::ForbiddenDeinterleavedMapping),
    }
  }
}

impl Drop for Tess {
  fn drop(&mut self) {
    unsafe {
      gl::DeleteVertexArrays(1, &self.vao);
    }
  }
}

// Give OpenGL types information on the content of the VBO by setting vertex formats and pointers
// to buffer memory.
fn set_vertex_pointers(formats: &VertexFmt) {
  // this function sets the vertex attribute pointer for the input list by computing:
  //   - The vertex attribute ID: this is the “rank” of the attribute in the input list (order
  //     matters, for short).
  //   - The stride: this is easily computed, since it’s the size (bytes) of a single vertex.
  //   - The offsets: each attribute has a given offset in the buffer. This is computed by
  //     accumulating the size of all previously set attributes.
  let offsets = aligned_offsets(formats);
  let vertex_weight = offset_based_vertex_weight(formats, &offsets) as GLsizei;

  for (format, off) in formats.iter().zip(offsets) {
    set_component_format(vertex_weight, off, format);
  }
}

// Compute offsets for all the vertex components according to the alignments provided.
fn aligned_offsets(formats: &VertexFmt) -> Vec<usize> {
  let mut offsets = Vec::with_capacity(formats.len());
  let mut off = 0;

  // compute offsets
  for f in formats {
    let fmt = &f.attrib_fmt;
    off = off_align(off, fmt.align); // keep the current component format aligned
    offsets.push(off);
    off += component_weight(fmt); // increment the offset by the pratical size of the component
  }

  offsets
}

// Align an offset.
#[inline]
fn off_align(off: usize, align: usize) -> usize {
  let a = align - 1;
  (off + a) & !a
}

// Weight in bytes of a vertex component.
fn component_weight(f: &VertexAttribFmt) -> usize {
  dim_as_size(&f.dim) as usize * f.unit_size
}

fn dim_as_size(d: &VertexAttribDim) -> GLint {
  match *d {
    VertexAttribDim::Dim1 => 1,
    VertexAttribDim::Dim2 => 2,
    VertexAttribDim::Dim3 => 3,
    VertexAttribDim::Dim4 => 4,
  }
}

// Weight in bytes of a single vertex, taking into account padding so that the vertex stay correctly
// aligned.
fn offset_based_vertex_weight(formats: &VertexFmt, offsets: &[usize]) -> usize {
  if formats.is_empty() || offsets.is_empty() {
    return 0;
  }

  off_align(
    offsets[offsets.len() - 1] + component_weight(&formats[formats.len() - 1].attrib_fmt),
    formats[0].attrib_fmt.align,
  )
}

// Set the vertex component OpenGL pointers regarding the index of the component (i), the stride
fn set_component_format(stride: GLsizei, off: usize, fmt: &IndexedVertexAttribFmt) {
  let f = &fmt.attrib_fmt;
  let index = fmt.index as GLuint;

  unsafe {
    match f.comp_type {
      VertexAttribType::Floating => {
        gl::VertexAttribPointer(
          index,
          dim_as_size(&f.dim),
          opengl_sized_type(&f),
          gl::FALSE,
          stride,
          ptr::null::<c_void>().offset(off as isize),
          );
      },
      VertexAttribType::Integral | VertexAttribType::Unsigned | VertexAttribType::Boolean => {
        gl::VertexAttribIPointer(
          index,
          dim_as_size(&f.dim),
          opengl_sized_type(&f),
          stride,
          ptr::null::<c_void>().offset(off as isize),
          );
      },
    }

    gl::EnableVertexAttribArray(index);
  }
}

fn opengl_sized_type(f: &VertexAttribFmt) -> GLenum {
  match (f.comp_type, f.unit_size) {
    (VertexAttribType::Integral, 1) => gl::BYTE,
    (VertexAttribType::Integral, 2) => gl::SHORT,
    (VertexAttribType::Integral, 4) => gl::INT,
    (VertexAttribType::Unsigned, 1) | (VertexAttribType::Boolean, 1) => gl::UNSIGNED_BYTE,
    (VertexAttribType::Unsigned, 2) => gl::UNSIGNED_SHORT,
    (VertexAttribType::Unsigned, 4) => gl::UNSIGNED_INT,
    (VertexAttribType::Floating, 4) => gl::FLOAT,
    _ => panic!("unsupported vertex component format: {:?}", f),
  }
}

fn opengl_mode(mode: Mode) -> GLenum {
  match mode {
    Mode::Point => gl::POINTS,
    Mode::Line => gl::LINES,
    Mode::LineStrip => gl::LINE_STRIP,
    Mode::Triangle => gl::TRIANGLES,
    Mode::TriangleFan => gl::TRIANGLE_FAN,
    Mode::TriangleStrip => gl::TRIANGLE_STRIP,
  }
}

/// Tessellation slice.
///
/// This type enables slicing a tessellation on the fly so that we can render patches of it.
#[derive(Clone)]
pub struct TessSlice<'a> {
  /// Tessellation to render.
  tess: &'a Tess,
  /// Start index (vertex) in the tessellation.
  start_index: usize,
  /// Number of vertices to pick from the tessellation. If `None`, all of them are selected.
  vert_nb: usize,
  /// Number of instances to render.
  inst_nb: usize,
}

impl<'a> TessSlice<'a> {
  /// Create a tessellation render that will render the whole input tessellation with only one
  /// instance.
  pub fn one_whole(tess: &'a Tess) -> Self {
    TessSlice {
      tess,
      start_index: 0,
      vert_nb: tess.vert_nb,
      inst_nb: 1,
    }
  }

  /// Create a tessellation render for a part of the tessellation starting at the beginning of its
  /// buffer with only one instance.
  ///
  /// The part is selected by giving the number of vertices to render.
  ///
  /// > Note: if you also need to use an arbitrary part of your tessellation (not starting at the
  /// > first vertex in its buffer), have a look at `TessSlice::one_slice`.
  ///
  /// # Panic
  ///
  /// Panic if the number of vertices is higher to the capacity of the tessellation’s vertex buffer.
  pub fn one_sub(tess: &'a Tess, vert_nb: usize) -> Self {
    if vert_nb > tess.vert_nb {
      panic!(
        "cannot render {} vertices for a tessellation which vertex capacity is {}",
        vert_nb, tess.vert_nb
      );
    }

    TessSlice {
      tess,
      start_index: 0,
      vert_nb,
      inst_nb: 1,
    }
  }

  /// Create a tessellation render for a slice of the tessellation starting anywhere in its buffer
  /// with only one instance.
  ///
  /// The part is selected by giving the start vertex and the number of vertices to render. This
  ///
  /// # Panic
  ///
  /// Panic if the start vertex is higher to the capacity of the tessellation’s vertex buffer.
  ///
  /// Panic if the number of vertices is higher to the capacity of the tessellation’s vertex buffer.
  pub fn one_slice(tess: &'a Tess, start: usize, nb: usize) -> Self {
    if start > tess.vert_nb {
      panic!(
        "cannot render {} vertices starting at vertex {} for a tessellation which vertex capacity is {}",
        nb, start, tess.vert_nb
      );
    }

    if nb > tess.vert_nb {
      panic!(
        "cannot render {} vertices for a tessellation which vertex capacity is {}",
        nb, tess.vert_nb
      );
    }

    TessSlice {
      tess,
      start_index: start,
      vert_nb: nb,
      inst_nb: 1,
    }
  }

  /// Render a tessellation.
  pub fn render<C>(&self, ctx: &mut C) where C: GraphicsContext {
    self
      .tess
      .render(ctx, self.start_index, self.vert_nb, self.inst_nb);
  }
}

impl<'a> From<&'a Tess> for TessSlice<'a> {
  fn from(tess: &'a Tess) -> Self {
    TessSlice::one_whole(tess)
  }
}

pub trait TessSliceIndex<Idx> {
  fn slice(&self, idx: Idx) -> TessSlice;
}

impl TessSliceIndex<RangeFull> for Tess {
  fn slice<'a>(&self, _: RangeFull) -> TessSlice {
    TessSlice::one_whole(self)
  }
}

impl TessSliceIndex<RangeTo<usize>> for Tess {
  fn slice(&self, to: RangeTo<usize>) -> TessSlice {
    TessSlice::one_sub(self, to.end)
  }
}

impl TessSliceIndex<RangeFrom<usize>> for Tess {
  fn slice(&self, from: RangeFrom<usize>) -> TessSlice {
    TessSlice::one_slice(self, from.start, self.vert_nb)
  }
}

impl TessSliceIndex<Range<usize>> for Tess {
  fn slice(&self, range: Range<usize>) -> TessSlice {
    TessSlice::one_slice(self, range.start, range.end)
  }
}
