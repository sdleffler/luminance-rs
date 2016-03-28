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

// use std::collections::BTreeSet;
use vertex::Vertex;

/// Vertices can be connected via several modes.
#[derive(Copy, Clone, Debug)]
pub enum Mode {
    Point
  , Line
  , LineStrip
  , Triangle
  , TriangleFan
  , TriangleStrip
}

/// Trait to implement to provide tessellation features.
pub trait HasTessellation {
  /// A type representing tessellation on GPU.
  type Tessellation;

  /// Create a `Tessellation` from its vertices and a `Mode`.
  ///
  /// If `indices == None`, the `vertices` represent an array of vertices that are connected to each
  /// others in the order they appear. If you want to connect them in another way, you can index
  /// them with `Some(indices)`.
  fn new<T>(mode: Mode, vertices: Vec<T>, indices: Option<Vec<u32>>) -> Self::Tessellation where T: Vertex;
  /// Render the tessellation. The `instances` parameter gives the number of instances to actually
  /// draw.
  fn render(tessellation: &Self::Tessellation, instances: u32);
}

// TODO
// /// Turn *direct geometry* into *indexed geometry*. This function removes duplicate elements from
// /// the data you pass in and returns the cleaned data along with an array of indices to restore the
// /// initial data.
// ///
// /// # Complexity
// ///
// /// **O (n log n)**
// pub fn index_geometry<T>(vertices: &Vec<T>) -> (Vec<T>,Vec<u32>) where T: Ord {
//   let mut uniq: Vec<T> = Vec::with_capacity(vertices.len()); // we’ll resize later on
//   let mut seen: BTreeSet<T> = BTreeSet::new();
// }
