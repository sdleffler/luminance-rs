//! Tessellation gate backend interface.
//!
//! This interface defines the low-level API tessellation gates must implement to be usable.

use crate::backend::tess::Tess;
use crate::tess::{TessIndex, TessVertexData};

pub unsafe trait TessGate<V, I, W, S>: Tess<V, I, W, S>
where
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  unsafe fn render(
    &mut self,
    tess: &Self::TessRepr,
    start_index: usize,
    vert_nb: usize,
    inst_nb: usize,
  );
}
