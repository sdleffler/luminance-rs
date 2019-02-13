//! Vertex formats, associated types and functions.
//!
//! A vertex is a type representing a point. It’s common to find vertex positions, normals, colors
//! or even texture coordinates. However, you’re free to use whichever type you want. Nevertheless,
//! you’re limited to a range of types and dimensions. See `VertexAttributeType` and `VertexAttributeDim` for further details.
//!
//! # `Vertex`
//!
//! ## Rules
//!
//! To be able to use a type as a vertex, you have to implement the `Vertex` trait. That trait
//! represents a mapping between your type and `VertexFmt`. A `VertexFmt` gives runtime hints
//! about your type and restricts the supported type. If you cannot map your type to `VertexFmt`,
//! that means you cannot use it as a `Vertex`.
//!
//! The rule is that your type should have a static size greater than 0 and less than or equal to 4.
//! It should also be either integral, unsigned, floating or boolean. If your type is a complex one
//! – for instance a `struct` – you have to recursively apply that rule to all its fields.
//! For instance, the tuple `(i32, bool)` implements `Vertex` by providing an implementation using
//! the ones of `i32` and `bool`.
//!
//! ## Attributes list
//!
//! As mentionned above, you can use tuples and structs as `Vertex`. If you look at the definition
//! of `VertexFmt`, you’ll notice that it’s a `Vec<VertexAttributeFmt>`. That means simple
//! and primary types map to unit vectors – i.e. their size is 1 – but tuples and structs need
//! several `VertexAttributeFmt`s to be represented, hence vectors with sizes greater than 1. No
//! check is made on how many vertex attributes you’re using – there’s a practical limit, set by the
//! GPU, but it’s not checked (yet).
//!
//! # Generic implementation
//!
//! You have `Vertex` implementations for all the primary types that can be mapped to
//! `VertexFmt`. However, as it’s not possible to automatically implement `Vertex` for your
//! structure (yet?), a type is provided to help you design your vertex type so that you’re
//! automatically provided with a `Vertex` implementation if you use `GTup`.
//!
//! `GTup` is a special type used to represent static heterogeneous list of types. With that in
//! hand, you can easily create `Vertex` types and start using them without even implementing
//! `Vertex`, as long as you use `Vertex` types. Feel free to dig in the `GTup` documentation for
//! further details.
//!
//! If you absolutely want to use your own types – which is legit, you can implement `Vertex` by
//! mapping your inner fields to a tuple or `GTup`, and call the right `Vertex` method on that
//! tuple.

use crate::deinterleave::Deinterleave;

/// A type that can be used as a `Vertex` has to implement that trait – it must provide a mapping
/// to `VertexFmt`.
///
/// If you’re not sure on how to implement that or if you want to use automatic types, feel free
/// to use the primary supported types and `GTup` or tuples.
pub unsafe trait Vertex<'a> {
  type Deinterleaved: Deinterleave;

  const VERTEX_FMT: VertexFmt;
}

unsafe impl<'a> Vertex<'a> for () {
  type Deinterleaved = ();

  const VERTEX_FMT: VertexFmt = &[];
}

// /// Universal `Vertex` implementation. Any type that already implements `VertexAttribute` also
// /// implements `Vertex`.
// impl <'a, T> Vertex<'a> where T: VertexAttribute {
//   type Deinterleaved = &'a [T];
//
//   fn vertex_fmt() -> VertexFmt {
//     &[Self::vertex_attribute()]
//   }
// }

/// A hint trait to implement to state whether a vertex type is compatible with another.
///
/// If you have two types `V0: Vertex` and `V1: Vertex`, we say that `V1` is compatible with `V0`
/// only if `&V0::vertex_fmt()[0..V1::vertex_fmt().len()] == &V1::vertex_fmt()[..]`. That
/// is, if `V1` is a sub-slice of `V0` starting at 0.
///
/// We node that as `V1: CompatibleVertex<V0>`.
pub unsafe trait CompatibleVertex<'a, V>: Vertex<'a>
where
  V: Vertex<'a>,
{
}

unsafe impl<'a, V> CompatibleVertex<'a, V> for V where V: Vertex<'a> {}

/// A `VertexFmt` is a list of `VertexAttributeFmt`s.
pub type VertexFmt = &'static [VertexAttributeFmt];

/// Vertex attribute format. It gives information on how vertices should be passed to the GPU and
/// optimized in buffers.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct VertexAttributeFmt {
  /// Type of the attribute. See `VertexAttributeType` for further details.
  pub comp_type: VertexAttributeType,
  /// Dimension of the attribute. It should be in 1–4. See `VertexAttributeDim` for further details.
  pub dim: VertexAttributeDim,
  /// Size in bytes that a single element of the attribute takes. That is, if your attribute has
  /// a dimension set to 2, then the unit size should be the size of a single element (not two).
  pub unit_size: usize,
  /// Alignment of the attribute. The best advice is to respect what Rust does, so it’s highly
  /// recommended to use `::std::mem::align_of` to let it does the job for you.
  pub align: usize,
}

/// Possible type of vertex attributes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum VertexAttributeType {
  Integral,
  Unsigned,
  Floating,
  Boolean,
}

/// Possible dimension of vertex attributes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum VertexAttributeDim {
  Dim1,
  Dim2,
  Dim3,
  Dim4,
}

/// Class of vertex attributes.
pub unsafe trait VertexAttribute {
  const VERTEX_ATTRIBUTE_FMT: VertexAttributeFmt;
}

/// A local version of size_of that depends on the state of the std feature.
#[inline(always)]
pub const fn size_of<T>() -> usize {
  #[cfg(feature = "std")]
  {
    ::std::mem::size_of::<T>()
  }

  #[cfg(not(feature = "std"))]
  {
    ::core::mem::size_of::<T>()
  }
}

/// A local version of align_of that depends on the state of the std feature.
#[inline(always)]
pub const fn align_of<T>() -> usize {
  #[cfg(feature = "std")]
  {
    ::std::mem::align_of::<T>()
  }

  #[cfg(not(feature = "std"))]
  {
    ::core::mem::align_of::<T>()
  }
}

// Macro to quickly implement VertexAttribute for a given type.
macro_rules! impl_vertex_attribute {
  ($t:ty, $q:ty, $comp_type:ident, $dim:ident) => {
    unsafe impl VertexAttribute for $t {
      const VERTEX_ATTRIBUTE_FMT: VertexAttributeFmt = VertexAttributeFmt {
        comp_type: VertexAttributeType::$comp_type,
        dim: VertexAttributeDim::$dim,
        unit_size: $crate::vertex::size_of::<$q>(),
        align: $crate::vertex::align_of::<$q>(),
      };
    }
  };

  ($t:ty, $comp_type:ident) => {
    impl_vertex_attribute!($t, $t, $comp_type, Dim1);
    impl_vertex_attribute!([$t; 1], $t, $comp_type, Dim1);
    impl_vertex_attribute!([$t; 2], $t, $comp_type, Dim2);
    impl_vertex_attribute!([$t; 3], $t, $comp_type, Dim3);
    impl_vertex_attribute!([$t; 4], $t, $comp_type, Dim4);
  };
}

impl_vertex_attribute!(i8, Integral);
impl_vertex_attribute!(i16, Integral);
impl_vertex_attribute!(i32, Integral);
impl_vertex_attribute!(u8, Unsigned);
impl_vertex_attribute!(u16, Unsigned);
impl_vertex_attribute!(u32, Unsigned);
impl_vertex_attribute!(f32, Floating);
impl_vertex_attribute!(f64, Floating);
impl_vertex_attribute!(bool, Floating);

macro_rules! impl_vertex_for_tuple {
  ($($t:tt),+) => {
    unsafe impl<'a, $($t),+> Vertex<'a> for ($($t),+) where $($t: 'a + VertexAttribute),+ {
      type Deinterleaved = ($(&'a [$t]),*);

      const VERTEX_FMT: VertexFmt =
        &[
          $(
            $t::VERTEX_ATTRIBUTE_FMT
          ),*
        ];
    }
  }
}

macro_rules! impl_vertex_for_tuples {
  ($a:tt, $b:tt) => {
    impl_vertex_for_tuple!($a, $b);
  };

  ($first_t:tt, $($t:tt),+) => {
    impl_vertex_for_tuples!($($t),+);
    impl_vertex_for_tuple!($first_t, $($t),+);
  }
}

impl_vertex_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
