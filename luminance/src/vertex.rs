//! Vertex formats and associated types and functions.
//!
//! A vertex is a type representing a point. It’s common to find vertex position, normals, colors or
//! even texture coordinates. However, you’re free to use whichever type you want.
//! Nevertheless, you’re limited to a range of types and dimensions. See `Type` and
//! `Dim` for further details.
//!
//! # `Vertex`
//!
//! ## Rules
//!
//! To be able to use a type as a vertex, you have to implement the `Vertex` trait. That trait
//! represents a mapping between your type and `VertexFormat`. A `VertexFormat` gives runtime hints
//! about your type and restricts the supported type. If you cannot map your type to `VertexFormat`,
//! that means you cannot use it as a `Vertex`.
//!
//! The rule is that your type should have a static size greater than 0 and less than or equal to 4.
//! It should also be either integral, unsigned, floating or boolean. If your type is a complex one
//! – for instance a `struct` – you have to recursively apply that rule to all its fields.
//! For instance, the tuple `(i32, bool)` implements `Vertex` by providing an implementation using
//! the ones of `i32` and `bool`.
//!
//! ## Components list
//!
//! As mentionned above, you can use tuples and structs as `Vertex`. If you look at the definition
//! of `VertexFormat`, you’ll notice that it’s a `Vec<VertexComponentFormat>`. That means simple
//! and primary types map to unit vectors – i.e. their size is 1 – but tuples and structs need
//! several `VertexComponentFormat`s to be represented, hence vectors with sizes greater than 1.
//!
//! # Generic implementation
//!
//! You have `Vertex` implementations for all the primary types that can be mapped to
//! `VertexFormat`. However, as it’s not possible to automatically implement `Vertex` for your
//! structure (yet?), a type is provided to help you design your vertex type so that you’re
//! automatically provided with a `Vertex` implementation if you use `Chain`.
//!
//! `Chain` is a special type used to represent static list of types. With that in hand, you can
//! easily create `Vertex` types and start using them without even implementing `Vertex`, as long as
//! you use `Vertex` types. Feel free to dig in the `Chain` documentation for further details.

use chain::Chain;
use std::vec::Vec;

/// A `VertexFormat` is a list of `VertexComponentFormat`s.
pub type VertexFormat = Vec<VertexComponentFormat>;

/// Retrieve the number of components in a `VertexFormat`.
pub fn vertex_format_size(vf: &[VertexComponentFormat]) -> usize {
  vf.len()
}

/// Vertex component format. It gives information on how vertices should be passed to the GPU and
/// optimized in buffers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VertexComponentFormat {
  /// Type of the component. See `Type` for further details.
  pub comp_type: Type,
  /// Dimension of the component. It should be in 1–4. See `Dim` for further details.
  pub dim: Dim,
  /// Size in bytes that a single element of the component takes. That is, if your component has
  /// a dimension set to 2, then the unit size should be the size of a single element (not two).
  pub unit_size: usize,
  /// Alignment of the component. The best advice is to respect what Rust does, so it’s highly
  /// recommended to use `::std::mem::align_of` to let it does the job for you.
  pub align: usize
}

/// Possible type of vertex components.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Type {
  Integral,
  Unsigned,
  Floating,
  Boolean
}

/// Possible dimension of vertex components.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Dim {
  Dim1,
  Dim2,
  Dim3,
  Dim4
}

/// A type that can be used as a `Vertex` has to implement that trait – it must provide a mapping
/// to `VertexFormat`.
///
/// If you’re not sure on how to implement that or if you want to use automatic types, feel free
/// to use the primary supported types and `Chain` or tuples.
pub trait Vertex {
  fn vertex_format() -> VertexFormat;
}

/// Macro used to implement the `Vertex` trait.
///
/// The first form implements `Vertex` for `$t`. `$q` refers to the inner representation of `$t` –
/// for instance, `[f32; N]` has `f32` as inner representation. `$comp_type` is the type of the
/// component (variant of `Type`) and `$dim` is the dimension (variant of `Dim`).
///
/// The second form implement `Vertex` for `$t` for types that have the same inner representation
/// than theirselves – usually, scalars.
macro_rules! impl_base {
  ($t:ty, $q:ty, $comp_type:ident, $dim:ident) => {
    impl Vertex for $t {
      fn vertex_format() -> VertexFormat {
        vec![VertexComponentFormat {
          comp_type: Type::$comp_type,
          dim: Dim::$dim,
          unit_size: ::std::mem::size_of::<$q>(),
          align: ::std::mem::align_of::<$q>()
        }]
      }
    }
  };

  ($t:ty, $comp_type:ident, $dim:ident) => {
    impl_base!($t, $t, $comp_type, $dim);
  }
}

macro_rules! impl_arr {
  ($t:ty, $q:ident) => {
    impl_base!([$t; 1], $t, $q, Dim1);
    impl_base!([$t; 2], $t, $q, Dim2);
    impl_base!([$t; 3], $t, $q, Dim3);
    impl_base!([$t; 4], $t, $q, Dim4);
  }
}

impl Vertex for () {
  fn vertex_format() -> VertexFormat {
    Vec::new()
  }
}

// scalars
impl_base!(i8, Integral, Dim1);
impl_base!(i16, Integral, Dim1);
impl_base!(i32, Integral, Dim1);

impl_base!(u8, Unsigned, Dim1);
impl_base!(u16, Unsigned, Dim1);
impl_base!(u32, Unsigned, Dim1);

impl_base!(f32, Floating, Dim1);
impl_base!(f64, Floating, Dim1);

impl_base!(bool, Floating, Dim1);

// arrays
impl_arr!(i8, Integral);
impl_arr!(i16, Integral);
impl_arr!(i32, Integral);

impl_arr!(u8, Unsigned);
impl_arr!(u16, Unsigned);
impl_arr!(u32, Unsigned);

impl_arr!(f32, Floating);
impl_arr!(f64, Floating);

impl_arr!(bool, Boolean);

impl<A, B> Vertex for Chain<A, B> where A: Vertex, B: Vertex {
  fn vertex_format() -> VertexFormat {
    let mut t = A::vertex_format();
    t.extend(B::vertex_format());
    t
  }
}

impl<A, B> Vertex for (A, B) where A: Vertex, B: Vertex {
  fn vertex_format() -> VertexFormat {
    Chain::<A, B>::vertex_format()
  }
}

impl<A, B, C> Vertex for (A, B, C) where A: Vertex, B: Vertex, C: Vertex {
  fn vertex_format() -> VertexFormat {
    Chain::<A, Chain<B, C>>::vertex_format()
  }
}

impl<A, B, C, D> Vertex for (A, B, C, D) where A: Vertex, B: Vertex, C: Vertex, D: Vertex {
  fn vertex_format() -> VertexFormat {
    Chain::<A, Chain<B, Chain<C, D>>>::vertex_format()
  }
}

impl<A, B, C, D, E> Vertex for (A, B, C, D, E) where A: Vertex, B: Vertex, C: Vertex, D: Vertex, E: Vertex {
  fn vertex_format() -> VertexFormat {
    Chain::<A, Chain<B, Chain<C, Chain<D, E>>>>::vertex_format()
  }
}

impl<A, B, C, D, E, F> Vertex for (A, B, C, D, E, F) where A: Vertex, B: Vertex, C: Vertex, D: Vertex, E: Vertex, F: Vertex {
  fn vertex_format() -> VertexFormat {
    Chain::<A, Chain<B, Chain<C, Chain<D, Chain<E, F>>>>>::vertex_format()
  }
}

impl<A, B, C, D, E, F, G> Vertex for (A, B, C, D, E, F, G) where A: Vertex, B: Vertex, C: Vertex, D: Vertex, E: Vertex, F: Vertex, G: Vertex {
  fn vertex_format() -> VertexFormat {
    Chain::<A, Chain<B, Chain<C, Chain<D, Chain<E, Chain<F, G>>>>>>::vertex_format()
  }
}

impl<A, B, C, D, E, F, G, H> Vertex for (A, B, C, D, E, F, G, H) where A: Vertex, B: Vertex, C: Vertex, D: Vertex, E: Vertex, F: Vertex, G: Vertex, H: Vertex {
  fn vertex_format() -> VertexFormat {
    Chain::<A, Chain<B, Chain<C, Chain<D, Chain<E, Chain<F, Chain<G, H>>>>>>>::vertex_format()
  }
}

impl<A, B, C, D, E, F, G, H, I> Vertex for (A, B, C, D, E, F, G, H, I) where A: Vertex, B: Vertex, C: Vertex, D: Vertex, E: Vertex, F: Vertex, G: Vertex, H: Vertex, I: Vertex {
  fn vertex_format() -> VertexFormat {
    Chain::<A, Chain<B, Chain<C, Chain<D, Chain<E, Chain<F, Chain<G, Chain<H, I>>>>>>>>::vertex_format()
  }
}

impl<A, B, C, D, E, F, G, H, I, J> Vertex for (A, B, C, D, E, F, G, H, I, J) where A: Vertex, B: Vertex, C: Vertex, D: Vertex, E: Vertex, F: Vertex, G: Vertex, H: Vertex, I: Vertex, J: Vertex {
  fn vertex_format() -> VertexFormat {
    Chain::<A, Chain<B, Chain<C, Chain<D, Chain<E, Chain<F, Chain<G, Chain<H, Chain<I, J>>>>>>>>>::vertex_format()
  }
}
