use core::marker::PhantomData;

pub trait HasStage {
  type AStage;

  fn new_shader(shader_type: Type, src: &str) -> Result<Self::AStage, StageError>;
}

pub trait ShaderTypeable {
  fn shader_type() -> Type;
}

/// A shader stage type.
#[derive(Debug)]
pub enum Type {
  TessellationControlShader,
  TessellationEvaluationShader,
  VertexShader,
  GeometryShader,
  FragmentShader
}

#[derive(Debug)]
pub struct TessellationControlShader;

impl ShaderTypeable for TessellationControlShader {
  fn shader_type() -> Type { Type::TessellationControlShader }
}

#[derive(Debug)]
pub struct TessellationEvaluationShader;

impl ShaderTypeable for TessellationEvaluationShader {
  fn shader_type() -> Type { Type::TessellationEvaluationShader }
}

#[derive(Debug)]
pub struct VertexShader;

impl ShaderTypeable for VertexShader {
  fn shader_type() -> Type { Type::VertexShader }
}

#[derive(Debug)]
pub struct GeometryShader;

impl ShaderTypeable for GeometryShader {
  fn shader_type() -> Type { Type::GeometryShader }
}

#[derive(Debug)]
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

impl<C, T> Stage<C, T> where C: HasStage, T: ShaderTypeable {
  pub fn new(_: T, src: &str) -> Result<Self, StageError> {
    let shader = C::new_shader(T::shader_type(), src);
    shader.map(|shader| Stage {
      repr: shader,
      _t: PhantomData
    })
  }
}

#[derive(Debug)]
pub enum StageError {
  /// Occurs when a shader fails to compile.
  CompilationFailed(String),
  /// Occurs when you try to create a shader which type is not supported on the current hardware.
  UnsupportedType(Type)
}
