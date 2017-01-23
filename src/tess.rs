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
//! # Tessellation abstraction
//!
//! The tessellation is an abstract concept that depends on the backend. That’s why tessellation is an
//! associated type found in the `HasTess` trait.
//!
//! You create a new tessellation with the `new` function.

use gl;
use gl::types::*;
use std::ptr;

use buffer::{Buffer, BufferError, BufferSlice, BufferSliceMut, RawBuffer};
use vertex::{Dim, Type, Vertex, VertexComponentFormat, VertexFormat};

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
  TriangleStrip
}

/// Error that can occur while trying to map GPU tessellation to host code.
#[derive(Debug, Eq, PartialEq)]
pub enum TessMapError {
  MismatchVertexFormat(VertexFormat, VertexFormat),
  VertexBufferMapFailed(BufferError),
  ForbiddenAttributelessMapping
}

/// Accepted vertices for building tessellations.
///
/// This type enables you to pass in a slice of vertices or ask for the GPU to only reserve enough
/// space for the number of vertices.
pub enum TessVertices<'a, T> where T: 'a + Vertex {
  /// Pass in a slice of vertices.
  Fill(&'a [T]),
  /// Reserve as many vertices as you want.
  Reserve(usize)
}

impl<'a, T> From<&'a [T]> for TessVertices<'a, T> where T: 'a + Vertex {
  fn from(slice: &'a [T]) -> Self {
    TessVertices::Fill(slice)
  }
}

/// GPU tessellation.
pub struct Tess {
  // closure taking the point / line size and the number of instances to render
  render: Box<Fn(Option<f32>, u32)>,
  vao: GLenum,
  vbo: Option<RawBuffer>, // no vbo means attributeless render
  ibo: Option<RawBuffer>,
  vertex_format: VertexFormat,
}

impl Tess {
  /// Create a new tessellation.
  ///
  /// The `mode` argument gives the type of the primitives and how to interpret the `vertices` and
  /// `indices` slices. If `indices` is set to `None`, the tessellation will use the `vertices`
  /// as-is.
  pub fn new<'a, V, T>(mode: Mode, vertices: V, indices: Option<&[u32]>) -> Self where TessVertices<'a, T>: From<V>, T: 'a + Vertex {
    let vertices = vertices.into();

    let mut vao: GLuint = 0;
    let vert_nb = match vertices {
      TessVertices::Fill(slice) => slice.len(),
      TessVertices::Reserve(nb) => nb
    };

    unsafe {
      gl::GenVertexArrays(1, &mut vao);

      gl::BindVertexArray(vao);

      // vertex buffer
      let vertex_buffer = Buffer::new(vert_nb);

      // fill the buffer with vertices only if asked by the user
      if let TessVertices::Fill(verts) = vertices {
        vertex_buffer.fill(verts);
      }

      let raw_vbo = vertex_buffer.to_raw();

      gl::BindBuffer(gl::ARRAY_BUFFER, raw_vbo.handle());
      set_vertex_pointers(&T::vertex_format());

      // in case of indexed render, create an index buffer
      if let Some(indices) = indices {
        let ind_nb = indices.len();
        let index_buffer = Buffer::new(ind_nb);
        index_buffer.fill(indices);

        let raw_ibo = index_buffer.to_raw();

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, raw_ibo.handle());

        gl::BindVertexArray(0);

        Tess {
          render: Box::new(move |size, instances| {
            gl::BindVertexArray(vao);

            set_point_line_size(mode, size);

            if instances == 1 {
              gl::DrawElements(opengl_mode(mode), ind_nb as GLsizei, gl::UNSIGNED_INT, ptr::null());
            } else if instances > 1 {
              gl::DrawElementsInstanced(opengl_mode(mode), ind_nb as GLsizei, gl::UNSIGNED_INT, ptr::null(), instances as GLsizei);
            } else {
              panic!("cannot index-render 0 instance");
            }
          }),
          vao: vao,
          vbo: Some(raw_vbo),
          ibo: Some(raw_ibo),
          vertex_format: T::vertex_format()
        }
      } else {
        gl::BindVertexArray(0);

        Tess {
          render: Box::new(move |size, instances| {
            gl::BindVertexArray(vao);

            set_point_line_size(mode, size);

            if instances == 1 {
              gl::DrawArrays(opengl_mode(mode), 0, vert_nb as GLsizei);
            } else if instances > 1 {
              gl::DrawArraysInstanced(opengl_mode(mode), 0, vert_nb as GLsizei, instances as GLsizei);
            } else {
              panic!("cannot render 0 instance");
            }
          }),
          vao: vao,
          vbo: Some(raw_vbo),
          ibo: None,
          vertex_format: T::vertex_format()
        }
      }
    }
  }

  /// Create a tessellation that will procedurally generate its vertices (i.e. *attribute-less*).
  ///
  /// You just have to give the `Mode` to use and the number of vertices the tessellation must
  /// have. You’ll be handed back a tessellation object that doesn’t actually hold anything. You
  /// will have to generate the vertices on the fly in your shaders.
  pub fn attributeless(mode: Mode, vert_nb: usize) -> Self {
    let mut vao = 0;

    unsafe {
      gl::GenVertexArrays(1, &mut vao);

      gl::BindVertexArray(vao);
      gl::BindVertexArray(0);

      Tess {
        render: Box::new(move |size, instances| {
          gl::BindVertexArray(vao);

          set_point_line_size(mode, size);

          if instances == 1 {
            gl::DrawArrays(opengl_mode(mode), 0, vert_nb as GLsizei);
          } else if instances > 1 {
            gl::DrawArraysInstanced(opengl_mode(mode), 0, vert_nb as GLsizei, instances as GLsizei);
          } else {
            panic!("cannot render 0 instance");
          }
        }),
        vao: vao,
        vbo: None,
        ibo: None,
        vertex_format: Vec::new()
      }
    }
  }

  #[inline]
  pub fn render(&self, rasterization_size: Option<f32>, instances: u32) {
    (self.render)(rasterization_size, instances)
  }

  pub fn as_slice<T>(&self) -> Result<BufferSlice<T>, TessMapError> where T: Vertex {
    let live_vf = &self.vertex_format;
    let req_vf = T::vertex_format();

    if live_vf != &req_vf {
      return Err(TessMapError::MismatchVertexFormat(live_vf.clone(), req_vf));
    }

    self.vbo.as_ref()
      .ok_or(TessMapError::ForbiddenAttributelessMapping)
      .and_then(|raw| unsafe { RawBuffer::as_slice(raw).map_err(TessMapError::VertexBufferMapFailed) })
  }

  pub fn as_slice_mut<T>(&mut self) -> Result<BufferSliceMut<T>, TessMapError> where T: Vertex {
    let live_vf = &self.vertex_format;
    let req_vf = T::vertex_format();

    if live_vf != &req_vf {
      return Err(TessMapError::MismatchVertexFormat(live_vf.clone(), req_vf));
    }

    self.vbo.as_mut()
      .ok_or(TessMapError::ForbiddenAttributelessMapping)
      .and_then(|raw| unsafe { RawBuffer::as_slice_mut(raw).map_err(TessMapError::VertexBufferMapFailed) })
  }
}

impl Drop for Tess {
  fn drop(&mut self) {
    // delete the vertex array and all bound buffers
    unsafe {
      gl::DeleteVertexArrays(1, &self.vao);

      if let &Some(ref vbo) = &self.vbo {
        gl::DeleteBuffers(1, &vbo.handle());
      }

      if let &Some(ref ibo) = &self.ibo {
        gl::DeleteBuffers(1, &ibo.handle());
      }
    }
  }
}

// Give OpenGL types information on the content of the VBO by setting vertex formats and pointers
// to buffer memory.
fn set_vertex_pointers(formats: &[VertexComponentFormat]) {
  let offsets = aligned_offsets(formats);
  let vertex_weight = offset_based_vertex_weight(formats, &offsets) as GLsizei;

  for (i, (format, off)) in formats.iter().zip(offsets).enumerate() {
    set_component_format(i as u32, vertex_weight, off, format);
  }
}

// Compute offsets for all the vertex components according to the alignments provided.
fn aligned_offsets(formats: &[VertexComponentFormat]) -> Vec<usize> {
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
fn component_weight(f: &VertexComponentFormat) -> usize {
  dim_as_size(&f.dim) as usize * f.unit_size
}

fn dim_as_size(d: &Dim) -> GLint {
  match *d {
    Dim::Dim1 => 1,
    Dim::Dim2 => 2,
    Dim::Dim3 => 3,
    Dim::Dim4 => 4
  }
}

// Weight in bytes of a single vertex, taking into account padding so that the vertex stay correctly
// aligned.
fn offset_based_vertex_weight(formats: &[VertexComponentFormat], offsets: &[usize]) -> usize {
  if formats.is_empty() || offsets.is_empty() {
    return 0;
  }

  off_align(offsets[offsets.len() - 1] + component_weight(&formats[formats.len() - 1]), formats[0].align)
}

// Set the vertex component OpenGL pointers regarding the index of the component (i), the stride
fn set_component_format(i: u32, stride: GLsizei, off: usize, f: &VertexComponentFormat) {
  match f.comp_type {
    Type::Floating => {
      unsafe {
        gl::VertexAttribPointer(i as GLuint, dim_as_size(&f.dim), opengl_sized_type(&f), gl::FALSE, stride, ptr::null().offset(off as isize));
      }
    },
    Type::Integral | Type::Unsigned | Type::Boolean => {
      unsafe {
        gl::VertexAttribIPointer(i as GLuint, dim_as_size(&f.dim), opengl_sized_type(&f), stride, ptr::null().offset(off as isize));
      }
    }
  }

  unsafe {
    gl::EnableVertexAttribArray(i as GLuint);
  }
}

fn opengl_sized_type(f: &VertexComponentFormat) -> GLenum {
  match (f.comp_type, f.unit_size) {
    (Type::Integral, 1) => gl::BYTE,
    (Type::Integral, 2) => gl::SHORT,
    (Type::Integral, 4) => gl::INT,
    (Type::Unsigned, 1) | (Type::Boolean, 1) => gl::UNSIGNED_BYTE,
    (Type::Unsigned, 2) => gl::UNSIGNED_SHORT,
    (Type::Unsigned, 4) => gl::UNSIGNED_INT,
    (Type::Floating, 4) => gl::FLOAT,
    _ => panic!("unsupported vertex component format: {:?}", f)
  }
}

// Either set the point line or the line size (regarding what the mode is). If it’s not either point
// or line, that function does nothing.
fn set_point_line_size(mode: Mode, size: Option<f32>) {
  let computed = size.unwrap_or(1.);

  match mode {
    Mode::Point => unsafe { gl::PointSize(computed) },
    Mode::Line | Mode::LineStrip => unsafe { gl::LineWidth(computed) },
    _ => {}
  }
}

fn opengl_mode(mode: Mode) -> GLenum {
  match mode {
    Mode::Point => gl::POINTS,
    Mode::Line => gl::LINES,
    Mode::LineStrip => gl::LINE_STRIP,
    Mode::Triangle => gl::TRIANGLES,
    Mode::TriangleFan => gl::TRIANGLE_FAN,
    Mode::TriangleStrip => gl::TRIANGLE_STRIP
  }
}
