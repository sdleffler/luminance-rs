//! Render gate backend interface.
//!
//! This interface defines the low-level API render gates must implement to be usable.
//!
//! A render gate is a special kind of pipeline node that allows to group renders behind a shared [`RenderState`]. All
//! subsequent nodes in the pipeline will be using that render state.

use crate::render_state::RenderState;

/// Render gate and associated [`RenderState`].
pub unsafe trait RenderGate {
  /// Enter the [`RenderGate`] and share the [`RenderState`] for all subsequent nodes in the pipeline.
  unsafe fn enter_render_state(&mut self, rdr_st: &RenderState);
}
