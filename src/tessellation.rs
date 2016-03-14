// use std::collections::BTreeSet;
use vertex::Vertex;

/// Vertices can be connected via several modes.
pub enum Mode {
    Point
  , Line
  , LineStrip
  , Triangle
  , TriangleFan
  , TriangleStrip
}

/// Trait to implement to provide tessellation features.
trait HasTessellation {
  /// A type representing tessellation on GPU.
  type Tessellation;

  /// Create a `Tessellation` from its vertices and a `Mode`.
  ///
  /// If `indices == None`, the `vertices` represent an array of vertices that are connected to each
  /// others in the order they appear. If you want to connect them in another way, you can index
  /// them with `Some(indices)`.
  fn new<T>(mode: Mode, vertices: Vec<T>, indices: Option<u32>) -> Self::Tessellation where T: Vertex;
  /// Render the tessellation. The `instances` parameter can be set to render several instances of
  /// the tessellation.
  fn render(tessellation: &Self::Tessellation, instances: Option<u32>);
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
