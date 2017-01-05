//! Shader programs related types and functions.
//!
//! A shader `Program` is an object representing several operations. It’s a streaming program that
//! will operate on vertices, vertex patches, primitives and/or fragments.
//!
//! > *Note: shader programs don’t have to run on all those objects; they can be ran only on
//! vertices and fragments, for instance*.
//!
//! Creating a shader program is very simple. You need shader `Stage`s representing each step of the
//! processing.
//!
//! You *have* to provide at least a vertex and fragment stages. If you want tessellation
//! processing, you need to provide a tessellation control and tessellation evaluation stages. If
//! you want primitives processing, you need to add a geometry stage.
//!
//! In order to customize the behavior of your shader programs, you have access to *uniforms*. For
//! more details about them, see the documentation for the type `Uniform` and trait `Uniformable`.
//! When creating a new shader program, you have to provide code to declare its *uniform interface*.
//! The *uniform interface* refers to a type of your own that will be kept by the shader program and
//! exposed to you when you’ll express the need to update its uniforms. That *uniform interface* is
//! created via a closure you pass. That closure takes as arguments a `ProgramProxy` used to
//! retrieve `Uniform`s from the program being constructed. That pattern, that can be a bit
//! overwhelming at first, is important to keep things safe and functional. Keep in mind that you
//! can make the closure fail, so that you can notify a `Uniform` lookup failure, for instance.
//!
//! You can create a `Program` with its `new` associated function.
//!
//! # Example
//!
//! TODO

use std::marker::PhantomData;

use buffer::Binding;
use linear::{M22, M33, M44};
use shader::stage::{HasStage, Stage};
use texture::Unit;

/// Trait to implement to provide shader program features.
pub trait HasProgram: HasStage {
  type Program;

  /// Create a new program by linking it with stages.
  ///
  /// The last argument – `sem_map` – is a mapping between your semantic and the names of the code
  /// variables they represent. You can generate them out of `Uniform`s with the `Uniform::sem`
  /// function.
  ///
  /// # Examples
  ///
  /// TODO
  fn new_program(tess: Option<(&Self::AStage, &Self::AStage)>, vertex: &Self::AStage, geometry: Option<&Self::AStage>, fragment: &Self::AStage, sem_map: &[Sem]) -> Result<(Self::Program, Vec<UniformWarning>), ProgramError>;
  /// Free a program.
  fn free_program(program: &mut Self::Program);
  /// Bulk update of uniforms. The update closure should contain the code used to update as many
  /// uniforms as wished.
  fn update_uniforms<F>(program: &Self::Program, f: F) where F: Fn();

  // uniform stuff

  // integral
  fn update1_i32(program: &Self::Program, uniform: SemIndex, x: i32);
  fn update2_i32(program: &Self::Program, uniform: SemIndex, xy: [i32; 2]);
  fn update3_i32(program: &Self::Program, uniform: SemIndex, xyz: [i32; 3]);
  fn update4_i32(program: &Self::Program, uniform: SemIndex, xyzw: [i32; 4]);
  fn update1_slice_i32(program: &Self::Program, uniform: SemIndex, x: &[i32]);
  fn update2_slice_i32(program: &Self::Program, uniform: SemIndex, xy: &[[i32; 2]]);
  fn update3_slice_i32(program: &Self::Program, uniform: SemIndex, xyz: &[[i32; 3]]);
  fn update4_slice_i32(program: &Self::Program, uniform: SemIndex, xyzw: &[[i32; 4]]);
  // unsigned
  fn update1_u32(program: &Self::Program, uniform: SemIndex, x: u32);
  fn update2_u32(program: &Self::Program, uniform: SemIndex, xy: [u32; 2]);
  fn update3_u32(program: &Self::Program, uniform: SemIndex, xyz: [u32; 3]);
  fn update4_u32(program: &Self::Program, uniform: SemIndex, xyzw: [u32; 4]);
  fn update1_slice_u32(program: &Self::Program, uniform: SemIndex, x: &[u32]);
  fn update2_slice_u32(program: &Self::Program, uniform: SemIndex, xy: &[[u32; 2]]);
  fn update3_slice_u32(program: &Self::Program, uniform: SemIndex, xyz: &[[u32; 3]]);
  fn update4_slice_u32(program: &Self::Program, uniform: SemIndex, xyzw: &[[u32; 4]]);
  // floating
  fn update1_f32(program: &Self::Program, uniform: SemIndex, x: f32);
  fn update2_f32(program: &Self::Program, uniform: SemIndex, xy: [f32; 2]);
  fn update3_f32(program: &Self::Program, uniform: SemIndex, xyz: [f32; 3]);
  fn update4_f32(program: &Self::Program, uniform: SemIndex, xyzw: [f32; 4]);
  fn update1_slice_f32(program: &Self::Program, uniform: SemIndex, x: &[f32]);
  fn update2_slice_f32(program: &Self::Program, uniform: SemIndex, xy: &[[f32; 2]]);
  fn update3_slice_f32(program: &Self::Program, uniform: SemIndex, xyz: &[[f32; 3]]);
  fn update4_slice_f32(program: &Self::Program, uniform: SemIndex, xyzw: &[[f32; 4]]);
  fn update22_f32(program: &Self::Program, uniform: SemIndex, x: M22);
  fn update33_f32(program: &Self::Program, uniform: SemIndex, x: M33);
  fn update44_f32(program: &Self::Program, uniform: SemIndex, x: M44);
  fn update22_slice_f32(program: &Self::Program, uniform: SemIndex, x: &[M22]);
  fn update33_slice_f32(program: &Self::Program, uniform: SemIndex, x: &[M33]);
  fn update44_slice_f32(program: &Self::Program, uniform: SemIndex, x: &[M44]);
  // boolean
  fn update1_bool(program: &Self::Program, uniform: SemIndex, x: bool);
  fn update2_bool(program: &Self::Program, uniform: SemIndex, xy: [bool; 2]);
  fn update3_bool(program: &Self::Program, uniform: SemIndex, xyz: [bool; 3]);
  fn update4_bool(program: &Self::Program, uniform: SemIndex, xyzw: [bool; 4]);
  fn update1_slice_bool(program: &Self::Program, uniform: SemIndex, x: &[bool]);
  fn update2_slice_bool(program: &Self::Program, uniform: SemIndex, xy: &[[bool; 2]]);
  fn update3_slice_bool(program: &Self::Program, uniform: SemIndex, xyz: &[[bool; 3]]);
  fn update4_slice_bool(program: &Self::Program, uniform: SemIndex, xyzw: &[[bool; 4]]);
  // textures
  fn update_texture_unit(program: &Self::Program, uniform: SemIndex, unit: u32);
  // uniform buffers
  fn update_buffer_binding(program: &Self::Program, uniform_block: SemIndex, binding: u32);
}

/// A shader program.
#[derive(Debug)]
pub struct Program<C>(pub C::Program) where C: HasProgram;

impl<C> Drop for Program<C> where C: HasProgram {
  fn drop(&mut self) {
    C::free_program(&mut self.0)
  }
}

impl<C> Program<C> where C: HasProgram {
  /// Create a new `Program` by linking it with shader stages.
  pub fn new(tess: Option<(&Stage<C>, &Stage<C>)>, vertex: &Stage<C>, geometry: Option<&Stage<C>>, fragment: &Stage<C>, sem_map: &[Sem]) -> Result<(Self, Vec<UniformWarning>), ProgramError> {
    let (repr, warnings) = C::new_program(tess.map(|(tcs, tes)| (&tcs.repr, &tes.repr)), &vertex.repr, geometry.map(|g| &g.repr), &fragment.repr, sem_map)?;

    Ok((Program(repr), warnings))
  }

  /// Update a uniform variable in the program.
  pub fn update<T>(&self, u: &Uniform<C, T>, value: T) where T: Uniformable<C> {
    value.update(&self.0, u);
  }
}

/// A shader uniform semantic. It holds information on the variable it represents such as a name,
/// its type and its dimension.
#[derive(Clone, Debug)]
pub struct Sem {
  name: String,
  index: SemIndex,
  ty: Type,
  dim: Dim
}

impl Sem {
  pub fn new(name: &str, index: SemIndex, ty: Type, dim: Dim) -> Self {
    Sem {
      name: name.to_owned(),
      index: index,
      ty: ty,
      dim: dim
    }
  }

  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn ty(&self) -> Type {
    self.ty
  }

  pub fn dim(&self) -> Dim {
    self.dim
  }
}

/// Semantic index.
pub type SemIndex = u32;

/// Errors that a `Program` can generate.
#[derive(Clone, Debug)]
pub enum ProgramError {
  /// Program link failed. You can inspect the reason by looking at the contained `String`.
  LinkFailed(String),
  /// Some uniform configuration is ill-formed. It can be a problem of inactive uniform, mismatch
  /// type, etc. Check the `UniformWarning` type for more information.
  UniformWarning(UniformWarning)
}

/// Warnings related to uniform issues.
#[derive(Clone, Debug)]
pub enum UniformWarning {
  /// Inactive uniform (not in use / no participation to the final output in shaders).
  Inactive(String),
  /// Type mismatch between the static requested type (i.e. the `T` in `Uniform<T>` for instance)
  /// and the type that got reflected from the backend in the shaders.
  ///
  /// The first `String` is the name of the uniform; the second one gives the type mismatch.
  TypeMismatch(String, String)
}

/// A shader uniform. `Uniform<T>` doesn’t hold any value. It’s more like a mapping between the
/// host code and the shader the uniform was retrieved from.
#[derive(Debug)]
pub struct Uniform<C, T> where C: HasProgram, T: Uniformable<C> {
  pub sem_index: SemIndex,
  _c: PhantomData<*const C>,
  _t: PhantomData<*const T>
}

impl<C, T> Uniform<C, T> where C: HasProgram, T: Uniformable<C> {
  pub const fn new(sem_index: SemIndex) -> Uniform<C, T> {
    Uniform {
      sem_index: sem_index,
      _c: PhantomData,
      _t: PhantomData
    }
  }

  /// Create a `Sem` by giving a mapping name. The `Type` and `Dim` are reified using the static
  /// type of the uniform (`T`).
  pub fn sem(&self, name: &str) -> Sem {
    Sem::new(name, self.sem_index, T::reify_type(), T::dim())
  }
}

/// Type of a uniform.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Type {
  Integral,
  Unsigned,
  Floating,
  Boolean,
  TextureUnit,
  BufferBinding
}

/// Dimension of the uniform.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Dim {
  Dim1,
  Dim2,
  Dim3,
  Dim4,
  Dim22,
  Dim33,
  Dim44,
  Cubemap
}

/// Types that can behave as `Uniform`.
pub trait Uniformable<C>: Sized where C: HasProgram {
  /// Update the uniform with a new value.
  fn update(self, program: &C::Program, u: &Uniform<C, Self>);
  /// Retrieve the `Type` of the uniform.
  fn reify_type() -> Type; // FIXME: call that ty() instead
  /// Retrieve the `Dim` of the uniform.
  fn dim() -> Dim;
}

impl<C> Uniformable<C> for i32 where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update1_i32(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim1 }
}

impl<C> Uniformable<C> for [i32; 2] where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update2_i32(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim2 }
}

impl<C> Uniformable<C> for [i32; 3] where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update3_i32(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim3 }
}

impl<C> Uniformable<C> for [i32; 4] where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update4_i32(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim4 }
}

impl<'a, C> Uniformable<C> for &'a [i32] where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update1_slice_i32(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim1 }
}

impl<'a, C> Uniformable<C> for &'a [[i32; 2]] where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update2_slice_i32(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim2 }
}

impl<'a, C> Uniformable<C> for &'a [[i32; 3]] where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update3_slice_i32(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim3 }
}

impl<'a, C> Uniformable<C> for &'a [[i32; 4]] where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update4_slice_i32(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim4 }
}

impl<C> Uniformable<C> for u32 where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update1_u32(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim1 }
}

impl<C> Uniformable<C> for [u32; 2] where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update2_u32(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim2 }
}

impl<C> Uniformable<C> for [u32; 3] where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update3_u32(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim3 }
}

impl<C> Uniformable<C> for [u32; 4] where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update4_u32(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim4 }
}

impl<'a, C> Uniformable<C> for &'a [u32] where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update1_slice_u32(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim1 }
}

impl<'a, C> Uniformable<C> for &'a [[u32; 2]] where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update2_slice_u32(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim2 }
}

impl<'a, C> Uniformable<C> for &'a [[u32; 3]] where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update3_slice_u32(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim3 }
}

impl<'a, C> Uniformable<C> for &'a [[u32; 4]] where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update4_slice_u32(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim4 }
}

impl<C> Uniformable<C> for f32 where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update1_f32(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim1 }
}

impl<C> Uniformable<C> for [f32; 2] where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update2_f32(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim2 }
}

impl<C> Uniformable<C> for [f32; 3] where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update3_f32(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim3 }
}

impl<C> Uniformable<C> for [f32; 4] where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update4_f32(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim4 }
}

impl<'a, C> Uniformable<C> for &'a [f32] where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update1_slice_f32(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim1 }
}

impl<'a, C> Uniformable<C> for &'a [[f32; 2]] where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update2_slice_f32(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim2 }
}

impl<'a, C> Uniformable<C> for &'a [[f32; 3]] where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update3_slice_f32(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim3 }
}

impl<'a, C> Uniformable<C> for &'a [[f32; 4]] where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update4_slice_f32(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim4 }
}

impl<C> Uniformable<C> for M22 where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update22_f32(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim22 }
}

impl<C> Uniformable<C> for M33 where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update33_f32(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim33 }
}

impl<C> Uniformable<C> for M44 where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update44_f32(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim44 }
}

impl<'a, C> Uniformable<C> for &'a [M22] where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update22_slice_f32(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim22 }
}

impl<'a, C> Uniformable<C> for &'a [M33] where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update33_slice_f32(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim33 }
}

impl<'a, C> Uniformable<C> for &'a [M44] where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update44_slice_f32(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim44 }
}

impl<C> Uniformable<C> for bool where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update1_bool(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim1 }
}

impl<C> Uniformable<C> for [bool; 2] where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update2_bool(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim2 }
}

impl<C> Uniformable<C> for [bool; 3] where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update3_bool(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim3 }
}

impl<C> Uniformable<C> for [bool; 4] where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update4_bool(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim4 }
}

impl<'a, C> Uniformable<C> for &'a [bool] where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update1_slice_bool(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim1 }
}

impl<'a, C> Uniformable<C> for &'a [[bool; 2]] where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update2_slice_bool(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim2 }
}

impl<'a, C> Uniformable<C> for &'a [[bool; 3]] where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update3_slice_bool(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim3 }
}

impl<'a, C> Uniformable<C> for &'a [[bool; 4]] where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update4_slice_bool(program, u.sem_index, self)
  }

  fn reify_type() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim4 }
}

impl<C> Uniformable<C> for Unit where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update_texture_unit(program, u.sem_index, *self);
  }

  fn reify_type() -> Type { Type::TextureUnit }

  fn dim() -> Dim { Dim::Dim1 }
}

impl<C> Uniformable<C> for Binding where C: HasProgram {
  fn update(self, program: &C::Program, u: &Uniform<C, Self>) {
    C::update_buffer_binding(program, u.sem_index, *self);
  }

  fn reify_type() -> Type { Type::BufferBinding }

  fn dim() -> Dim { Dim::Dim1 }
}
