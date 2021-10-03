//! Types and traits implementing the [std140] OpenGL rule.
//!
//! [std140]: https://www.khronos.org/registry/OpenGL/specs/gl/glspec45.core.pdf#page=159

use luminance::shader::types::{Vec2, Vec3, Vec4};

/// Types that have a `std140` representation.
///
/// This trait allows to encode types into their `std140` representation but also decode such representation into the
/// proper type.
pub trait Std140: Copy {
  type Encoded: Copy;

  /// Encode the value into its `std140` representation.
  fn std140_encode(self) -> Self::Encoded;

  /// Decode a value from its `std140` representation.
  fn std140_decode(encoded: Self::Encoded) -> Self;
}

/// 4-bytes aligned wrapper.
///
/// This wrapper type wraps its inner type on 4-bytes, allowing for fast encode/decode operations.
#[repr(C, align(4))]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Aligned4<T>(pub T);

/// 8-bytes aligned wrapper.
///
/// This wrapper type wraps its inner type on 8-bytes, allowing for fast encode/decode operations.
#[repr(C, align(8))]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Aligned8<T>(pub T);

/// 16-bytes aligned wrapper.
///
/// This wrapper type wraps its inner type on 16-bytes, allowing for fast encode/decode operations.
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Aligned16<T>(pub T);

/// Implement [`Std140`] for a type by wrapping it in [`Aligned4`].
macro_rules! impl_Std140_Aligned4 {
  ($t:ty) => {
    impl Std140 for $t {
      type Encoded = Aligned4<$t>;

      fn std140_encode(self) -> Self::Encoded {
        Aligned4(self)
      }

      fn std140_decode(encoded: Self::Encoded) -> Self {
        encoded.0
      }
    }
  };
}

/// Implement [`Std140`] for a type by wrapping it in [`Aligned8`].
macro_rules! impl_Std140_Aligned8 {
  ($t:ty) => {
    impl Std140 for $t {
      type Encoded = Aligned8<$t>;

      fn std140_encode(self) -> Self::Encoded {
        Aligned8(self)
      }

      fn std140_decode(encoded: Self::Encoded) -> Self {
        encoded.0
      }
    }
  };
}

/// Implement [`Std140`] for a type by wrapping it in [`Aligned16`].
macro_rules! impl_Std140_Aligned16 {
  ($t:ty) => {
    impl Std140 for $t {
      type Encoded = Aligned16<$t>;

      fn std140_encode(self) -> Self::Encoded {
        Aligned16(self)
      }

      fn std140_decode(encoded: Self::Encoded) -> Self {
        encoded.0
      }
    }
  };
}

impl_Std140_Aligned4!(f32);
impl_Std140_Aligned8!(Vec2<f32>);
impl_Std140_Aligned16!(Vec3<f32>);
impl_Std140_Aligned16!(Vec4<f32>);

impl_Std140_Aligned4!(i32);
impl_Std140_Aligned8!(Vec2<i32>);
impl_Std140_Aligned16!(Vec3<i32>);
impl_Std140_Aligned16!(Vec4<i32>);

impl_Std140_Aligned4!(u32);
impl_Std140_Aligned8!(Vec2<u32>);
impl_Std140_Aligned16!(Vec3<u32>);
impl_Std140_Aligned16!(Vec4<u32>);

impl_Std140_Aligned4!(bool);

impl Std140 for Vec2<bool> {
  type Encoded = Aligned8<Vec2<Aligned4<bool>>>;

  fn std140_encode(self) -> Self::Encoded {
    let Vec2([x, y]) = self;
    Aligned8(Vec2::new(Aligned4(x), Aligned4(y)))
  }

  fn std140_decode(encoded: Self::Encoded) -> Self {
    let Aligned8(Vec2([Aligned4(x), Aligned4(y)])) = encoded;
    Vec2::new(x, y)
  }
}

impl_Std140_Aligned16!(Vec3<bool>);
impl_Std140_Aligned16!(Vec4<bool>);

/// Type wrapper for values inside arrays.
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Arr<T>(pub T);

impl<T> Std140 for Arr<T>
where
  T: Std140,
{
  type Encoded = Aligned16<T>;

  fn std140_encode(self) -> Self::Encoded {
    Aligned16(self.0)
  }

  fn std140_decode(encoded: Self::Encoded) -> Self {
    Arr(encoded.0)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use luminance::shader::types::{Vec3, Vec4};
  use std::mem;

  fn assert_size_align<T>(size: usize, align: usize)
  where
    T: Std140,
  {
    assert_eq!(mem::size_of::<<T as Std140>::Encoded>(), size);
    assert_eq!(mem::align_of::<<T as Std140>::Encoded>(), align);
  }

  #[test]
  fn f32() {
    assert_size_align::<f32>(4, 4);
  }

  #[test]
  fn vec2() {
    assert_size_align::<Vec2<f32>>(8, 4);
  }

  #[test]
  fn vec3() {
    assert_size_align::<Vec3<f32>>(16, 16);
  }

  #[test]
  fn vec4() {
    assert_size_align::<Vec4<f32>>(16, 16);
  }

  #[test]
  fn i32() {
    assert_size_align::<i32>(4, 4);
  }

  #[test]
  fn ivec2() {
    assert_size_align::<Vec2<i32>>(8, 4);
  }

  #[test]
  fn ivec3() {
    assert_size_align::<Vec3<i32>>(16, 16);
  }

  #[test]
  fn ivec4() {
    assert_size_align::<Vec4<i32>>(16, 16);
  }

  #[test]
  fn u32() {
    assert_size_align::<u32>(4, 4);
  }

  #[test]
  fn uvec2() {
    assert_size_align::<Vec2<u32>>(8, 4);
  }

  #[test]
  fn uvec3() {
    assert_size_align::<Vec3<i32>>(16, 16);
  }

  #[test]
  fn uvec4() {
    assert_size_align::<Vec4<i32>>(16, 16);
  }

  #[test]
  fn bool() {
    assert_size_align::<bool>(4, 4);
  }

  #[test]
  fn bvec2() {
    assert_size_align::<Vec2<bool>>(8, 4);
  }

  #[test]
  fn bvec3() {
    assert_size_align::<Vec3<bool>>(16, 16);
  }

  #[test]
  fn bvec4() {
    assert_size_align::<Vec4<bool>>(16, 16);
  }

  #[test]
  fn vec2_arrayed() {
    assert_size_align::<Arr<Vec2<f32>>>(16, 16);
  }

  #[test]
  fn vec3_array() {
    assert_size_align::<Arr<Vec3<f32>>>(16, 16);
  }

  #[test]
  fn vec4_array() {
    assert_size_align::<Arr<Vec4<f32>>>(16, 16);
  }
}
