//! Tessellation features.
//!
//! # Tessellation mode
//!
//! Tessellation is geometric information. Currently, several kind of tessellation is supported:
//!
//! - *point clouds*;
//! - *lines*;
//! - *line strips*;
//! - *triangles*;
//! - *triangle fans*;
//! - *triangle strips*.
//!
//! Those kind of tessellation are designated by the `Mode` type.
//!
//! # Tessellation abstraction
//!
//! The tessellation is an abstract concept that depends on the backend. That’s why `Tessellation`
//! is an associated type found in `HasTessellation`.
//!
//! You create a new `Tessellation` with the `new` function, and you can render it with `render`.

use buffer::{BufferSlice, BufferSliceMut, HasBuffer};
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
pub trait HasTessellation: HasBuffer {
  /// A type representing tessellation on GPU.
  type Tessellation;

  /// Create a `Tessellation` from its vertices and a `Mode`.
  ///
  /// If `indices == None`, the `vertices` represent an array of vertices that are connected to each
  /// others in the order they appear. If you want to connect them in another way, you can index
  /// them with `Some(indices)`.
  fn new_tessellation<T>(mode: Mode, vertices: &[T], indices: Option<&[u32]>) -> Self::Tessellation where T: Vertex;
  /// Destroy a `Tessellation`.
  fn destroy_tessellation(tessellation: &mut Self::Tessellation);
  /// Create a `Tessellation` that will procedurally generate its vertices (i.e. *attribute-less*).
  ///
  /// You just have to give the `Mode` to use and the number of vertices the `Tessellation` must
  /// have. You’ll be handed back a `Tessellation` object that doesn’t actually hold anything. You
  /// will have to generate the vertices on the fly in your shaders.
  fn attributeless(mode: Mode, vert_nb: usize) -> Self::Tessellation;
  /// Retrieve the vertex format the tessellation was created with.
  fn vertex_format(tessellation: &Self::Tessellation) -> VertexFormat;
  /// Get a reference to the vertex buffer and the number of elements in it.
  fn get_vertex_buffer_ref_mut(tessellation: &mut Self::Tessellation) -> (&mut Self::ABuffer, usize);
}

/// GPU Tessellation.
#[derive(Debug)]
pub struct Tessellation<C> where C: HasTessellation {
  pub repr: C::Tessellation
}

impl<C> Tessellation<C> where C: HasTessellation {
  /// Create a new tessellation.
  ///
  /// The `mode` argument gives the type of the primitives and how to interpret the `vertices` and
  /// `indices` slices. If `indices` is set to `None`, the tessellation will use the `vertices`
  /// as-is.
  pub fn new<T>(mode: Mode, vertices: &[T], indices: Option<&[u32]>) -> Tessellation<C> where T: Vertex {
    Tessellation {
      repr: C::new_tessellation(mode, vertices, indices)
    }
  }

  /// Create a `Tessellation` that will procedurally generate its vertices (i.e. *attribute-less*).
  ///
  /// You just have to give the `Mode` to use and the number of vertices the `Tessellation` must
  /// have. You’ll be handed back a `Tessellation` object that doesn’t actually hold anything. You
  /// will have to generate the vertices on the fly in your shaders.
  pub fn attributeless(mode: Mode, vert_nb: usize) -> Tessellation<C> {
    Tessellation {
      repr: C::attributeless(mode, vert_nb)
    }
  }

  fn get_vertex_buffer<T>(&mut self) -> Result<(&mut C::ABuffer, usize), TessellationMapError> where T: Vertex {
    let live_vf = C::vertex_format(&self.repr);
    let req_vf = T::vertex_format();

    if live_vf != req_vf {
      return Err(TessellationMapError::MismatchVertexFormat(live_vf, req_vf));
    }

    Ok(C::get_vertex_buffer_ref_mut(&mut self.repr))
  }

  pub fn get<T>(&mut self) -> Result<BufferSlice<C, T>, TessellationMapError> where T: Vertex {
    let (vertex_buffer, len) = self.get_vertex_buffer::<T>()?;

    Ok(BufferSlice::map(vertex_buffer, len))
  }

  pub fn get_mut<T>(&mut self) -> Result<BufferSliceMut<C, T>, TessellationMapError> where T: Vertex {
    let (vertex_buffer, len) = self.get_vertex_buffer::<T>()?;

    Ok(BufferSliceMut::map_mut(vertex_buffer, len))
  }
}

impl<C> Drop for Tessellation<C> where C: HasTessellation {
  fn drop(&mut self) {
    C::destroy_tessellation(&mut self.repr);
  }
}

/// Error that can occur while trying to map GPU tessellation to host code.
pub enum TessellationMapError {
  MismatchVertexFormat(VertexFormat, VertexFormat)
}
