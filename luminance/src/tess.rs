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
use std::marker::PhantomData;
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
use core::marker::PhantomData;
#[cfg(not(feature = "std"))]
use core::mem::size_of;
#[cfg(not(feature = "std"))]
use core::ops::{Range, RangeFrom, RangeFull, RangeTo};
#[cfg(not(feature = "std"))]
use core::ptr;

use buffer::{Buffer, BufferError, BufferSlice, BufferSliceMut, RawBuffer};
use context::GraphicsContext;
use deinterleave::{Deinterleave, SliceVisitor};
use metagl::*;
use vertex::{Vertex, VertexAttributeDim, VertexAttributeFmt, VertexAttributeType};

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
      TessMapError::VertexBufferMapFailed(ref e) => write!(f, "cannot map tessallation buffer: {}", e),
      TessMapError::ForbiddenAttributelessMapping => f.write_str("cannot map an attributeless buffer"),
      TessMapError::ForbiddenDeinterleavedMapping => {
        f.write_str("cannot map a deinterleaved buffer as interleaved")
      }
    }
  }
}

/// Accepted vertices for building tessellations.
///
/// This type enables you to pass in a slice of vertices or ask for the GPU to only reserve enough
/// space for the number of vertices, leaving the allocated memory uninitialized.
#[derive(Debug, Eq, PartialEq)]
pub enum TessVertices<'a, T>
where
  T: 'a + ?Sized,
{
  /// Pass in a borrow of vertices.
  Fill(&'a T),
  /// Reserve a certain number of vertices.
  Reserve(usize),
}

impl<'a, T> TessVertices<'a, [T]> {
  fn len(&self) -> usize {
    match *self {
      TessVertices::Fill(ref slice) => slice.len(),
      TessVertices::Reserve(len) => len,
    }
  }
}

impl<'a, T> From<&'a [T]> for TessVertices<'a, [T]>
where
  T: 'a,
{
  fn from(slice: &'a [T]) -> Self {
    TessVertices::Fill(slice)
  }
}

/// GPU typed tessellation.
///
/// The tessellation is typed with the vertex type.
pub struct Tess<V> {
  mode: GLenum,
  vert_nb: usize,
  vao: GLenum,
  vbo: Vec<RawBuffer>, // no vbo means attributeless render
  ibo: Option<RawBuffer>,
  _v: PhantomData<V>,
}

impl<V> Tess<V> {
  /// Create a new tessellation.
  ///
  /// The `mode` argument gives the type of the primitives and how to interpret the `vertices` and
  /// `indices` slices. If `indices` is set to `None`, the tessellation will use the `vertices`
  /// as-is.
  ///
  /// This is the interleaved version. If you want deinterleaved tessellations, have a look at the
  /// [`Tess::new_deinterleaved`] method.
  pub fn new<'a, C, W, I>(ctx: &mut C, mode: Mode, vertices: W, indices: I) -> Self
  where
    C: GraphicsContext,
    TessVertices<'a, [V]>: From<W>,
    V: 'a + Vertex<'a>,
    I: Into<Option<&'a [u32]>>,
  {
    let vertices = TessVertices::from(vertices);

    let mut vao: GLuint = 0;
    let vert_nb = vertices.len();

    unsafe {
      gl::GenVertexArrays(1, &mut vao);

      ctx.state().borrow_mut().bind_vertex_array(vao);

      // vertex buffer
      let vertex_buffer = Buffer::new(ctx, vert_nb);

      // fill the buffer with vertices only if asked by the user
      if let TessVertices::Fill(verts) = vertices {
        vertex_buffer.fill(verts).unwrap();
      }

      let raw_vbo = vertex_buffer.to_raw();

      ctx.state().borrow_mut().bind_array_buffer(raw_vbo.handle()); // FIXME: issue the call whatever the caching result
      set_vertex_pointers_interleaved(V::VERTEX_FMT);

      // in case of indexed render, create an index buffer
      if let Some(indices) = indices.into() {
        let index_buffer = Buffer::from_slice(ctx, indices);
        let raw_ibo = index_buffer.to_raw();

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, raw_ibo.handle());

        // TODO: ensure we don’t need that
        // ctx.state().borrow_mut().bind_vertex_array(vao);

        Tess {
          mode: opengl_mode(mode),
          vert_nb: indices.len(),
          vao,
          vbo: vec![raw_vbo],
          ibo: Some(raw_ibo),
          _v: PhantomData,
        }
      } else {
        // TODO: ensure we don’t need that
        // ctx.state().borrow_mut().bind_vertex_array(vao);

        Tess {
          mode: opengl_mode(mode),
          vert_nb,
          vao,
          vbo: vec![raw_vbo],
          ibo: None,
          _v: PhantomData,
        }
      }
    }
  }

  /// Build a tessellation by using deinterleaved memory.
  pub fn new_deinterleaved<'a, C, W, I>(ctx: &mut C, mode: Mode, vertices: W, indices: I) -> Self
  where
    C: GraphicsContext,
    &'a V::Deinterleaved: From<W>,
    V: 'a + Vertex<'a>,
    I: Into<Option<&'a [u32]>>,
  {
    let mut vao: GLuint = 0;

    unsafe {
      gl::GenVertexArrays(1, &mut vao);
      ctx.state().borrow_mut().bind_vertex_array(vao);

      let deinterleaved: &V::Deinterleaved = vertices.into();
      let (buffers, vert_nb) = {
        let mut visitor = DeinterleavingVisitor::new(ctx, V::VERTEX_FMT.iter().cloned());

        deinterleaved.visit_deinterleave(&mut visitor);
        (visitor.buffers, visitor.vert_nb)
      };

      if let Some(indices) = indices.into() {
        let index_buffer = Buffer::from_slice(ctx, indices);
        let raw_ibo = index_buffer.to_raw();

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, raw_ibo.handle());

        Tess {
          mode: opengl_mode(mode),
          vert_nb: indices.len(),
          vao,
          vbo: buffers,
          ibo: Some(raw_ibo),
          _v: PhantomData,
        }
      } else {
        Tess {
          mode: opengl_mode(mode),
          vert_nb,
          vao,
          vbo: buffers,
          ibo: None,
          _v: PhantomData,
        }
      }
    }
  }

  // Render the tessellation by providing the number of vertices to pick from it and how many
  // instances are wished.
  fn render<C>(&self, ctx: &mut C, start_index: usize, vert_nb: usize, inst_nb: usize)
  where
    C: GraphicsContext,
  {
    let vert_nb = vert_nb as GLsizei;
    let inst_nb = inst_nb as GLsizei;

    unsafe {
      ctx.state().borrow_mut().bind_vertex_array(self.vao);

      if self.ibo.is_some() {
        // indexed render
        let first = (size_of::<u32>() * start_index) as *const c_void;

        if inst_nb == 1 {
          gl::DrawElements(self.mode, vert_nb, gl::UNSIGNED_INT, first);
        } else if inst_nb > 1 {
          gl::DrawElementsInstanced(self.mode, vert_nb, gl::UNSIGNED_INT, first, inst_nb);
        } else {
          panic!("cannot index-render 0 instance");
        }
      } else {
        // direct render
        let first = start_index as GLint;

        if inst_nb == 1 {
          gl::DrawArrays(self.mode, first, vert_nb);
        } else if inst_nb > 1 {
          gl::DrawArraysInstanced(self.mode, first, vert_nb, inst_nb);
        } else {
          panic!("cannot render 0 instance");
        }
      }
    }
  }

  /// Get an immutable slice over the vertices stored on GPU.
  pub fn as_slice(&self) -> Result<BufferSlice<V>, TessMapError> {
    match self.vbo.len() {
      0 => Err(TessMapError::ForbiddenAttributelessMapping),
      1 => self.vbo[0]
        .as_slice()
        .map_err(TessMapError::VertexBufferMapFailed),
      _ => Err(TessMapError::ForbiddenDeinterleavedMapping),
    }
  }

  /// Get a mutable slice over the vertices stored on GPU.
  pub fn as_slice_mut<C>(&mut self) -> Result<BufferSliceMut<V>, TessMapError> {
    match self.vbo.len() {
      0 => Err(TessMapError::ForbiddenAttributelessMapping),
      1 => self.vbo[0]
        .as_slice_mut()
        .map_err(TessMapError::VertexBufferMapFailed),
      _ => Err(TessMapError::ForbiddenDeinterleavedMapping),
    }
  }
}

impl Tess<()> {
  /// Create a tessellation that will procedurally generate its vertices (i.e. *attribute-less*).
  ///
  /// You just have to give the `Mode` to use and the number of vertices the tessellation must
  /// generate. You’ll be handed back a tessellation object that doesn’t actually hold anything.
  /// You will have to generate the vertices on the fly in your shaders.
  pub fn attributeless<C>(ctx: &mut C, mode: Mode, vert_nb: usize) -> Self
  where
    C: GraphicsContext,
  {
    let mut gfx_state = ctx.state().borrow_mut();
    let mut vao = 0;

    unsafe {
      gl::GenVertexArrays(1, &mut vao);

      gfx_state.bind_vertex_array(vao);
      gfx_state.bind_vertex_array(0);

      Tess {
        mode: opengl_mode(mode),
        vert_nb,
        vao,
        vbo: Vec::new(),
        ibo: None,
        _v: PhantomData,
      }
    }
  }
}

impl<V> Drop for Tess<V> {
  fn drop(&mut self) {
    unsafe {
      gl::DeleteVertexArrays(1, &self.vao);
    }
  }
}

// Give OpenGL types information on the content of the VBO by setting vertex formats and pointers
// to buffer memory.
//
// This is the interleaved version: it must be used for a single buffer only. If you want to set
// vertex pointer for a single buffer (deinterleaved buffers), please switch to the
// `set_vertex_pointer_deinterleaved` function instead
fn set_vertex_pointers_interleaved(formats: &[VertexAttributeFmt]) {
  // this function sets the vertex attribute pointer for the input list by computing:
  //   - The vertex attribute ID: this is the “rank” of the attribute in the input list (order
  //     matters, for short).
  //   - The stride: this is easily computed, since it’s the size (bytes) of a single vertex.
  //   - The offsets: each attribute has a given offset in the buffer. This is computed by
  //     accumulating the size of all previously set attributes.
  let offsets = aligned_offsets(formats);
  let vertex_weight = offset_based_vertex_weight(formats, &offsets) as GLsizei;

  for (i, (format, off)) in formats.iter().zip(offsets).enumerate() {
    set_component_format(i as u32, vertex_weight, off, format);
  }
}

// Give OpenGL types information on the content of the VBO by setting vertex format and pointer to
// buffer memory.
//
// This is the deinterleaved version. It will set the vertex attribute pointer for the given buffer.
fn set_vertex_pointer_deinterleaved(format: &VertexAttributeFmt, index: u32) {
  let stride = component_weight(format) as GLsizei;
  set_component_format(index, stride, 0, format);
}

// Compute offsets for all the vertex components according to the alignments provided.
fn aligned_offsets(formats: &[VertexAttributeFmt]) -> Vec<usize> {
  let mut offsets = Vec::with_capacity(formats.len());
  let mut off = 0;

  // compute offsets
  for f in formats {
    off = off_align(off, f.align); // keep the current component format aligned
    offsets.push(off);
    off += component_weight(f); // increment the offset by the pratical size of the component
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
fn component_weight(f: &VertexAttributeFmt) -> usize {
  dim_as_size(&f.dim) as usize * f.unit_size
}

fn dim_as_size(d: &VertexAttributeDim) -> GLint {
  match *d {
    VertexAttributeDim::Dim1 => 1,
    VertexAttributeDim::Dim2 => 2,
    VertexAttributeDim::Dim3 => 3,
    VertexAttributeDim::Dim4 => 4,
  }
}

// Weight in bytes of a single vertex, taking into account padding so that the vertex stay correctly
// aligned.
fn offset_based_vertex_weight(formats: &[VertexAttributeFmt], offsets: &[usize]) -> usize {
  if formats.is_empty() || offsets.is_empty() {
    return 0;
  }

  off_align(
    offsets[offsets.len() - 1] + component_weight(&formats[formats.len() - 1]),
    formats[0].align,
  )
}

// Set the vertex component OpenGL pointers regarding the index of the component (i), the stride
fn set_component_format(i: u32, stride: GLsizei, off: usize, f: &VertexAttributeFmt) {
  match f.comp_type {
    VertexAttributeType::Floating => unsafe {
      gl::VertexAttribPointer(
        i as GLuint,
        dim_as_size(&f.dim),
        opengl_sized_type(&f),
        gl::FALSE,
        stride,
        ptr::null::<c_void>().offset(off as isize),
      );
    },
    VertexAttributeType::Integral | VertexAttributeType::Unsigned | VertexAttributeType::Boolean => unsafe {
      gl::VertexAttribIPointer(
        i as GLuint,
        dim_as_size(&f.dim),
        opengl_sized_type(&f),
        stride,
        ptr::null::<c_void>().offset(off as isize),
      );
    },
  }

  unsafe {
    gl::EnableVertexAttribArray(i as GLuint);
  }
}

fn opengl_sized_type(f: &VertexAttributeFmt) -> GLenum {
  match (f.comp_type, f.unit_size) {
    (VertexAttributeType::Integral, 1) => gl::BYTE,
    (VertexAttributeType::Integral, 2) => gl::SHORT,
    (VertexAttributeType::Integral, 4) => gl::INT,
    (VertexAttributeType::Unsigned, 1) | (VertexAttributeType::Boolean, 1) => gl::UNSIGNED_BYTE,
    (VertexAttributeType::Unsigned, 2) => gl::UNSIGNED_SHORT,
    (VertexAttributeType::Unsigned, 4) => gl::UNSIGNED_INT,
    (VertexAttributeType::Floating, 4) => gl::FLOAT,
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
pub struct TessSlice<'a, V>
where
  V: 'a,
{
  /// Tessellation to render.
  tess: &'a Tess<V>,
  /// Start index (vertex) in the tessellation.
  start_index: usize,
  /// Number of vertices to pick from the tessellation. If `None`, all of them are selected.
  vert_nb: usize,
  /// Number of instances to render.
  inst_nb: usize,
}

impl<'a, V> TessSlice<'a, V> {
  /// Create a tessellation render that will render the whole input tessellation with only one
  /// instance.
  pub fn one_whole(tess: &'a Tess<V>) -> Self {
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
  pub fn one_sub(tess: &'a Tess<V>, vert_nb: usize) -> Self {
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
  pub fn one_slice(tess: &'a Tess<V>, start: usize, nb: usize) -> Self {
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
  pub fn render<C>(&self, ctx: &mut C)
  where
    C: GraphicsContext,
    V: Vertex<'a>,
  {
    self
      .tess
      .render(ctx, self.start_index, self.vert_nb, self.inst_nb);
  }
}

impl<'a, V> From<&'a Tess<V>> for TessSlice<'a, V> {
  fn from(tess: &'a Tess<V>) -> Self {
    TessSlice::one_whole(tess)
  }
}

pub trait TessSliceIndex<Idx, V> {
  fn slice<'a>(&'a self, idx: Idx) -> TessSlice<'a, V>;
}

impl<V> TessSliceIndex<RangeFull, V> for Tess<V> {
  fn slice<'a>(&'a self, _: RangeFull) -> TessSlice<'a, V> {
    TessSlice::one_whole(self)
  }
}

impl<V> TessSliceIndex<RangeTo<usize>, V> for Tess<V> {
  fn slice<'a>(&'a self, to: RangeTo<usize>) -> TessSlice<'a, V> {
    TessSlice::one_sub(self, to.end)
  }
}

impl<V> TessSliceIndex<RangeFrom<usize>, V> for Tess<V> {
  fn slice<'a>(&'a self, from: RangeFrom<usize>) -> TessSlice<'a, V> {
    TessSlice::one_slice(self, from.start, self.vert_nb)
  }
}

impl<V> TessSliceIndex<Range<usize>, V> for Tess<V> {
  fn slice<'a>(&'a self, range: Range<usize>) -> TessSlice<'a, V> {
    TessSlice::one_slice(self, range.start, range.end)
  }
}

// this visitor creates the deinterleaved buffers by visiting the input slices from the
// deinterleaved representation of the vertex type
struct DeinterleavingVisitor<'a, C, V> {
  ctx: &'a mut C,
  vertex_fmt: V,
  buffers: Vec<RawBuffer>,
  vert_nb: usize,
}

impl<'a, C, V> DeinterleavingVisitor<'a, C, V> {
  fn new(ctx: &'a mut C, vertex_fmt: V) -> Self {
    DeinterleavingVisitor {
      ctx,
      vertex_fmt,
      buffers: Vec::new(),
      vert_nb: 0,
    }
  }
}

impl<'a, C, V> SliceVisitor for DeinterleavingVisitor<'a, C, V>
where
  C: GraphicsContext,
  V: Iterator<Item = VertexAttributeFmt>,
{
  fn visit_slice<T>(&mut self, slice: &[T]) {
    let buffer = Buffer::from_slice(self.ctx, slice);
    let raw_buf = buffer.to_raw();

    if self.vert_nb == 0 {
      self.vert_nb = slice.len();
    }

    unsafe { self.ctx.state().borrow_mut().bind_array_buffer(raw_buf.handle()) };

    // get the next vertex attribute format
    let vertex_attr_format = self.vertex_fmt.next().unwrap();
    set_vertex_pointer_deinterleaved(&vertex_attr_format, self.buffers.len() as u32);

    self.buffers.push(raw_buf);
  }
}
