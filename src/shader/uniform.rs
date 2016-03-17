use core::marker::PhantomData;

pub trait HasUniform {
  type U;
}

pub enum Dim {
  DIM1,
  DIM2,
  DIM3,
  DIM4
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
  fn uniform_dim() -> Dim;

  fn value_type() -> Type;
}

impl Uniformable for i32 {
  fn uniform_dim() -> Dim { Dim::DIM1 }

  fn value_type() -> Type { Type::Integral }
}

impl Uniformable for (i32, i32) {
  fn uniform_dim() -> Dim { Dim::DIM2 }

  fn value_type() -> Type { Type::Integral }
}

impl Uniformable for (i32, i32, i32) {
  fn uniform_dim() -> Dim { Dim::DIM3 }

  fn value_type() -> Type { Type::Integral }
}

impl Uniformable for (i32, i32, i32, i32) {
  fn uniform_dim() -> Dim { Dim::DIM4 }

  fn value_type() -> Type { Type::Integral }
}

impl Uniformable for u32 {
  fn uniform_dim() -> Dim { Dim::DIM1 }

  fn value_type() -> Type { Type::Unsigned }
}

impl Uniformable for (u32, u32) {
  fn uniform_dim() -> Dim { Dim::DIM2 }

  fn value_type() -> Type { Type::Unsigned }
}

impl Uniformable for (u32, u32, u32) {
  fn uniform_dim() -> Dim { Dim::DIM3 }

  fn value_type() -> Type { Type::Unsigned }
}

impl Uniformable for (u32, u32, u32, u32) {
  fn uniform_dim() -> Dim { Dim::DIM4 }

  fn value_type() -> Type { Type::Unsigned }
}

impl Uniformable for f32 {
  fn uniform_dim() -> Dim { Dim::DIM1 }

  fn value_type() -> Type { Type::Floating }
}

impl Uniformable for (f32, f32) {
  fn uniform_dim() -> Dim { Dim::DIM2 }

  fn value_type() -> Type { Type::Floating }
}

impl Uniformable for (f32, f32, f32) {
  fn uniform_dim() -> Dim { Dim::DIM3 }

  fn value_type() -> Type { Type::Floating }
}

impl Uniformable for (f32, f32, f32, f32) {
  fn uniform_dim() -> Dim { Dim::DIM4 }

  fn value_type() -> Type { Type::Floating }
}

impl Uniformable for bool {
  fn uniform_dim() -> Dim { Dim::DIM1 }

  fn value_type() -> Type { Type::Boolean }
}

impl Uniformable for (bool, bool) {
  fn uniform_dim() -> Dim { Dim::DIM2 }

  fn value_type() -> Type { Type::Boolean }
}

impl Uniformable for (bool, bool, bool) {
  fn uniform_dim() -> Dim { Dim::DIM3 }

  fn value_type() -> Type { Type::Boolean }
}

impl Uniformable for (bool, bool, bool, bool) {
  fn uniform_dim() -> Dim { Dim::DIM4 }

  fn value_type() -> Type { Type::Boolean }
}
