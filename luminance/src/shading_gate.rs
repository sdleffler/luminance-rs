//! Shading gates.
//!
//! A shading gate is a _pipeline node_ that allows to share shader [`Program`] for deeper nodes.
//!
//! [`Program`]: crate::shader::Program

use crate::backend::shading_gate::ShadingGate as ShadingGateBackend;
use crate::context::GraphicsContext;
use crate::render_gate::RenderGate;
use crate::shader::{Program, ProgramInterface, UniformInterface};
use crate::vertex::Semantics;

/// A shading gate.
///
/// This is obtained after entering a [`PipelineGate`].
///
/// # Parametricity
///
/// - `C` is the backend type.
///
/// [`PipelineGate`]: crate::pipeline::PipelineGate
pub struct ShadingGate<'a, C>
where
  C: ?Sized + GraphicsContext,
  C::Backend: ShadingGateBackend,
{
  pub(crate) ctx: &'a mut C,
}

impl<'a, C> ShadingGate<'a, C>
where
  C: ?Sized + GraphicsContext,
  C::Backend: ShadingGateBackend,
{
  /// Enter a [`ShadingGate`] by using a shader [`Program`].
  ///
  /// The argument closure is given two arguments:
  ///
  /// - A [`ProgramInterface`], that allows to pass values (via [`ProgramInterface::set`]) to the
  ///   in-use shader [`Program`] and/or perform dynamic lookup of uniforms.
  /// - A [`RenderGate`], allowing to create deeper nodes in the graphics pipeline.
  pub fn shade<Sem, Out, Uni, F>(&mut self, program: &mut Program<C::Backend, Sem, Out, Uni>, f: F)
  where
    Sem: Semantics,
    Uni: UniformInterface<C::Backend>,
    F: for<'b> FnOnce(ProgramInterface<'b, C::Backend>, &'b Uni, RenderGate<'b, C>),
  {
    unsafe {
      self.ctx.backend().apply_shader_program(&mut program.repr);
    }

    let render_gate = RenderGate { ctx: self.ctx };
    let program_interface = ProgramInterface {
      program: &mut program.repr,
    };

    f(program_interface, &program.uni, render_gate);
  }
}
