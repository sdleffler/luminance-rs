//! Tessellation gate backend interface.
//!
//! This interface defines the low-level API tessellation gates must implement to be usable.

use crate::backend::tess::Tess;
use crate::tess::TessIndex;
use crate::vertex::Vertex;

pub unsafe trait TessGate<V, I, W>: Tess<V, I, W>
where
  V: Vertex,
  I: TessIndex,
  W: Vertex,
{
  unsafe fn render(
    &mut self,
    tess: &Self::TessRepr,
    start_index: usize,
    vert_nb: usize,
    inst_nb: usize,
  );
}
