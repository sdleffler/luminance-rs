use crate::Backend;

pub use luminance::shader::{
  ProgramError, ProgramWarning, StageError, StageType, TessellationStages, Uniform, UniformType,
  UniformWarning, VertexAttribWarning,
};

pub type Stage = luminance::shader::Stage<Backend>;
pub type UniformBuilder<'a> = luminance::shader::UniformBuilder<'a, Backend>;
pub type BuiltProgram<Sem, Out, Uni> = luminance::shader::BuiltProgram<Backend, Sem, Out, Uni>;
pub type AdaptationFailure<Sem, Out, Uni> =
  luminance::shader::AdaptationFailure<Backend, Sem, Out, Uni>;
pub type ProgramInterface<'a> = luminance::shader::ProgramInterface<'a, Backend>;
pub type Program<Sem, Out, Uni> = luminance::shader::Program<Backend, Sem, Out, Uni>;
