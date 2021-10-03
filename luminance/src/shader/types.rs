//! Shader type wrappers.
//!
//! These types are used, mostly, to be passed to shaders as [`Uniform`] data.

use std::ops::{Deref, DerefMut};

/// A 2 dimensional vector.
///
/// This is akin to a `[T; 2]`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Vec2<T>(pub [T; 2]);

impl<T> From<[T; 2]> for Vec2<T> {
  fn from(a: [T; 2]) -> Self {
    Vec2(a)
  }
}

impl<T> From<Vec2<T>> for [T; 2] {
  fn from(Vec2(a): Vec2<T>) -> Self {
    a
  }
}

impl<T> AsRef<[T; 2]> for Vec2<T> {
  fn as_ref(&self) -> &[T; 2] {
    &self.0
  }
}

impl<T> Deref for Vec2<T> {
  type Target = [T; 2];

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<T> DerefMut for Vec2<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl<T> Vec2<T> {
  /// Create a new vector.
  pub const fn new(x: T, y: T) -> Self {
    Self([x, y])
  }
}

/// A 3 dimensional vector.
///
/// This is akin to a `[T; 3]`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Vec3<T>(pub [T; 3]);

impl<T> From<[T; 3]> for Vec3<T> {
  fn from(a: [T; 3]) -> Self {
    Vec3(a)
  }
}

impl<T> From<Vec3<T>> for [T; 3] {
  fn from(Vec3(a): Vec3<T>) -> Self {
    a
  }
}

impl<T> AsRef<[T; 3]> for Vec3<T> {
  fn as_ref(&self) -> &[T; 3] {
    &self.0
  }
}

impl<T> Deref for Vec3<T> {
  type Target = [T; 3];

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<T> DerefMut for Vec3<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl<T> Vec3<T> {
  /// Create a new vector.
  pub const fn new(x: T, y: T, z: T) -> Self {
    Self([x, y, z])
  }
}

/// A 4 dimensional vector.
///
/// This is akin to a `[T; 4]`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Vec4<T>(pub [T; 4]);

impl<T> From<[T; 4]> for Vec4<T> {
  fn from(a: [T; 4]) -> Self {
    Vec4(a)
  }
}

impl<T> From<Vec4<T>> for [T; 4] {
  fn from(Vec4(a): Vec4<T>) -> Self {
    a
  }
}

impl<T> AsRef<[T; 4]> for Vec4<T> {
  fn as_ref(&self) -> &[T; 4] {
    &self.0
  }
}

impl<T> Deref for Vec4<T> {
  type Target = [T; 4];

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<T> DerefMut for Vec4<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl<T> Vec4<T> {
  /// Create a new vector.
  pub const fn new(x: T, y: T, z: T, w: T) -> Self {
    Self([x, y, z, w])
  }
}
