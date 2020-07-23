//! Render gates.
//!
//! A render gate is a _pipeline node_ that allows to share [`RenderState`] for deeper nodes,
//! which are used to render [`Tess`].
//!
//! [`Tess`]: crate::tess::Tess

use crate::backend::render_gate::RenderGate as RenderGateBackend;
use crate::render_state::RenderState;
use crate::tess_gate::TessGate;

/// A render gate.
///
/// # Parametricity
///
/// - `B` is the backend type.
pub struct RenderGate<'a, B>
where
  B: ?Sized,
{
  pub(crate) backend: &'a mut B,
}

impl<'a, B> RenderGate<'a, B>
where
  B: ?Sized + RenderGateBackend,
{
  /// Enter a [`RenderGate`] and go deeper in the pipeline.
  pub fn render<'b, E, F>(&'b mut self, rdr_st: &RenderState, f: F) -> Result<(), E>
  where
    F: FnOnce(TessGate<'b, B>) -> Result<(), E>,
  {
    unsafe {
      self.backend.enter_render_state(rdr_st);
    }

    let tess_gate = TessGate {
      backend: self.backend,
    };

    f(tess_gate)
  }
}
