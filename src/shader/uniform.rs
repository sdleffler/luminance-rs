//! Shader uniforms and associated operations.
//!
//! Uniforms kick in several and useful ways. They’re used to customize shaders.
use core::marker::PhantomData;

pub trait HasUniform {
  /// Uniform representation.
  type U;

  // integral
  fn update1_i32(uniform: &Self::U, x: i32);
  fn update2_i32(uniform: &Self::U, xy: (i32, i32));
  fn update3_i32(uniform: &Self::U, xyz: (i32, i32, i32));
  fn update4_i32(uniform: &Self::U, xyzw: (i32, i32, i32, i32));
  fn update1_vec_i32(uniform: &Self::U, x: &Vec<i32>);
  fn update2_vec_i32(uniform: &Self::U, xy: &Vec<(i32, i32)>);
  fn update3_vec_i32(uniform: &Self::U, xyz: &Vec<(i32, i32, i32)>);
  fn update4_vec_i32(uniform: &Self::U, xyzw: &Vec<(i32, i32, i32, i32)>);
  // unsigned
  fn update1_u32(uniform: &Self::U, x: u32);
  fn update2_u32(uniform: &Self::U, xy: (u32, u32));
  fn update3_u32(uniform: &Self::U, xyz: (u32, u32, u32));
  fn update4_u32(uniform: &Self::U, xyzw: (u32, u32, u32, u32));
  fn update1_vec_u32(uniform: &Self::U, x: &Vec<u32>);
  fn update2_vec_u32(uniform: &Self::U, xy: &Vec<(u32, u32)>);
  fn update3_vec_u32(uniform: &Self::U, xyz: &Vec<(u32, u32, u32)>);
  fn update4_vec_u32(uniform: &Self::U, xyzw: &Vec<(u32, u32, u32, u32)>);
  // floating
  fn update1_f32(uniform: &Self::U, x: f32);
  fn update2_f32(uniform: &Self::U, xy: (f32, f32));
  fn update3_f32(uniform: &Self::U, xyz: (f32, f32, f32));
  fn update4_f32(uniform: &Self::U, xyzw: (f32, f32, f32, f32));
  fn update1_vec_f32(uniform: &Self::U, x: &Vec<f32>);
  fn update2_vec_f32(uniform: &Self::U, xy: &Vec<(f32, f32)>);
  fn update3_vec_f32(uniform: &Self::U, xyz: &Vec<(f32, f32, f32)>);
  fn update4_vec_f32(uniform: &Self::U, xyzw: &Vec<(f32, f32, f32, f32)>);
  fn update22_f32(uniform: &Self::U, x: ((f32, f32), (f32, f32)));
  fn update33_f32(uniform: &Self::U, x: ((f32, f32, f32), (f32, f32, f32), (f32, f32, f32)));
  fn update44_f32(uniform: &Self::U, x: ((f32, f32, f32, f32), (f32, f32, f32, f32), (f32, f32, f32, f32), (f32, f32, f32, f32)));
  fn update22_vec_f32(uniform: &Self::U, x: &Vec<((f32, f32), (f32, f32))>);
  fn update33_vec_f32(uniform: &Self::U, x: &Vec<((f32, f32, f32), (f32, f32, f32), (f32, f32, f32))>);
  fn update44_vec_f32(uniform: &Self::U, x: &Vec<((f32, f32, f32, f32), (f32, f32, f32, f32), (f32, f32, f32, f32), (f32, f32, f32, f32))>);
  // boolean
  fn update1_bool(uniform: &Self::U, x: bool);
  fn update2_bool(uniform: &Self::U, xy: (bool, bool));
  fn update3_bool(uniform: &Self::U, xyz: (bool, bool, bool));
  fn update4_bool(uniform: &Self::U, xyzw: (bool, bool, bool, bool));
  fn update1_vec_bool(uniform: &Self::U, x: &Vec<bool>);
  fn update2_vec_bool(uniform: &Self::U, xy: &Vec<(bool, bool)>);
  fn update3_vec_bool(uniform: &Self::U, xyz: &Vec<(bool, bool, bool)>);
  fn update4_vec_bool(uniform: &Self::U, xyzw: &Vec<(bool, bool, bool, bool)>);
}

/// A shader uniform. `Uniform<C, T>` doesn’t hold any value. It’s more like a mapping between the
/// host code and the shader the uniform was retrieved from.
pub struct Uniform<C, T> where C: HasUniform, T: Uniformable {
  pub repr: C::U,
  _t: PhantomData<T>
}

/// Name of a `Uniform`.
pub enum UniformName<'a> {
  StringName(&'a str),
  SemanticName(u32)
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

impl Uniformable for (i32, i32) {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update2_i32(&u.repr, x)
  }
}

impl Uniformable for (i32, i32, i32) {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update3_i32(&u.repr, x)
  }
}

impl Uniformable for (i32, i32, i32, i32) {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update4_i32(&u.repr, x)
  }
}

impl Uniformable for u32 {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update1_u32(&u.repr, x)
  }
}

impl Uniformable for (u32, u32) {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update2_u32(&u.repr, x)
  }
}

impl Uniformable for (u32, u32, u32) {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update3_u32(&u.repr, x)
  }
}

impl Uniformable for (u32, u32, u32, u32) {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update4_u32(&u.repr, x)
  }
}

impl Uniformable for f32 {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update1_f32(&u.repr, x)
  }
}

impl Uniformable for (f32, f32) {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update2_f32(&u.repr, x)
  }
}

impl Uniformable for (f32, f32, f32) {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update3_f32(&u.repr, x)
  }
}

impl Uniformable for (f32, f32, f32, f32) {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update4_f32(&u.repr, x)
  }
}

impl Uniformable for ((f32, f32), (f32, f32)) {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update22_f32(&u.repr, x)
  }
}

impl Uniformable for ((f32, f32, f32), (f32, f32, f32), (f32, f32, f32)) {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update33_f32(&u.repr, x)
  }
}

impl Uniformable for ((f32, f32, f32, f32), (f32, f32, f32, f32), (f32, f32, f32, f32), (f32, f32, f32, f32)) {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update44_f32(&u.repr, x)
  }
}

impl Uniformable for bool {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update1_bool(&u.repr, x)
  }
}

impl Uniformable for (bool, bool) {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update2_bool(&u.repr, x)
  }
}

impl Uniformable for (bool, bool, bool) {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update3_bool(&u.repr, x)
  }
}

impl Uniformable for (bool, bool, bool, bool) {
  fn update<C>(u: &Uniform<C, Self>, x: Self) where C: HasUniform {
    C::update4_bool(&u.repr, x)
  }
}
