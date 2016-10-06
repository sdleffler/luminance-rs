use std::marker::PhantomData;

/// Implement this trait to expose the concept of shader stages.
pub trait HasStage {
  /// Representation of a shader stage.
  type AStage;

  /// Create a new shader stage with its type and its source code.
  fn new_shader(shader_type: Type, src: &str) -> Result<Self::AStage, StageError>;
  /// Free a shader stage.
  fn free_shader(shader: &mut Self::AStage);
}

/// Class of types that are shader stage types.
pub trait ShaderTypeable {
  /// Reify a shader stage type to its runtime representation.
  fn shader_type() -> Type;
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

/// Tessellation control shader. An optional stage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TessellationControlShader;

impl ShaderTypeable for TessellationControlShader {
  fn shader_type() -> Type { Type::TessellationControlShader }
}

/// Tessellation evaluation shader. An optional stage
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TessellationEvaluationShader;

impl ShaderTypeable for TessellationEvaluationShader {
  fn shader_type() -> Type { Type::TessellationEvaluationShader }
}

/// Vertex shader. A mandatory stage
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VertexShader;

impl ShaderTypeable for VertexShader {
  fn shader_type() -> Type { Type::VertexShader }
}

/// Geometry shader. An optional stage
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GeometryShader;

impl ShaderTypeable for GeometryShader {
  fn shader_type() -> Type { Type::GeometryShader }
}

/// Fragment shader. A mandatory stage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FragmentShader;

impl ShaderTypeable for FragmentShader {
  fn shader_type() -> Type { Type::FragmentShader }
}

/// A shader stage. The `T` type variable gives the type of the shader.
#[derive(Debug)]
pub struct Stage<C, T> where C: HasStage {
  pub repr: C::AStage,
  _t: PhantomData<T>
}

impl<C, T> Drop for Stage<C, T> where C: HasStage {
  fn drop(&mut self) {
    C::free_shader(&mut self.repr)
  }
}

impl<C, T> Stage<C, T> where C: HasStage, T: ShaderTypeable {
  pub fn new(src: &str) -> Result<Self, StageError> {
    let shader = C::new_shader(T::shader_type(), src);
    shader.map(|shader| Stage {
      repr: shader,
      _t: PhantomData
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
