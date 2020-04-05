//! Shading gates backend interface.
//!
//! This interface defines the low-level API shading gates must implement to be usable.

use crate::backend::shader::Shader as ShaderBackend;

pub unsafe trait ShadingGate: ShaderBackend {
  unsafe fn apply_shader_program(&mut self, shader_program: &Self::ProgramRepr);
}
