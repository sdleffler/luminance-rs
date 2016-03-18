//! Shader uniforms and associated operations.
//!
//! Uniforms kick in several and useful ways. They’re used to customize shaders.
use core::marker::PhantomData;

pub trait HasUniform {
  /// Uniform representation.
  type U;

	fn update<T>(uniform: &Self::U, dim: Dim, value_type: Type, value: &T);
}

/// Dimension of a `Uniform`.
#[derive(Clone, Copy)]
pub enum Dim {
  DIM1,
  DIM2,
  DIM3,
  DIM4,
  DIM22,
  DIM33,
  DIM44
}

/// Type of a `Uniform`.
#[derive(Clone, Copy)]
pub enum Type {
  Integral,
  Unsigned,
  Floating,
  Boolean
}

/// A shader uniform. `Uniform<C, T>` doesn’t hold any value. It’s more like a mapping between the
/// host code and the shader the uniform was retrieved from.
pub struct Uniform<C, T> where C: HasUniform, T: Uniformable {
  pub repr: C::U,
  pub dim: Dim,
  pub value_type: Type,
  _t: PhantomData<T>
}

impl<C, T> Uniform<C, T> where C: HasUniform, T: Uniformable {
	pub fn update(&self, x: &T) {
		C::update(&self.repr, self.dim, self.value_type, x)
	}
}

/// Name of a `Uniform`.
pub enum UniformName<'a> {
  StringName(&'a str),
  SemanticName(u32)
}

/// Types that can behave as `Uniform`.
pub trait Uniformable {
  /// Dimension of the uniform type.
  fn uniform_dim() -> Dim;
  /// Type of the target value.
  fn value_type() -> Type;
  /// Number of target values the uniform handles.
  fn value_size(_: &Self) -> usize;
}

impl Uniformable for i32 {
  fn uniform_dim() -> Dim { Dim::DIM1 }

  fn value_type() -> Type { Type::Integral }

  fn value_size(_: &Self) -> usize { 1 }
}

impl Uniformable for (i32, i32) {
  fn uniform_dim() -> Dim { Dim::DIM2 }

  fn value_type() -> Type { Type::Integral }

  fn value_size(_: &Self) -> usize { 1 }
}

impl Uniformable for (i32, i32, i32) {
  fn uniform_dim() -> Dim { Dim::DIM3 }

  fn value_type() -> Type { Type::Integral }

  fn value_size(_: &Self) -> usize { 1 }
}

impl Uniformable for (i32, i32, i32, i32) {
  fn uniform_dim() -> Dim { Dim::DIM4 }

  fn value_type() -> Type { Type::Integral }

  fn value_size(_: &Self) -> usize { 1 }
}

impl Uniformable for u32 {
  fn uniform_dim() -> Dim { Dim::DIM1 }

  fn value_type() -> Type { Type::Unsigned }

  fn value_size(_: &Self) -> usize { 1 }
}

impl Uniformable for (u32, u32) {
  fn uniform_dim() -> Dim { Dim::DIM2 }

  fn value_type() -> Type { Type::Unsigned }

  fn value_size(_: &Self) -> usize { 1 }
}

impl Uniformable for (u32, u32, u32) {
  fn uniform_dim() -> Dim { Dim::DIM3 }

  fn value_type() -> Type { Type::Unsigned }

  fn value_size(_: &Self) -> usize { 1 }
}

impl Uniformable for (u32, u32, u32, u32) {
  fn uniform_dim() -> Dim { Dim::DIM4 }

  fn value_type() -> Type { Type::Unsigned }

  fn value_size(_: &Self) -> usize { 1 }
}

impl Uniformable for f32 {
  fn uniform_dim() -> Dim { Dim::DIM1 }

  fn value_type() -> Type { Type::Floating }

  fn value_size(_: &Self) -> usize { 1 }
}

impl Uniformable for (f32, f32) {
  fn uniform_dim() -> Dim { Dim::DIM2 }

  fn value_type() -> Type { Type::Floating }

  fn value_size(_: &Self) -> usize { 1 }
}

impl Uniformable for (f32, f32, f32) {
  fn uniform_dim() -> Dim { Dim::DIM3 }

  fn value_type() -> Type { Type::Floating }

  fn value_size(_: &Self) -> usize { 1 }
}

impl Uniformable for (f32, f32, f32, f32) {
  fn uniform_dim() -> Dim { Dim::DIM4 }

  fn value_type() -> Type { Type::Floating }

  fn value_size(_: &Self) -> usize { 1 }
}

impl Uniformable for ((f32, f32), (f32, f32)) {
  fn uniform_dim() -> Dim { Dim::DIM22 }

  fn value_type() -> Type { Type::Floating}

  fn value_size(_: &Self) -> usize { 1 }
}

impl Uniformable for ((f32, f32, f32), (f32, f32, f32), (f32, f32, f32)) {
  fn uniform_dim() -> Dim { Dim::DIM33 }

  fn value_type() -> Type { Type::Floating}

  fn value_size(_: &Self) -> usize { 1 }
}

impl Uniformable for ((f32, f32, f32, f32), (f32, f32, f32, f32), (f32, f32, f32, f32), (f32, f32, f32, f32)) {
  fn uniform_dim() -> Dim { Dim::DIM44 }

  fn value_type() -> Type { Type::Floating}

  fn value_size(_: &Self) -> usize { 1 }
}

impl Uniformable for bool {
  fn uniform_dim() -> Dim { Dim::DIM1 }

  fn value_type() -> Type { Type::Boolean }

  fn value_size(_: &Self) -> usize { 1 }
}

impl Uniformable for (bool, bool) {
  fn uniform_dim() -> Dim { Dim::DIM2 }

  fn value_type() -> Type { Type::Boolean }

  fn value_size(_: &Self) -> usize { 1 }
}

impl Uniformable for (bool, bool, bool) {
  fn uniform_dim() -> Dim { Dim::DIM3 }

  fn value_type() -> Type { Type::Boolean }

  fn value_size(_: &Self) -> usize { 1 }
}

impl Uniformable for (bool, bool, bool, bool) {
  fn uniform_dim() -> Dim { Dim::DIM4 }

  fn value_type() -> Type { Type::Boolean }

  fn value_size(_: &Self) -> usize { 1 }
}

impl<T> Uniformable for Vec<T> where T: Uniformable {
  fn uniform_dim() -> Dim { T::uniform_dim() }

  fn value_type() -> Type { T::value_type() }

  fn value_size(x: &Self) -> usize { x.len() }
}
