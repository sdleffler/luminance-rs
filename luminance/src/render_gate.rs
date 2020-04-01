//! Render gates.
//!
//! A render gate is a _pipeline node_ that allows to share [`RenderState`] for deeper nodes,
//! which are used to render [`Tess`].
//!
//! [`Tess`]: crate::tess::Tess

use crate::backend::render_gate::RenderGate as RenderGateBackend;
use crate::context::GraphicsContext;
use crate::render_state::RenderState;
use crate::tess_gate::TessGate;

/// A render gate.
///
/// # Parametricity
///
/// - `C` is the backend type. It must implement [`GraphicsContext`] and `C::Backend` must
///   implement [`backend::RenderGate`].
///
/// [`backend::RenderGate`]: crate::backend::render_gate::RenderGate
pub struct RenderGate<'a, C>
where
  C: ?Sized,
{
  pub(crate) ctx: &'a mut C,
}

impl<'a, C> RenderGate<'a, C>
where
  C: ?Sized + GraphicsContext,
  C::Backend: RenderGateBackend,
{
  /// Enter a [`RenderGate`] and go deeper in the pipeline.
  pub fn render<'b, F>(&'b mut self, rdr_st: &RenderState, f: F)
  where
    F: FnOnce(TessGate<'b, C>),
  {
    unsafe {
      self.ctx.backend().enter_render_state(rdr_st);
    }

    let tess_gate = TessGate { ctx: self.ctx };
    f(tess_gate);
  }
}
