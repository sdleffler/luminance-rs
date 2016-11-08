
/// Implement this trait to expose the concept of shader stages.
pub trait HasStage {
  /// Representation of a shader stage.
  type AStage;

  /// Create a new shader stage with its type and its source code.
  fn new_shader(shader_type: Type, src: &str) -> Result<Self::AStage, StageError>;
  /// Free a shader stage.
  fn free_shader(shader: &mut Self::AStage);
}

/// A shader stage type.
#[derive(Clone, Copy, Debug)]
pub enum Type {
  TessellationControlShader,
  TessellationEvaluationShader,
  VertexShader,
  GeometryShader,
  FragmentShader
}

/// A shader stage.
#[derive(Debug)]
pub struct Stage<C> where C: HasStage {
  pub repr: C::AStage,
  ty: Type
}

impl<C> Drop for Stage<C> where C: HasStage {
  fn drop(&mut self) {
    C::free_shader(&mut self.repr)
  }
}

impl<C> Stage<C> where C: HasStage {
  pub fn new(ty: Type, src: &str) -> Result<Self, StageError> {
    C::new_shader(ty, src).map(|stage| Stage {
      repr: stage,
      ty: ty
    })
  }
}

/// Errors that shader stages can emit.
#[derive(Clone, Debug)]
pub enum StageError {
  /// Occurs when a shader fails to compile.
  CompilationFailed(Type, String),
  /// Occurs when you try to create a shader which type is not supported on the current hardware.
  UnsupportedType(Type)
}
