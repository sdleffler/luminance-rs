//! Types and traits implementing the [std140] OpenGL rule.
//!
//! [std140]: https://www.khronos.org/registry/OpenGL/specs/gl/glspec45.core.pdf#page=159

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

/// 16-bytes aligned wrapper.
///
/// This wrapper type wraps its inner type on 16-bytes, allowing for fast encode/decode operations.
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Aligned16<T>(pub T);

/// Implement [`Std140`] for a type via an identity function.
macro_rules! impl_Std140_id {
  ($t:ty, $zero:expr) => {
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
  ($t:ty, $zero:expr) => {
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

/// Implement [`Std140`] for a type by wrapping it in [`Aligned16`].
macro_rules! impl_Std140_Aligned16 {
  ($t:ty, $zero:expr) => {
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

// vec*
impl_Std140_id!([f32; 2], [0.; 2]);
impl_Std140_Aligned16!([f32; 3], [0.; 3]);
impl_Std140_Aligned16!([f32; 4], [0.; 4]);

// ivec*
impl_Std140_id!([i32; 2], [0; 2]);
impl_Std140_Aligned16!([i32; 3], [0; 3]);
impl_Std140_Aligned16!([i32; 4], [0; 4]);

impl_Std140_Aligned4!(bool, false);

impl Std140 for Vec2<bool> {
  type Encoded = Vec2<Aligned4<bool>>;

  fn std140_encode(self) -> Self::Encoded {
    let Vec2([x, y]) = self;
    Vec2::new(Aligned4(x), Aligned4(y))
  }

  fn std140_decode(encoded: Self::Encoded) -> Self {
    let Vec2([Aligned4(x), Aligned4(y)]) = encoded;
    Vec2::new(x, y)
  }
}

impl_Std140_Aligned16!(Vec3<bool>, Vec3::new(false, false, false));
impl_Std140_Aligned16!(Vec4<bool>, Vec4::new(false, false, false, false));

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
  use std::mem;

  fn assert_size_align<T>(size: usize, align: usize)
  where
    T: Std140,
  {
    assert_eq!(mem::size_of::<<T as Std140>::Encoded>(), size);
    assert_eq!(mem::align_of::<<T as Std140>::Encoded>(), align);
  }

  #[test]
  fn vec2() {
    assert_size_align::<[f32; 2]>(8, 4);
  }

  #[test]
  fn vec3() {
    assert_size_align::<[f32; 3]>(16, 16);
  }

  #[test]
  fn vec4() {
    assert_size_align::<[f32; 4]>(16, 16);
  }

  #[test]
  fn ivec2() {
    assert_size_align::<[i32; 2]>(8, 4);
  }

  #[test]
  fn ivec3() {
    assert_size_align::<[i32; 3]>(16, 16);
  }

  #[test]
  fn ivec4() {
    assert_size_align::<[i32; 4]>(16, 16);
  }

  #[test]
  fn uvec2() {
    assert_size_align::<[u32; 2]>(8, 4);
  }

  #[test]
  fn uvec3() {
    assert_size_align::<[u32; 3]>(16, 16);
  }

  #[test]
  fn uvec4() {
    assert_size_align::<[u32; 4]>(16, 16);
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
