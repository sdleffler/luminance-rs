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
pub trait HasProgram: HasStage + HasUniform {
  type Program;

  /// Create a new program by linking it with stages.
  fn new_program(tess: Option<(&Self::AStage, &Self::AStage)>, vertex: &Self::AStage, geometry: Option<&Self::AStage>, fragment: &Self::AStage) -> Result<Self::Program, ProgramError>;
  /// Free a program.
  fn free_program(program: &mut Self::Program);
  /// Map a uniform name to its uniform representation. Can fail with `ProgramError`.
  fn map_uniform(program: &Self::Program, name: &str, ty: Type, dim: Dim) -> (Self::U, Option<UniformWarning>);
  /// Bulk update of uniforms. The update closure should contain the code used to update as many
  /// uniforms as wished.
  fn update_uniforms<F>(program: &Self::Program, f: F) where F: Fn();
}

/// A shader program with *uniform interface*.
#[derive(Debug)]
pub struct Program<C, T> where C: HasProgram {
  pub repr: C::Program,
  pub uniform_interface: T
}

impl<C, T> Drop for Program<C, T> where C: HasProgram {
  fn drop(&mut self) {
    C::free_program(&mut self.repr)
  }
}

impl<C, T> Program<C, T> where C: HasProgram {
  /// Create a new `Program` by linking it with shader stages and by providing a function to build
  /// its *uniform interface*, which the `Program` will hold for you.
  ///
  /// The *uniform interface* is any type you want. The idea is to bake `Uniform<_>` in your type so
  /// that you can access them later. To do so, you’re given an object of type `ProgramProxy`, which
  /// has a function `uniform`. That function can be used to lookup uniforms so that you can build
  /// your *uniform interface*.
  ///
  /// Use the `update` function to access the *uniform interface* back.
  pub fn new<GetUni>(tess: Option<(&Stage<C>, &Stage<C>)>, vertex: &Stage<C>, geometry: Option<&Stage<C>>, fragment: &Stage<C>, get_uni: GetUni) -> Result<Self, ProgramError>
      where GetUni: Fn(ProgramProxy<C>) -> Result<T, UniformWarning> {
    let repr = try!(C::new_program(tess.map(|(tcs, tes)| (&tcs.repr, &tes.repr)), &vertex.repr, geometry.map(|g| &g.repr), &fragment.repr));
    let uniform_interface = try!(get_uni(ProgramProxy::new(&repr)).map_err(ProgramError::UniformWarning));

    Ok(Program {
      repr: repr,
      uniform_interface: uniform_interface
    })
  }

  /// Uniform bulk update.
  ///
  /// Access the uniform interface and update uniforms.
  pub fn update<F>(&self, f: F) where F: Fn(&T) {
    C::update_uniforms(&self.repr, || { f(&self.uniform_interface) })
  }
}

/// `Program` proxy used to map uniforms. When building a `Program`, as the `Program` doesn’t exist
/// yet, a `ProgramProxy` is passed to act as it was the `Program`.
#[derive(Debug)]
pub struct ProgramProxy<'a, C> where C: 'a + HasProgram {
  repr: &'a C::Program
}

impl<'a, C> ProgramProxy<'a, C> where C: HasProgram {
  fn new(program: &'a C::Program) -> Self {
    ProgramProxy {
      repr: program
    }
  }

  /// Map a uniform name to a uniformable value.
  pub fn uniform<T>(&self, name: &str) -> (Uniform<C, T>, Option<UniformWarning>) where T: Uniformable<C> {
    // gather information about the uniform so that backend can proceed with runtime reification and
    // validation
    let ty = T::reify_type();
    let dim = T::dim();

    let (u, w) = C::map_uniform(self.repr, name, ty, dim);
    (Uniform::new(u), w)
  }
}

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
  TypeMismatch(String)
}

// ---------------------------------------
// -- Uniforms ---------------------------

/// Implement that trait to expose the *uniform* concept and be able to use types like `Uniform<_>`.
pub trait HasUniform {
  /// Uniform representation.
  type U;

  // integral
  fn update1_i32(uniform: &Self::U, x: i32);
  fn update2_i32(uniform: &Self::U, xy: [i32; 2]);
  fn update3_i32(uniform: &Self::U, xyz: [i32; 3]);
  fn update4_i32(uniform: &Self::U, xyzw: [i32; 4]);
  fn update1_slice_i32(uniform: &Self::U, x: &[i32]);
  fn update2_slice_i32(uniform: &Self::U, xy: &[[i32; 2]]);
  fn update3_slice_i32(uniform: &Self::U, xyz: &[[i32; 3]]);
  fn update4_slice_i32(uniform: &Self::U, xyzw: &[[i32; 4]]);
  // unsigned
  fn update1_u32(uniform: &Self::U, x: u32);
  fn update2_u32(uniform: &Self::U, xy: [u32; 2]);
  fn update3_u32(uniform: &Self::U, xyz: [u32; 3]);
  fn update4_u32(uniform: &Self::U, xyzw: [u32; 4]);
  fn update1_slice_u32(uniform: &Self::U, x: &[u32]);
  fn update2_slice_u32(uniform: &Self::U, xy: &[[u32; 2]]);
  fn update3_slice_u32(uniform: &Self::U, xyz: &[[u32; 3]]);
  fn update4_slice_u32(uniform: &Self::U, xyzw: &[[u32; 4]]);
  // floating
  fn update1_f32(uniform: &Self::U, x: f32);
  fn update2_f32(uniform: &Self::U, xy: [f32; 2]);
  fn update3_f32(uniform: &Self::U, xyz: [f32; 3]);
  fn update4_f32(uniform: &Self::U, xyzw: [f32; 4]);
  fn update1_slice_f32(uniform: &Self::U, x: &[f32]);
  fn update2_slice_f32(uniform: &Self::U, xy: &[[f32; 2]]);
  fn update3_slice_f32(uniform: &Self::U, xyz: &[[f32; 3]]);
  fn update4_slice_f32(uniform: &Self::U, xyzw: &[[f32; 4]]);
  fn update22_f32(uniform: &Self::U, x: M22);
  fn update33_f32(uniform: &Self::U, x: M33);
  fn update44_f32(uniform: &Self::U, x: M44);
  fn update22_slice_f32(uniform: &Self::U, x: &[M22]);
  fn update33_slice_f32(uniform: &Self::U, x: &[M33]);
  fn update44_slice_f32(uniform: &Self::U, x: &[M44]);
  // boolean
  fn update1_bool(uniform: &Self::U, x: bool);
  fn update2_bool(uniform: &Self::U, xy: [bool; 2]);
  fn update3_bool(uniform: &Self::U, xyz: [bool; 3]);
  fn update4_bool(uniform: &Self::U, xyzw: [bool; 4]);
  fn update1_slice_bool(uniform: &Self::U, x: &[bool]);
  fn update2_slice_bool(uniform: &Self::U, xy: &[[bool; 2]]);
  fn update3_slice_bool(uniform: &Self::U, xyz: &[[bool; 3]]);
  fn update4_slice_bool(uniform: &Self::U, xyzw: &[[bool; 4]]);
  // textures
  fn update_texture_unit(uniform: &Self::U, unit: u32);
  // uniform buffers
  fn update_buffer_binding(uniform_block: &Self::U, binding: u32);
}

/// A shader uniform. `Uniform<C, T>` doesn’t hold any value. It’s more like a mapping between the
/// host code and the shader the uniform was retrieved from.
#[derive(Debug)]
pub struct Uniform<C, T> where C: HasUniform, T: Uniformable<C> {
  pub repr: C::U,
  _t: PhantomData<*const T>
}

impl<C, T> Uniform<C, T> where C: HasUniform, T: Uniformable<C> {
  pub fn new(repr: C::U) -> Uniform<C, T> {
    Uniform {
      repr: repr,
      _t: PhantomData
    }
  }

  /// Value update.
  pub fn update(&self, x: T) {
    T::update(self, x);
  }
}

/// Wrapper over `Uniform`, discarding everything but update.
///
/// Among its features, this type enables you to `contramap` a function to build more interesting
/// `UniformUpdate`.
///
/// Use `From` or `Into` to build a `UniformUpdate`.
pub struct UniformUpdate<'a, T> {
  update_closure: Box<Fn(T) + 'a>
}

impl<'a, C, T> From<Uniform<C, T>> for UniformUpdate<'a, T> where C: 'a + HasUniform, T: 'a + Uniformable<C> {
  fn from(u: Uniform<C, T>) -> Self {
    UniformUpdate {
      update_closure: Box::new(move |x| {
        u.update(x);
      })
    }
  }
}

impl<'a, T> UniformUpdate<'a, T> where T: 'a {
  /// Update the underlying `Uniform`.
  pub fn update(&self, x: T) {
    (self.update_closure)(x)
  }

  /// Apply a contravariant functor.
  pub fn contramap<F, Q>(self, f: F) -> UniformUpdate<'a, Q> where F: 'a + Fn(Q) -> T {
    UniformUpdate {
      update_closure: Box::new(move |x| {
        (self.update_closure)(f(x))
      })
    }
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
pub trait Uniformable<C>: Sized where C: HasUniform {
  /// Update the uniform with a new value.
  fn update(u: &Uniform<C, Self>, x: Self);
  /// Retrieve the `Type` of the uniform.
  fn reify_type() -> Type;
  /// Retrieve the `Dim` of the uniform.
  fn dim() -> Dim;
}

impl<C> Uniformable<C> for i32 where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update1_i32(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim1 }
}

impl<C> Uniformable<C> for [i32; 2] where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update2_i32(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim2 }
}

impl<C> Uniformable<C> for [i32; 3] where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update3_i32(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim3 }
}

impl<C> Uniformable<C> for [i32; 4] where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update4_i32(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim4 }
}

impl<'a, C> Uniformable<C> for &'a [i32] where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update1_slice_i32(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim1 }
}

impl<'a, C> Uniformable<C> for &'a [[i32; 2]] where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update2_slice_i32(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim2 }
}

impl<'a, C> Uniformable<C> for &'a [[i32; 3]] where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update3_slice_i32(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim3 }
}

impl<'a, C> Uniformable<C> for &'a [[i32; 4]] where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update4_slice_i32(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim4 }
}

impl<C> Uniformable<C> for u32 where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update1_u32(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim1 }
}

impl<C> Uniformable<C> for [u32; 2] where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update2_u32(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim2 }
}

impl<C> Uniformable<C> for [u32; 3] where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update3_u32(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim3 }
}

impl<C> Uniformable<C> for [u32; 4] where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update4_u32(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim4 }
}

impl<'a, C> Uniformable<C> for &'a [u32] where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update1_slice_u32(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim1 }
}

impl<'a, C> Uniformable<C> for &'a [[u32; 2]] where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update2_slice_u32(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim2 }
}

impl<'a, C> Uniformable<C> for &'a [[u32; 3]] where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update3_slice_u32(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim3 }
}

impl<'a, C> Uniformable<C> for &'a [[u32; 4]] where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update4_slice_u32(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim4 }
}

impl<C> Uniformable<C> for f32 where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update1_f32(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim1 }
}

impl<C> Uniformable<C> for [f32; 2] where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update2_f32(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim2 }
}

impl<C> Uniformable<C> for [f32; 3] where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update3_f32(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim3 }
}

impl<C> Uniformable<C> for [f32; 4] where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update4_f32(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim4 }
}

impl<'a, C> Uniformable<C> for &'a [f32] where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update1_slice_f32(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim1 }
}

impl<'a, C> Uniformable<C> for &'a [[f32; 2]] where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update2_slice_f32(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim2 }
}

impl<'a, C> Uniformable<C> for &'a [[f32; 3]] where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update3_slice_f32(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim3 }
}

impl<'a, C> Uniformable<C> for &'a [[f32; 4]] where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update4_slice_f32(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim4 }
}

impl<C> Uniformable<C> for M22 where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update22_f32(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim22 }
}

impl<C> Uniformable<C> for M33 where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update33_f32(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim33 }
}

impl<C> Uniformable<C> for M44 where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update44_f32(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim44 }
}

impl<'a, C> Uniformable<C> for &'a [M22] where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update22_slice_f32(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim22 }
}

impl<'a, C> Uniformable<C> for &'a [M33] where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update33_slice_f32(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim33 }
}

impl<'a, C> Uniformable<C> for &'a [M44] where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update44_slice_f32(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim44 }
}

impl<C> Uniformable<C> for bool where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update1_bool(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim1 }
}

impl<C> Uniformable<C> for [bool; 2] where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update2_bool(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim2 }
}

impl<C> Uniformable<C> for [bool; 3] where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update3_bool(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim3 }
}

impl<C> Uniformable<C> for [bool; 4] where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update4_bool(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim4 }
}

impl<'a, C> Uniformable<C> for &'a [bool] where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update1_slice_bool(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim1 }
}

impl<'a, C> Uniformable<C> for &'a [[bool; 2]] where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update2_slice_bool(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim2 }
}

impl<'a, C> Uniformable<C> for &'a [[bool; 3]] where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update3_slice_bool(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim3 }
}

impl<'a, C> Uniformable<C> for &'a [[bool; 4]] where C: HasUniform {
  fn update(u: &Uniform<C, Self>, x: Self) {
    C::update4_slice_bool(&u.repr, x)
  }

  fn reify_type() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim4 }
}

impl<C> Uniformable<C> for Unit where C: HasUniform {
  fn update(u: &Uniform<C, Self>, unit: Self) {
    C::update_texture_unit(&u.repr, *unit);
  }

  fn reify_type() -> Type { Type::TextureUnit }

  fn dim() -> Dim { Dim::Dim1 }
}

impl<C> Uniformable<C> for Binding where C: HasUniform {
  fn update(u: &Uniform<C, Self>, binding: Self) {
    C::update_buffer_binding(&u.repr, *binding);
  }

  fn reify_type() -> Type { Type::BufferBinding }

  fn dim() -> Dim { Dim::Dim1 }
}
