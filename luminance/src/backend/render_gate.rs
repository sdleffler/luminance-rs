//! Render gate backend interface.
//!
//! This interface defines the low-level API render gates must implement to be usable.

use crate::render_state::RenderState;

pub unsafe trait RenderGate {
  unsafe fn enter_render_state(&mut self, rdr_st: &RenderState);
}
