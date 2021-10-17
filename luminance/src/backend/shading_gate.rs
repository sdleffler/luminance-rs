//! Shading gates backend interface.
//!
//! This interface defines the low-level API shading gates must implement to be usable.
//!
//! Shading gates allow to shade a scene with a shader program.

use crate::backend::shader::Shader as ShaderBackend;

/// Shading gate backend.
///
/// This trait requires [`Shader`] as super trait.
///
/// [`Shader`]: crate::backend::shader::Shader
pub unsafe trait ShadingGate: ShaderBackend {
  /// Apply the shader program and make it currently in-use for subsequent pipeline nodes.
  unsafe fn apply_shader_program(&mut self, shader_program: &Self::ProgramRepr);
}
