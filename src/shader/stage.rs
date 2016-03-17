use core::marker::PhantomData;

pub trait HasStage {
  type AStage;

  fn new<'a, 'b>(shader_type: Type, src: &'a str) -> Result<StageError<'b>, Self::AStage>;
}

pub trait Typeable {
  fn shader_type() -> Type;
}

/// A shader stage type.
pub enum Type {
  TessellationControlShader,
  TessellationEvaluationShader,
  VertexShader,
  GeometryShader,
  FragmentShader
}

pub struct TessellationControlShader;

impl Typeable for TessellationControlShader {
  fn shader_type() -> Type { Type::TessellationControlShader }
}

pub struct TessellationEvaluationShader;

impl Typeable for TessellationEvaluationShader {
  fn shader_type() -> Type { Type::TessellationEvaluationShader }
}

pub struct VertexShader;

impl Typeable for VertexShader {
  fn shader_type() -> Type { Type::VertexShader }
}

pub struct GeometryShader;

impl Typeable for GeometryShader {
  fn shader_type() -> Type { Type::GeometryShader }
}

pub struct FragmentShader;

impl Typeable for FragmentShader {
  fn shader_type() -> Type { Type::FragmentShader }
}

/// A shader stage. The `T` type variable gives the type of the shader.
pub struct Stage<C, T> where C: HasStage {
  pub repr: C::AStage,
  _c: PhantomData<C>,
  _t: PhantomData<T>
}

pub enum StageError<'a> {
  /// Occurs when a shader fails to compile.
  CompilationFailed(&'a str),
  /// Occurs when you try to create a shader which type is not supported on the current hardware.
  UnsupportedType(Type)
}
