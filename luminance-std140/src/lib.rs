//! Types and traits implementing the [std140] OpenGL rule.
//!
//! [std140]: https://www.khronos.org/registry/OpenGL/specs/gl/glspec45.core.pdf#page=159

use luminance::shader::types::{Mat22, Mat33, Mat44, Vec2, Vec3, Vec4};

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

/// 32-bytes aligned wrapper.
///
/// This wrapper type wraps its inner type on 32-bytes, allowing for fast encode/decode operations.
#[repr(C, align(32))]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Aligned32<T>(pub T);

/// Implement [`Std140`] for a type as an identity
macro_rules! impl_Std140_id {
  ($t:ty) => {
    impl Std140 for $t {
      type Encoded = $t;

      fn std140_encode(self) -> Self::Encoded {
        self
      }

      fn std140_decode(encoded: Self::Encoded) -> Self {
        encoded
      }
    }
  };
}

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

/// Implement [`Std140`] for a type by wrapping it in [`Aligned32`].
macro_rules! impl_Std140_Aligned32 {
  ($t:ty) => {
    impl Std140 for $t {
      type Encoded = Aligned32<$t>;

      fn std140_encode(self) -> Self::Encoded {
        Aligned32(self)
      }

      fn std140_decode(encoded: Self::Encoded) -> Self {
        encoded.0
      }
    }
  };
}

impl_Std140_id!(f32);
impl_Std140_Aligned8!(Vec2<f32>);
impl_Std140_Aligned16!(Vec3<f32>);
impl_Std140_Aligned16!(Vec4<f32>);

impl_Std140_id!(f64);
impl_Std140_Aligned16!(Vec2<f64>);
impl_Std140_Aligned32!(Vec3<f64>);
impl_Std140_Aligned32!(Vec4<f64>);

impl_Std140_id!(i32);
impl_Std140_Aligned8!(Vec2<i32>);
impl_Std140_Aligned16!(Vec3<i32>);
impl_Std140_Aligned16!(Vec4<i32>);

impl_Std140_id!(u32);
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

impl Std140 for Mat22<f32> {
  type Encoded = Aligned16<[Aligned16<[f32; 2]>; 2]>;

  fn std140_encode(self) -> Self::Encoded {
    let [a, b]: [[f32; 2]; 2] = self.into();
    Aligned16([Aligned16(a), Aligned16(b)])
  }

  fn std140_decode(encoded: Self::Encoded) -> Self {
    let Aligned16([Aligned16(a), Aligned16(b)]) = encoded;
    [a, b].into()
  }
}

impl Std140 for Mat22<f64> {
  type Encoded = Aligned32<[Aligned32<[f64; 2]>; 2]>;

  fn std140_encode(self) -> Self::Encoded {
    let [a, b]: [[f64; 2]; 2] = self.into();
    Aligned32([Aligned32(a), Aligned32(b)])
  }

  fn std140_decode(encoded: Self::Encoded) -> Self {
    let Aligned32([Aligned32(a), Aligned32(b)]) = encoded;
    [a, b].into()
  }
}

impl Std140 for Mat33<f32> {
  type Encoded = Aligned16<[Aligned16<[f32; 3]>; 3]>;

  fn std140_encode(self) -> Self::Encoded {
    let [a, b, c]: [[f32; 3]; 3] = self.into();
    Aligned16([Aligned16(a), Aligned16(b), Aligned16(c)])
  }

  fn std140_decode(encoded: Self::Encoded) -> Self {
    let Aligned16([Aligned16(a), Aligned16(b), Aligned16(c)]) = encoded;
    [a, b, c].into()
  }
}

impl Std140 for Mat33<f64> {
  type Encoded = Aligned32<[Aligned32<[f64; 3]>; 3]>;

  fn std140_encode(self) -> Self::Encoded {
    let [a, b, c]: [[f64; 3]; 3] = self.into();
    Aligned32([Aligned32(a), Aligned32(b), Aligned32(c)])
  }

  fn std140_decode(encoded: Self::Encoded) -> Self {
    let Aligned32([Aligned32(a), Aligned32(b), Aligned32(c)]) = encoded;
    [a, b, c].into()
  }
}

impl Std140 for Mat44<f32> {
  type Encoded = Aligned16<[Aligned16<[f32; 4]>; 4]>;

  fn std140_encode(self) -> Self::Encoded {
    let [a, b, c, d]: [[f32; 4]; 4] = self.into();
    Aligned16([Aligned16(a), Aligned16(b), Aligned16(c), Aligned16(d)])
  }

  fn std140_decode(encoded: Self::Encoded) -> Self {
    let Aligned16([Aligned16(a), Aligned16(b), Aligned16(c), Aligned16(d)]) = encoded;
    [a, b, c, d].into()
  }
}

impl Std140 for Mat44<f64> {
  type Encoded = Aligned32<[Aligned32<[f64; 4]>; 4]>;

  fn std140_encode(self) -> Self::Encoded {
    let [a, b, c, d]: [[f64; 4]; 4] = self.into();
    Aligned32([Aligned32(a), Aligned32(b), Aligned32(c), Aligned32(d)])
  }

  fn std140_decode(encoded: Self::Encoded) -> Self {
    let Aligned32([Aligned32(a), Aligned32(b), Aligned32(c), Aligned32(d)]) = encoded;
    [a, b, c, d].into()
  }
}

/// Type wrapper for values inside arrays.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Arr<T>(pub T);

impl<T> Std140 for Arr<T>
where
  T: Std140,
{
  type Encoded = Aligned16<<T as Std140>::Encoded>;

  fn std140_encode(self) -> Self::Encoded {
    Aligned16(self.0.std140_encode())
  }

  fn std140_decode(encoded: Self::Encoded) -> Self {
    Arr(<T as Std140>::std140_decode(encoded.0))
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
  fn aligned16() {
    assert_eq!(std::mem::size_of::<Aligned16<f32>>(), 16);
    assert_eq!(std::mem::size_of::<Aligned16<Aligned16<f32>>>(), 16);
  }

  #[test]
  fn vec2() {
    assert_size_align::<Vec2<f32>>(8, 8);
    assert_size_align::<Vec2<f64>>(16, 16);
  }

  #[test]
  fn vec3() {
    assert_size_align::<Vec3<f32>>(16, 16);
    assert_size_align::<Vec3<f64>>(32, 32);
  }

  #[test]
  fn vec4() {
    assert_size_align::<Vec4<f32>>(16, 16);
    assert_size_align::<Vec4<f64>>(32, 32);
  }

  #[test]
  fn i32() {
    assert_size_align::<i32>(4, 4);
  }

  #[test]
  fn ivec2() {
    assert_size_align::<Vec2<i32>>(8, 8);
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
    assert_size_align::<Vec2<u32>>(8, 8);
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
    assert_size_align::<Vec2<bool>>(8, 8);
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
  fn mat22() {
    assert_size_align::<Mat22<f32>>(32, 16);
  }

  #[test]
  fn mat33() {
    assert_size_align::<Mat33<f32>>(48, 16);
  }

  #[test]
  fn mat44() {
    assert_size_align::<Mat44<f32>>(64, 16);
  }

  #[test]
  fn vec2_arrayed() {
    assert_size_align::<Arr<Vec2<f32>>>(16, 16);
    assert_size_align::<Arr<Vec2<f64>>>(16, 16);
  }

  #[test]
  fn vec3_array() {
    assert_size_align::<Arr<Vec3<f32>>>(16, 16);
    assert_size_align::<Arr<Vec3<f64>>>(32, 32);
  }

  #[test]
  fn vec4_array() {
    assert_size_align::<Arr<Vec4<f32>>>(16, 16);
    assert_size_align::<Arr<Vec4<f64>>>(32, 32);
  }

  #[test]
  fn mat22_array() {
    assert_size_align::<Arr<Mat22<f32>>>(32, 16);
    assert_size_align::<Arr<Mat22<f64>>>(64, 32);
  }

  #[test]
  fn mat33_array() {
    assert_size_align::<Arr<Mat33<f32>>>(48, 16);
    assert_size_align::<Arr<Mat33<f64>>>(96, 32);
  }

  #[test]
  fn mat44_array() {
    assert_size_align::<Arr<Mat44<f32>>>(64, 16);
    assert_size_align::<Arr<Mat44<f64>>>(128, 32);
  }
}
