//! Tessellation backend interface.
//!
//! This interface defines the low-level API tessellations must implement to be usable.

use std::ops::{Deref, DerefMut};

use crate::tess::{Mode, TessError, TessIndex, TessMapError, TessVertexData};

pub unsafe trait Tess<V, I, W, S>
where
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  type TessRepr;

  unsafe fn build(
    &mut self,
    vertex_data: Option<V::Data>,
    index_data: Vec<I>,
    instance_data: Option<W::Data>,
    mode: Mode,
    restart_index: Option<I>,
  ) -> Result<Self::TessRepr, TessError>;

  unsafe fn tess_vertices_nb(tess: &Self::TessRepr) -> usize;

  unsafe fn tess_indices_nb(tess: &Self::TessRepr) -> usize;

  unsafe fn tess_instances_nb(tess: &Self::TessRepr) -> usize;

  unsafe fn render(
    tess: &Self::TessRepr,
    start_index: usize,
    vert_nb: usize,
    inst_nb: usize,
  ) -> Result<(), TessError>;
}

pub unsafe trait VertexSlice<'a, V, I, W, S, T>: Tess<V, I, W, S>
where
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  type VertexSliceRepr: 'a + Deref<Target = [T]>;
  type VertexSliceMutRepr: 'a + DerefMut<Target = [T]>;

  unsafe fn vertices(tess: &'a mut Self::TessRepr) -> Result<Self::VertexSliceRepr, TessMapError>;

  unsafe fn vertices_mut(
    tess: &'a mut Self::TessRepr,
  ) -> Result<Self::VertexSliceMutRepr, TessMapError>;
}

pub unsafe trait IndexSlice<'a, V, I, W, S>: Tess<V, I, W, S>
where
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  type IndexSliceRepr: 'a + Deref<Target = [I]>;
  type IndexSliceMutRepr: 'a + DerefMut<Target = [I]>;

  unsafe fn indices(tess: &'a mut Self::TessRepr) -> Result<Self::IndexSliceRepr, TessMapError>;

  unsafe fn indices_mut(
    tess: &'a mut Self::TessRepr,
  ) -> Result<Self::IndexSliceMutRepr, TessMapError>;
}

pub unsafe trait InstanceSlice<'a, V, I, W, S, T>: Tess<V, I, W, S>
where
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  type InstanceSliceRepr: 'a + Deref<Target = [T]>;
  type InstanceSliceMutRepr: 'a + DerefMut<Target = [T]>;

  unsafe fn instances(
    tess: &'a mut Self::TessRepr,
  ) -> Result<Self::InstanceSliceRepr, TessMapError>;

  unsafe fn instances_mut(
    tess: &'a mut Self::TessRepr,
  ) -> Result<Self::InstanceSliceMutRepr, TessMapError>;
}
