//! Tessellation gate backend interface.
//!
//! This interface defines the low-level API tessellation gates must implement to be usable.

use crate::backend::tess::Tess;

pub unsafe trait TessGate: Tess {
  unsafe fn render(
    &mut self,
    tess: &Self::TessRepr,
    start_index: usize,
    vert_nb: usize,
    inst_nb: usize,
  );
}
