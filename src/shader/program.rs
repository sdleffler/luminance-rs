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
//! You *have* to provide at least a vertex and a fragment stages. If you want tessellation
//! processing, you need to provide a tessellation control and tessellation evaluation stages. If
//! you want primitives processing, you need to add a geometry stage.
//!
//! In order to customize the behavior of your shader programs, you have access to *uniforms*. For
//! more details about them, see the documentation for the type `Uniform` and trait `Uniformable`.
//! When creating a new shader program, you have to provide code to declare its *uniform semantics*.
//!
//! The *uniform semantics* represent a mapping between the variables declared in your shader
//! sources and variables you have access in your host code in Rust. Typically, you declare your
//! variable – `Uniform` – in Rust as `const` and use the function `Uniform::sem` to get the
//! semantic associated with the string you pass in.
//!
//! > **Becareful: currently, uniforms are a bit messy as you have to provide a per-program unique
//! number when you use the `Uniform::new` method. Efforts will be done in that direction in later
//! releases.
//!
//! You can create a `Program` with its `new` associated function.

use gl;
use gl::types::*;
use std::ffi::CString;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::null_mut;

use buffer::Binding;
use linear::{M22, M33, M44};
use shader::stage::Stage;
use texture::Unit;
use vertex::Vertex;

type Result<A> = ::std::result::Result<A, ProgramError>;

/// A shader program.
#[derive(Debug)]
pub struct RawProgram {
  handle: GLuint
}

impl RawProgram {
  /// Create a new program by linking shader stages.
  pub fn new<'a, T, G>(tess: T,
                       vertex: &Stage,
                       geometry: G,
                       fragment: &Stage)
                       -> Result<Self>
      where T: Into<Option<(&'a Stage, &'a Stage)>>,
            G: Into<Option<&'a Stage>> {
    unsafe {
      let handle = gl::CreateProgram();

      if let Some((tcs, tes)) = tess.into() {
        gl::AttachShader(handle, tcs.handle());
        gl::AttachShader(handle, tes.handle());
      }

      gl::AttachShader(handle, vertex.handle());

      if let Some(geometry) = geometry.into() {
        gl::AttachShader(handle, geometry.handle());
      }

      gl::AttachShader(handle, fragment.handle());

      gl::LinkProgram(handle);

      let mut linked: GLint = gl::FALSE as GLint;
      gl::GetProgramiv(handle, gl::LINK_STATUS, &mut linked);

      if linked == (gl::TRUE as GLint) {
        Ok(RawProgram { handle: handle })
      } else {
        let mut log_len: GLint = 0;
        gl::GetProgramiv(handle, gl::INFO_LOG_LENGTH, &mut log_len);

        let mut log: Vec<u8> = Vec::with_capacity(log_len as usize);
        gl::GetProgramInfoLog(handle, log_len, null_mut(), log.as_mut_ptr() as *mut GLchar);

        gl::DeleteProgram(handle);

        log.set_len(log_len as usize);

        Err(ProgramError::LinkFailed(String::from_utf8(log).unwrap()))
      }
    }
  }

  #[inline]
  pub unsafe fn handle(&self) -> GLuint {
    self.handle
  }
}

impl Drop for RawProgram {
  fn drop(&mut self) {
    unsafe { gl::DeleteProgram(self.handle) }
  }
}

/// A typed shader program.
///
/// Typed shader programs represent their inputs, outputs and environment (uniforms) directly in
/// their types. This is very interesting as it adds more static safety and enables such programs
/// to *“store”* information like uniform variables and such.
pub struct Program<In, Out, Uni> {
  raw: RawProgram,
  uni_iface: Uni,
  _in: PhantomData<*const In>,
  _out: PhantomData<*const Out>
}

impl<In, Out, Uni> Program<In, Out, Uni> where In: Vertex, Uni: UniformInterface {
  pub fn new<'a, T, G>(tess: T,
                       vertex: &Stage,
                       geometry: G,
                       fragment: &Stage)
                       -> Result<(Self, Vec<UniformWarning>)>
      where T: Into<Option<(&'a Stage, &'a Stage)>>,
            G: Into<Option<&'a Stage>> {
    let raw = RawProgram::new(tess, vertex, geometry, fragment)?;
    let (iface, warnings) = Uni::uniform_interface(UniformBuilder::new(&raw))?;

    let program = Program {
      raw: raw,
      uni_iface: iface,
      _in: PhantomData,
      _out: PhantomData
    };

    Ok((program, warnings))
  }

  // TODO: hide that from the public interface when pub(crate) is available
  /// Get the uniform interface associated with this program.
  ///
  /// > Note: please do not use that function as it’s unsafe and for internal use only.
  pub unsafe fn uniform_interface(&self) -> &Uni {
    &self.uni_iface
  }
}

impl<In, Out, Uni> Deref for Program<In, Out, Uni> {
  type Target = RawProgram;

  fn deref(&self) -> &Self::Target {
    &self.raw
  }
}

pub trait UniformInterface: Sized {
  /// Build the uniform interface.
  ///
  /// When mapping a uniform, if you want to accept failures, you can discard the error and use
  /// `UniformBuilder::unbound` to let the uniform pass through, and collect the uniform warning.
  fn uniform_interface<'a>(builder: UniformBuilder<'a>) -> Result<(Self, Vec<UniformWarning>)>;
}

pub struct UniformBuilder<'a> {
  raw: &'a RawProgram
}

impl<'a> UniformBuilder<'a> {
  fn new(raw: &'a RawProgram) -> Self {
    UniformBuilder {
      raw: raw
    }
  }

  pub fn ask<T>(&self, name: &str) -> ::std::result::Result<Uniform<T>, UniformWarning> where T: Uniformable {
    let uniform = match T::ty() {
      Type::BufferBinding => self.ask_uniform_block(name)?,
      _ => self.ask_uniform(name)?
    };

    if let Some(err) = uniform_type_match(self.raw.handle, name, T::ty(), T::dim()) {
      Err(UniformWarning::TypeMismatch(name.to_owned(), err))
    } else {
      Ok(uniform)
    }
  }

  fn ask_uniform<T>(&self, name: &str) -> ::std::result::Result<Uniform<T>, UniformWarning> where T: Uniformable {
    let c_name = CString::new(name.as_bytes()).unwrap();
    let location = unsafe { gl::GetUniformLocation(self.raw.handle, c_name.as_ptr() as *const GLchar) };

    if location < 0 {
      Err(UniformWarning::Inactive(name.to_owned()))
    } else {
      Ok(Uniform::new(self.raw.handle, location))
    }
  }

  fn ask_uniform_block<T>(&self, name: &str) -> ::std::result::Result<Uniform<T>, UniformWarning> where T: Uniformable {
    let c_name = CString::new(name.as_bytes()).unwrap();
    let location = unsafe { gl::GetUniformBlockIndex(self.raw.handle, c_name.as_ptr() as *const GLchar) };

    if location == gl::INVALID_INDEX {
      Err(UniformWarning::Inactive(name.to_owned()))
    } else {
      Ok(Uniform::new(self.raw.handle, location as GLint))
    }
  }

  pub fn unbound<T>(&self) -> Uniform<T> where T: Uniformable {
    Uniform::unbound(self.raw.handle)
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
  ///
  /// The first `String` is the name of the uniform; the second one gives the type mismatch.
  TypeMismatch(String, String)
}

/// A shader uniform. `Uniform<T>` doesn’t hold any value. It’s more like a mapping between the
/// host code and the shader the uniform was retrieved from.
#[derive(Debug)]
pub struct Uniform<T> {
  program: GLuint,
  index: GLint,
  _t: PhantomData<*const T>
}

impl<T> Uniform<T> where T: Uniformable {
  fn new(program: GLuint, index: GLint) -> Self {
    Uniform {
      program: program,
      index: index,
      _t: PhantomData
    }
  }

  /// Create a new unbound uniform.
  fn unbound(program: GLuint) -> Self {
    Uniform {
      program: program,
      index: -1,
      _t: PhantomData
    }
  }

  // TODO: state whether it should be mutable or not – feels like a RefCell to me.
  // TODO: redesign the whole Uniformable + Uniform.update thing
  /// Update the value pointed by this uniform.
  pub fn update(&self, x: T) {
    x.update(self);
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
pub trait Uniformable: Copy + Sized {
  /// Update the uniform with a new value.
  fn update(self, u: &Uniform<Self>);
  /// Retrieve the `Type` of the uniform.
  fn ty() -> Type; // FIXME: call that ty() instead
  /// Retrieve the `Dim` of the uniform.
  fn dim() -> Dim;
}

impl Uniformable for i32 {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform1i(u.index, self) }
  }

  fn ty() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim1 }
}

impl Uniformable for [i32; 2] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform2iv(u.index, 1, &self as *const i32) }
  }

  fn ty() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim2 }
}

impl Uniformable for [i32; 3] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform3iv(u.index, 1, &self as *const i32) }
  }

  fn ty() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim3 }
}

impl Uniformable for [i32; 4] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform4iv(u.index, 1, &self as *const i32) }
  }

  fn ty() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim4 }
}

impl<'a> Uniformable for &'a [i32] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform1iv(u.index, self.len() as GLsizei, self.as_ptr()) }
  }

  fn ty() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim1 }
}

impl<'a> Uniformable for &'a [[i32; 2]] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform2iv(u.index, self.len() as GLsizei, self.as_ptr() as *const i32) }
  }

  fn ty() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim2 }
}

impl<'a> Uniformable for &'a [[i32; 3]] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform3iv(u.index, self.len() as GLsizei, self.as_ptr() as *const i32) }
  }

  fn ty() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim3 }
}

impl<'a> Uniformable for &'a [[i32; 4]] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform4iv(u.index, self.len() as GLsizei, self.as_ptr() as *const i32) }
  }

  fn ty() -> Type { Type::Integral }

  fn dim() -> Dim { Dim::Dim4 }
}

impl Uniformable for u32 {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform1ui(u.index, self) }
  }

  fn ty() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim1 }
}

impl Uniformable for [u32; 2] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform2uiv(u.index, 1, &self as *const u32) }
  }

  fn ty() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim2 }
}

impl Uniformable for [u32; 3] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform3uiv(u.index, 1, &self as *const u32) }
  }

  fn ty() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim3 }
}

impl Uniformable for [u32; 4] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform4uiv(u.index, 1, &self as *const u32) }
  }

  fn ty() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim4 }
}

impl<'a> Uniformable for &'a [u32] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform1uiv(u.index, self.len() as GLsizei, self.as_ptr() as *const u32) }
  }

  fn ty() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim1 }
}

impl<'a> Uniformable for &'a [[u32; 2]] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform2uiv(u.index, self.len() as GLsizei, self.as_ptr() as *const u32) }
  }

  fn ty() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim2 }
}

impl<'a> Uniformable for &'a [[u32; 3]] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform3uiv(u.index, self.len() as GLsizei, self.as_ptr() as *const u32) }
  }

  fn ty() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim3 }
}

impl<'a> Uniformable for &'a [[u32; 4]] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform4uiv(u.index, self.len() as GLsizei, self.as_ptr() as *const u32) }
  }

  fn ty() -> Type { Type::Unsigned }

  fn dim() -> Dim { Dim::Dim4 }
}

impl Uniformable for f32 {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform1f(u.index, self) }
  }

  fn ty() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim1 }
}

impl Uniformable for [f32; 2] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform2fv(u.index, 1, &self as *const f32) }
  }

  fn ty() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim2 }
}

impl Uniformable for [f32; 3] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform3fv(u.index, 1, &self as *const f32) }
  }

  fn ty() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim3 }
}

impl Uniformable for [f32; 4] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform4fv(u.index, 1, &self as *const f32) }
  }

  fn ty() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim4 }
}

impl<'a> Uniformable for &'a [f32] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform1fv(u.index, self.len() as GLsizei, self.as_ptr() as *const f32) }
  }

  fn ty() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim1 }
}

impl<'a> Uniformable for &'a [[f32; 2]] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform2fv(u.index, self.len() as GLsizei, self.as_ptr() as *const f32) }
  }

  fn ty() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim2 }
}

impl<'a> Uniformable for &'a [[f32; 3]] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform3fv(u.index, self.len() as GLsizei, self.as_ptr() as *const f32) }
  }

  fn ty() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim3 }
}

impl<'a> Uniformable for &'a [[f32; 4]] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform4fv(u.index, self.len() as GLsizei, self.as_ptr() as *const f32) }
  }

  fn ty() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim4 }
}

impl Uniformable for M22 {
  fn update(self, u: &Uniform<Self>) {
    let v = [self];
    unsafe { gl::UniformMatrix2fv(u.index, 1, gl::FALSE, v.as_ptr() as *const f32) }
  }

  fn ty() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim22 }
}

impl Uniformable for M33 {
  fn update(self, u: &Uniform<Self>) {
    let v = [self];
    unsafe { gl::UniformMatrix3fv(u.index, 1, gl::FALSE, v.as_ptr() as *const f32) }
  }

  fn ty() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim33 }
}

impl Uniformable for M44 {
  fn update(self, u: &Uniform<Self>) {
    let v = [self];
    unsafe { gl::UniformMatrix4fv(u.index, 1, gl::FALSE, v.as_ptr() as *const f32) }
  }

  fn ty() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim44 }
}

impl<'a> Uniformable for &'a [M22] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::UniformMatrix2fv(u.index, self.len() as GLsizei, gl::FALSE, self.as_ptr() as *const f32) }
  }

  fn ty() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim22 }
}

impl<'a> Uniformable for &'a [M33] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::UniformMatrix3fv(u.index, self.len() as GLsizei, gl::FALSE, self.as_ptr() as *const f32) }
  }

  fn ty() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim33 }
}

impl<'a> Uniformable for &'a [M44] {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::UniformMatrix4fv(u.index, self.len() as GLsizei, gl::FALSE, self.as_ptr() as *const f32) }
  }

  fn ty() -> Type { Type::Floating }

  fn dim() -> Dim { Dim::Dim44 }
}

impl Uniformable for bool {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform1ui(u.index, self as GLuint) }
  }

  fn ty() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim1 }
}

impl Uniformable for [bool; 2] {
  fn update(self, u: &Uniform<Self>) {
    let v = [self[0] as u32, self[1] as u32];
    unsafe { gl::Uniform2uiv(u.index, 1, &v as *const u32) }
  }

  fn ty() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim2 }
}

impl Uniformable for [bool; 3] {
  fn update(self, u: &Uniform<Self>) {
    let v = [self[0] as u32, self[1] as u32, self[2] as u32];
    unsafe { gl::Uniform3uiv(u.index, 1, &v as *const u32) }
  }

  fn ty() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim3 }
}

impl Uniformable for [bool; 4] {
  fn update(self, u: &Uniform<Self>) {
    let v = [self[0] as u32, self[1] as u32, self[2] as u32, self[3] as u32];
    unsafe { gl::Uniform4uiv(u.index, 1,  &v as *const u32) }
  }

  fn ty() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim4 }
}

impl<'a> Uniformable for &'a [bool] {
  fn update(self, u: &Uniform<Self>) {
    let v: Vec<_> = self.iter().map(|x| *x as u32).collect();
    unsafe { gl::Uniform1uiv(u.index, v.len() as GLsizei, v.as_ptr()) }
  }

  fn ty() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim1 }
}

impl<'a> Uniformable for &'a [[bool; 2]] {
  fn update(self, u: &Uniform<Self>) {
    let v: Vec<_> = self.iter().map(|x| [x[0] as u32, x[1] as u32]).collect();
    unsafe { gl::Uniform2uiv(u.index, v.len() as GLsizei, v.as_ptr() as *const u32) }
  }

  fn ty() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim2 }
}

impl<'a> Uniformable for &'a [[bool; 3]] {
  fn update(self, u: &Uniform<Self>) {
    let v: Vec<_> = self.iter().map(|x| [x[0] as u32, x[1] as u32, x[2] as u32]).collect();
    unsafe { gl::Uniform3uiv(u.index, v.len() as GLsizei, v.as_ptr() as *const u32) }
  }

  fn ty() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim3 }
}

impl<'a> Uniformable for &'a [[bool; 4]] {
  fn update(self, u: &Uniform<Self>) {
    let v: Vec<_> = self.iter().map(|x| [x[0] as u32, x[1] as u32, x[2] as u32, x[3] as u32]).collect();
    unsafe { gl::Uniform4uiv(u.index, v.len() as GLsizei, v.as_ptr() as *const u32) }
  }

  fn ty() -> Type { Type::Boolean }

  fn dim() -> Dim { Dim::Dim4 }
}

impl Uniformable for Unit {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::Uniform1i(u.index, *self as GLint) }
  }

  fn ty() -> Type { Type::TextureUnit }

  fn dim() -> Dim { Dim::Dim1 }
}

impl Uniformable for Binding {
  fn update(self, u: &Uniform<Self>) {
    unsafe { gl::UniformBlockBinding(u.program, u.index as GLuint, *self as GLuint) }
  }

  fn ty() -> Type { Type::BufferBinding }

  fn dim() -> Dim { Dim::Dim1 }
}

// Retrieve the uniform location.
//fn get_uniform_location(program: GLuint, name: &str, ty: Type, dim: Dim) -> (Location, ::std::result::Result<(), UniformWarning>) {
//  let c_name = CString::new(name.as_bytes()).unwrap();
//  let location = if ty == Type::BufferBinding {
//    let index = unsafe { gl::GetUniformBlockIndex(program, c_name.as_ptr() as *const GLchar) };
//
//    if index == gl::INVALID_INDEX {
//      return (Location::UniformBlock(-1), Err(UniformWarning::Inactive(name.to_owned())));
//    }
//
//    Location::UniformBlock(index as GLint)
//  } else {
//    let location = unsafe { gl::GetUniformLocation(program, c_name.as_ptr() as *const GLchar) };
//
//    if location == -1 {
//      return (Location::Uniform(-1), Err(UniformWarning::Inactive(name.to_owned())));
//    }
//
//    Location::Uniform(location)
//  };
//
//  if let Some(err) = uniform_type_match(program, name, ty, dim) {
//    return (location, Err(UniformWarning::TypeMismatch(name.to_owned(), err)));
//  }
//
//  (location, Ok(()))
//}

// Return something if no match can be established.
fn uniform_type_match(program: GLuint, name: &str, ty: Type, dim: Dim) -> Option<String> {
  let mut size: GLint = 0;
  let mut typ: GLuint = 0;

  unsafe {
    // get the index of the uniform
    let mut index = 0;
    gl::GetUniformIndices(program, 1, [name.as_ptr() as *const i8].as_ptr(), &mut index);
    // get its size and type
    gl::GetActiveUniform(program, index, 0, null_mut(), &mut size, &mut typ, null_mut());
  }

  // FIXME
  // early-return if array – we don’t support them yet
  if size != 1 {
    return None;
  }

  match (ty, dim) {
    (Type::Integral, Dim::Dim1) if typ != gl::INT => Some("requested int doesn't match".to_owned()),
    (Type::Integral, Dim::Dim2) if typ != gl::INT_VEC2 => Some("requested ivec2 doesn't match".to_owned()),
    (Type::Integral, Dim::Dim3) if typ != gl::INT_VEC3 => Some("requested ivec3 doesn't match".to_owned()),
    (Type::Integral, Dim::Dim4) if typ != gl::INT_VEC4 => Some("requested ivec4 doesn't match".to_owned()),
    (Type::Unsigned, Dim::Dim1) if typ != gl::UNSIGNED_INT => Some("requested uint doesn't match".to_owned()),
    (Type::Unsigned, Dim::Dim2) if typ != gl::UNSIGNED_INT_VEC2 => Some("requested uvec2 doesn't match".to_owned()),
    (Type::Unsigned, Dim::Dim3) if typ != gl::UNSIGNED_INT_VEC3 => Some("requested uvec3 doesn't match".to_owned()),
    (Type::Unsigned, Dim::Dim4) if typ != gl::UNSIGNED_INT_VEC4 => Some("requested uvec4 doesn't match".to_owned()),
    (Type::Floating, Dim::Dim1) if typ != gl::FLOAT => Some("requested float doesn't match".to_owned()),
    (Type::Floating, Dim::Dim2) if typ != gl::FLOAT_VEC2 => Some("requested vec2 doesn't match".to_owned()),
    (Type::Floating, Dim::Dim3) if typ != gl::FLOAT_VEC3 => Some("requested vec3 doesn't match".to_owned()),
    (Type::Floating, Dim::Dim4) if typ != gl::FLOAT_VEC4 => Some("requested vec4 doesn't match".to_owned()),
    (Type::Floating, Dim::Dim22) if typ != gl::FLOAT_MAT2 => Some("requested mat2 doesn't match".to_owned()),
    (Type::Floating, Dim::Dim33) if typ != gl::FLOAT_MAT3 => Some("requested mat3 doesn't match".to_owned()),
    (Type::Floating, Dim::Dim44) if typ != gl::FLOAT_MAT4 => Some("requested mat4 doesn't match".to_owned()),
    (Type::Boolean, Dim::Dim1) if typ != gl::BOOL => Some("requested bool doesn't match".to_owned()),
    (Type::Boolean, Dim::Dim2) if typ != gl::BOOL_VEC2 => Some("requested bvec2 doesn't match".to_owned()),
    (Type::Boolean, Dim::Dim3) if typ != gl::BOOL_VEC3 => Some("requested bvec3 doesn't match".to_owned()),
    (Type::Boolean, Dim::Dim4) if typ != gl::BOOL_VEC4 => Some("requested bvec4 doesn't match".to_owned()),
    _ => None
  }
}
