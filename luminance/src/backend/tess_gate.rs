//! Tessellation gate backend interface.
//!
//! This interface defines the low-level API tessellation gates must implement to be usable.

use crate::backend::tess::Tess;

pub unsafe trait TessGate<V, I, W>: Tess<V, I, W> {
  unsafe fn render(
    &mut self,
    tess: &Self::TessRepr,
    start_index: usize,
    vert_nb: usize,
    inst_nb: usize,
  );
}
