//! Shader uniforms and associated operations.
//!
//! Uniforms kick in several and useful ways. They’re used to customize shaders.

use core::marker::PhantomData;
use linear::*;

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
}

/// A shader uniform. `Uniform<C, T>` doesn’t hold any value. It’s more like a mapping between the
/// host code and the shader the uniform was retrieved from.
#[derive(Debug)]
pub struct Uniform<C, T> where C: HasUniform, T: Uniformable {
  pub repr: C::U,
  _t: PhantomData<T>
}

impl<C, T> Uniform<C, T> where C: HasUniform, T: Uniformable {
  pub fn new(repr: C::U) -> Uniform<C, T> {
    Uniform {
      repr: repr,
      _t: PhantomData
    }
  }

  pub fn update(&self, x: T) {
    T::update(self, x);
  }
}

/// Name of a `Uniform`.
#[derive(Debug)]
pub enum UniformName {
  StringName(String),
  SemanticName(u32)
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

impl<'a, C, T> From<Uniform<C, T>> for UniformUpdate<'a, T> where C: 'a + HasUniform, T: 'a + Uniformable {
  fn from(u: Uniform<C, T>) -> Self {
    UniformUpdate {
      update_closure: Box::new(move |x| {
        u.update(x);
      })
    }
  }
}

impl<'a, T> UniformUpdate<'a, T> where T: 'a + Uniformable {
  /// Update the underlying `Uniform`.
  pub fn update(&'a self, x: T) {
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


/// Types that can behave as `Uniform`.
pub trait Uniformable: Sized {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform;
}

impl Uniformable for i32 {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update1_i32(&u.repr, x)
  }
}

impl Uniformable for [i32; 2] {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update2_i32(&u.repr, x)
  }
}

impl Uniformable for [i32; 3] {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update3_i32(&u.repr, x)
  }
}

impl Uniformable for [i32; 4] {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update4_i32(&u.repr, x)
  }
}

impl<'a> Uniformable for &'a [i32] {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update1_slice_i32(&u.repr, x)
  }
}

impl<'a> Uniformable for &'a [[i32; 2]] {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update2_slice_i32(&u.repr, x)
  }
}

impl<'a> Uniformable for &'a [[i32; 3]] {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update3_slice_i32(&u.repr, x)
  }
}

impl<'a> Uniformable for &'a [[i32; 4]] {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update4_slice_i32(&u.repr, x)
  }
}

impl Uniformable for u32 {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update1_u32(&u.repr, x)
  }
}

impl Uniformable for [u32; 2] {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update2_u32(&u.repr, x)
  }
}

impl Uniformable for [u32; 3] {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update3_u32(&u.repr, x)
  }
}

impl Uniformable for [u32; 4] {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update4_u32(&u.repr, x)
  }
}

impl<'a> Uniformable for &'a [u32] {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update1_slice_u32(&u.repr, x)
  }
}

impl<'a> Uniformable for &'a [[u32; 2]] {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update2_slice_u32(&u.repr, x)
  }
}

impl<'a> Uniformable for &'a [[u32; 3]] {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update3_slice_u32(&u.repr, x)
  }
}

impl<'a> Uniformable for &'a [[u32; 4]] {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update4_slice_u32(&u.repr, x)
  }
}

impl Uniformable for f32 {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update1_f32(&u.repr, x)
  }
}

impl Uniformable for [f32; 2] {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update2_f32(&u.repr, x)
  }
}

impl Uniformable for [f32; 3] {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update3_f32(&u.repr, x)
  }
}

impl Uniformable for [f32; 4] {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update4_f32(&u.repr, x)
  }
}

impl<'a> Uniformable for &'a [f32] {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update1_slice_f32(&u.repr, x)
  }
}

impl<'a> Uniformable for &'a [[f32; 2]] {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update2_slice_f32(&u.repr, x)
  }
}

impl<'a> Uniformable for &'a [[f32; 3]] {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update3_slice_f32(&u.repr, x)
  }
}

impl<'a> Uniformable for &'a [[f32; 4]] {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update4_slice_f32(&u.repr, x)
  }
}

impl Uniformable for M22 {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update22_f32(&u.repr, x)
  }
}

impl Uniformable for M33 {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update33_f32(&u.repr, x)
  }
}

impl Uniformable for M44 {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update44_f32(&u.repr, x)
  }
}

impl<'a> Uniformable for &'a [M22] {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update22_slice_f32(&u.repr, x)
  }
}

impl<'a> Uniformable for &'a [M33] {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update33_slice_f32(&u.repr, x)
  }
}

impl<'a> Uniformable for &'a [M44] {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update44_slice_f32(&u.repr, x)
  }
}

impl Uniformable for bool {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update1_bool(&u.repr, x)
  }
}

impl Uniformable for [bool; 2] {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update2_bool(&u.repr, x)
  }
}

impl Uniformable for [bool; 3] {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update3_bool(&u.repr, x)
  }
}

impl Uniformable for [bool; 4] {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update4_bool(&u.repr, x)
  }
}

impl<'a> Uniformable for &'a [bool] {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update1_slice_bool(&u.repr, x)
  }
}

impl<'a> Uniformable for &'a [[bool; 2]] {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update2_slice_bool(&u.repr, x)
  }
}

impl<'a> Uniformable for &'a [[bool; 3]] {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update3_slice_bool(&u.repr, x)
  }
}

impl<'a> Uniformable for &'a [[bool; 4]] {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update4_slice_bool(&u.repr, x)
  }
}
