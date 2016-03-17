use core::marker::PhantomData;

pub trait HasUniform {
  type U;
}

pub enum Dim {
  DIM1,
  DIM2,
  DIM3,
  DIM4,
  DIM22,
  DIM33,
  DIM44
}

pub enum Type {
  Integral,
  Unsigned,
  Floating,
  Boolean
}

pub struct Uniform<C, T> where C: HasUniform {
  pub repr: C::U,
  pub dim: Dim,
  pub value_type: Type,
  _t: PhantomData<T>
}

pub trait Uniformable {
  /// Dimension of the uniform type.
  fn uniform_dim() -> Dim;
  /// Type of the target value.
  fn value_type() -> Type;
  /// Number of target values the uniform handles.
  fn value_size() -> u32;
}

impl Uniformable for i32 {
  fn uniform_dim() -> Dim { Dim::DIM1 }

  fn value_type() -> Type { Type::Integral }

  fn value_size() -> u32 { 1 }
}

impl Uniformable for (i32, i32) {
  fn uniform_dim() -> Dim { Dim::DIM2 }

  fn value_type() -> Type { Type::Integral }

  fn value_size() -> u32 { 1 }
}

impl Uniformable for (i32, i32, i32) {
  fn uniform_dim() -> Dim { Dim::DIM3 }

  fn value_type() -> Type { Type::Integral }

  fn value_size() -> u32 { 1 }
}

impl Uniformable for (i32, i32, i32, i32) {
  fn uniform_dim() -> Dim { Dim::DIM4 }

  fn value_type() -> Type { Type::Integral }

  fn value_size() -> u32 { 1 }
}

impl Uniformable for u32 {
  fn uniform_dim() -> Dim { Dim::DIM1 }

  fn value_type() -> Type { Type::Unsigned }

  fn value_size() -> u32 { 1 }
}

impl Uniformable for (u32, u32) {
  fn uniform_dim() -> Dim { Dim::DIM2 }

  fn value_type() -> Type { Type::Unsigned }

  fn value_size() -> u32 { 1 }
}

impl Uniformable for (u32, u32, u32) {
  fn uniform_dim() -> Dim { Dim::DIM3 }

  fn value_type() -> Type { Type::Unsigned }

  fn value_size() -> u32 { 1 }
}

impl Uniformable for (u32, u32, u32, u32) {
  fn uniform_dim() -> Dim { Dim::DIM4 }

  fn value_type() -> Type { Type::Unsigned }

  fn value_size() -> u32 { 1 }
}

impl Uniformable for f32 {
  fn uniform_dim() -> Dim { Dim::DIM1 }

  fn value_type() -> Type { Type::Floating }

  fn value_size() -> u32 { 1 }
}

impl Uniformable for (f32, f32) {
  fn uniform_dim() -> Dim { Dim::DIM2 }

  fn value_type() -> Type { Type::Floating }

  fn value_size() -> u32 { 1 }
}

impl Uniformable for (f32, f32, f32) {
  fn uniform_dim() -> Dim { Dim::DIM3 }

  fn value_type() -> Type { Type::Floating }

  fn value_size() -> u32 { 1 }
}

impl Uniformable for (f32, f32, f32, f32) {
  fn uniform_dim() -> Dim { Dim::DIM4 }

  fn value_type() -> Type { Type::Floating }

  fn value_size() -> u32 { 1 }
}

impl Uniformable for ((f32, f32), (f32, f32)) {
  fn uniform_dim() -> Dim { Dim::DIM22 }

  fn value_type() -> Type { Type::Floating}

  fn value_size() -> u32 { 1 }
}

impl Uniformable for ((f32, f32, f32), (f32, f32, f32), (f32, f32, f32)) {
  fn uniform_dim() -> Dim { Dim::DIM33 }

  fn value_type() -> Type { Type::Floating}

  fn value_size() -> u32 { 1 }
}

impl Uniformable for ((f32, f32, f32, f32), (f32, f32, f32, f32), (f32, f32, f32, f32), (f32, f32, f32, f32)) {
  fn uniform_dim() -> Dim { Dim::DIM44 }

  fn value_type() -> Type { Type::Floating}

  fn value_size() -> u32 { 1 }
}

impl Uniformable for bool {
  fn uniform_dim() -> Dim { Dim::DIM1 }

  fn value_type() -> Type { Type::Boolean }

  fn value_size() -> u32 { 1 }
}

impl Uniformable for (bool, bool) {
  fn uniform_dim() -> Dim { Dim::DIM2 }

  fn value_type() -> Type { Type::Boolean }

  fn value_size() -> u32 { 1 }
}

impl Uniformable for (bool, bool, bool) {
  fn uniform_dim() -> Dim { Dim::DIM3 }

  fn value_type() -> Type { Type::Boolean }

  fn value_size() -> u32 { 1 }
}

impl Uniformable for (bool, bool, bool, bool) {
  fn uniform_dim() -> Dim { Dim::DIM4 }

  fn value_type() -> Type { Type::Boolean }

  fn value_size() -> u32 { 1 }
}
