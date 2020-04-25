//! Tessellation backend interface.
//!
//! This interface defines the low-level API tessellations must implement to be usable.

use std::ops::DerefMut;

use crate::backend::buffer::Buffer;
use crate::tess::{Mode, TessError, TessIndex, TessMapError, TessStorage};
use crate::vertex::Vertex;

pub unsafe trait Tess<V, I, W> {
  type TessRepr;

  type VertexSliceRepr: DerefMut<Target = [V]>;

  type IndexSliceRepr: DerefMut<Target = [I]>;

  type InstanceSliceRepr: DerefMut<Target = [W]>;

  unsafe fn build(
    &mut self,
    vertex_data: Vec<V>,
    index_data: Vec<I>,
    instance_data: Vec<W>,
    mode: Mode,
    vert_nb: usize,
    inst_nb: usize,
    restart_index: Option<I>,
  ) -> Result<Self::TessRepr, TessError>;

  unsafe fn tess_vertices_nb(tess: &Self::TessRepr) -> usize;

  unsafe fn tess_instances_nb(tess: &Self::TessRepr) -> usize;

  unsafe fn render(
    tess: &Self::TessRepr,
    start_index: usize,
    vert_nb: usize,
    inst_nb: usize,
  ) -> Result<(), TessError>;

  unsafe fn vertices(tess: &mut Self::TessRepr) -> Result<Self::VertexSliceRepr, TessMapError>
  where
    V: Vertex;

  unsafe fn vertices_mut(tess: &mut Self::TessRepr) -> Result<Self::VertexSliceRepr, TessMapError>
  where
    V: Vertex;

  unsafe fn indices(tess: &mut Self::TessRepr) -> Result<Self::IndexSliceRepr, TessMapError>
  where
    I: TessIndex;

  unsafe fn indices_mut(tess: &mut Self::TessRepr) -> Result<Self::IndexSliceRepr, TessMapError>
  where
    I: TessIndex;

  unsafe fn instances(tess: &mut Self::TessRepr) -> Result<Self::InstanceSliceRepr, TessMapError>
  where
    W: Vertex;

  unsafe fn instances_mut(
    tess: &mut Self::TessRepr,
  ) -> Result<Self::InstanceSliceRepr, TessMapError>
  where
    W: Vertex;
}
