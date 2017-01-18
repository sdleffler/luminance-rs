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

use buffer::{Buffer, BufferError, BufferSlice, BufferSliceMut};
use vertex::{Vertex, VertexFormat};

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

/// Trait to implement to provide tessellation features.
pub trait HasTess {
  /// A type representing tessellation on GPU.
  type Tess;

  /// Create a tessellation from its vertices and a `Mode`.
  ///
  /// If `indices == None`, the `vertices` represent an array of vertices that are connected to each
  /// others in the order they appear. If you want to connect them in another way, you can index
  /// them with `Some(indices)`.
  fn new_tess<T>(mode: Mode, vertices: &[T], indices: Option<&[u32]>) -> Self::Tess where T: Vertex;
  /// Destroy a tessellation.
  fn destroy_tess(tessellation: &mut Self::Tess);
  /// Create a tessellation that will procedurally generate its vertices (i.e. *attribute-less*).
  ///
  /// You just have to give the `Mode` to use and the number of vertices the tessellation must
  /// have. You’ll be handed back a tessellation object that doesn’t actually hold anything. You
  /// will have to generate the vertices on the fly in your shaders.
  fn attributeless(mode: Mode, vert_nb: usize) -> Self::Tess;
  /// Retrieve the vertex format the tessellation was created with.
  fn vertex_format(tessellation: &Self::Tess) -> &VertexFormat;
  /// Get a reference to the vertex buffer and the number of elements in it.
  fn get_vertex_buffer_ref_mut(tessellation: &mut Self::Tess) -> Option<(&mut Buffer, usize)>;
}

/// GPU tessellation.
#[derive(Debug)]
pub struct Tess<C> where C: HasTess {
  pub repr: C::Tess
}

impl<C> Tess<C> where C: HasTess {
  /// Create a new tessellation.
  ///
  /// The `mode` argument gives the type of the primitives and how to interpret the `vertices` and
  /// `indices` slices. If `indices` is set to `None`, the tessellation will use the `vertices`
  /// as-is.
  pub fn new<T>(mode: Mode, vertices: &[T], indices: Option<&[u32]>) -> Tess<C> where T: Vertex {
    Tess {
      repr: C::new_tess(mode, vertices, indices)
    }
  }

  /// Create a tessellation that will procedurally generate its vertices (i.e. *attribute-less*).
  ///
  /// You just have to give the `Mode` to use and the number of vertices the tessellation must
  /// have. You’ll be handed back a tessellation object that doesn’t actually hold anything. You
  /// will have to generate the vertices on the fly in your shaders.
  pub fn attributeless(mode: Mode, vert_nb: usize) -> Tess<C> {
    Tess {
      repr: C::attributeless(mode, vert_nb)
    }
  }

  fn get_vertex_buffer<T>(&mut self) -> Result<(&mut Buffer, usize), TessMapError> where T: Vertex {
    {
      let live_vf = C::vertex_format(&self.repr);
      let req_vf = T::vertex_format();

      if live_vf != &req_vf {
        return Err(TessMapError::MismatchVertexFormat(live_vf.clone(), req_vf));
      }
    }

    match C::get_vertex_buffer_ref_mut(&mut self.repr) {
      Some(x) => Ok(x),
      None => Err(TessMapError::ForbiddenAttributelessMapping)
    }
  }

  pub fn get<T>(&mut self) -> Result<BufferSlice<C, T>, TessMapError> where T: Vertex {
    let (vertex_buffer, len) = self.get_vertex_buffer::<T>()?;

    BufferSlice::map(vertex_buffer, len).map_err(TessMapError::VertexBufferMapFailed)
  }

  pub fn get_mut<T>(&mut self) -> Result<BufferSliceMut<C, T>, TessMapError> where T: Vertex {
    let (vertex_buffer, len) = self.get_vertex_buffer::<T>()?;

    BufferSliceMut::map_mut(vertex_buffer, len).map_err(TessMapError::VertexBufferMapFailed)
  }
}

impl<C> Drop for Tess<C> where C: HasTess {
  fn drop(&mut self) {
    C::destroy_tess(&mut self.repr);
  }
}

/// Error that can occur while trying to map GPU tessellation to host code.
#[derive(Debug, Eq, PartialEq)]
pub enum TessMapError {
  MismatchVertexFormat(VertexFormat, VertexFormat),
  VertexBufferMapFailed(BufferError),
  ForbiddenAttributelessMapping
}
