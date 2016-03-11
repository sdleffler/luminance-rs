/// Possible type of vertex components.
pub enum VertexComponentType {
    Integral
  , Unsigned
  , Floating
}

/// A `VertexComponentFormat` gives hints about:
///
/// - the type of the component (`VertexComponentType`);
/// - the dimension of the component (`u8`).
pub struct VertexComponentFormat {
    comp_type: VertexComponentType
  , dim: u8
}
