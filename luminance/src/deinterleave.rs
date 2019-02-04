//! Deinterleaving operations.
//!
//! *Deinterleaving* is the principle of taking *interleaved* data – often represented as contiguous
//! region memory (array or slice) – into several arrays. If you have, for instance, a struct as
//! such:
//!
//! ```
//! struct Point2D {
//!   xyz: [f32; 2],
//!   rgba: [f32; 4],
//! }
//! ```
//!
//! A slice of this struct (i.e. `&[Point2D]`) implies *interleaved data*. That is, if you look at
//! the raw memory your slice points to, you get something similar to:
//!
//! ```ignore
//! [x, y, z, r, g, b, a, x, y, z, r, g, b, a, …]
//! ```
//!
//! All fields are *interleaved* in memory. This is great when you actually want to work on all the
//! fields at once. Think about your CPU and its caches: if you want to iterate on such a slice,
//! using all fields, your CPU should find a correct way to load into its caches several `Point2D`
//! in advance and then allows you to implement a cache-friendly iteration loop.
//!
//! However, sometimes, you don’t care about some fields and just want to do something specific with
//! only, let’s say, the position of your `Point2D`. In this case, *interleaved* memory will waste
//! a lot of space in your caches and the algorithm might not be as cache-friendly as you would like
//! it to be.
//!
//! You then have a choice: pay the price of keeping the *interleaved* data or use its
//! *deinterleaved* representation. This module provides you with some traits and functions to use
//! *deinterleaved* data. As with *interleaved* data,  *deinterleaved* data only makes sense when
//! you talk about a collection or sequence of its data.
//!
//! Regarding the `Point2D` example from above, its *deinterleaved* representation could be, for
//! instance (using slices):
//!
//! ```
//! type DeinterleavedPoint2Ds = (&[f32; 2], &[f32; 4]);
//! ```
//!
//! Obviously, both the slices need to point originate from allocated containers (such as `Vec<_>`).
//! In memory, you have two address from which you can pick data (either positions or RGBA colors):
//!
//! ```ignore
//! positions: [x, y, z, x, y, z, x, y, z, …]
//! colors: [r, g, b, a, r, g, b, a, r, g, b, a, …]
//! ```
//!
//! If you want to recreate a `Point2D` at a given *index* in a deinterleaved representation, you
//! then need to ask your CPU to load all required data from different places from memory (this,
//! then, can be way more costful and non-cache-friendly).

pub unsafe trait Deinterleave: Sized {
  fn visit_deinterleave<V>(&self, visitor: &mut V)
  where V: SliceVisitor;
}

unsafe impl Deinterleave for () {
  fn visit_deinterleave<V>(&self, _: &mut V)
  where V: SliceVisitor {
  }
}

pub trait SliceVisitor {
  fn visit_slice<T>(&mut self, slice: &[T]);
}

macro_rules! impl_deinterleave_for_scalar {
  ($t:ty) => {
    unsafe impl<'a> Deinterleave for &'a [$t] {
      fn visit_deinterleave<V>(&self, visitor: &mut V)
      where V: SliceVisitor {
        visitor.visit_slice(self);
      }
    }
  };
}

macro_rules! impl_deinterleave_for_arr {
  ($t:ty) => {
    impl_deinterleave_for_scalar!([$t; 1]);
    impl_deinterleave_for_scalar!([$t; 2]);
    impl_deinterleave_for_scalar!([$t; 3]);
    impl_deinterleave_for_scalar!([$t; 4]);
  };
}

macro_rules! impl_deinterleave_for_tuple {
  ($($t:tt . $nth:tt),+) => {
    unsafe impl<'a, $($t),+> Deinterleave for ($(&'a [$t]),+) {
      fn visit_deinterleave<V>(&self, visitor: &mut V) where V: SliceVisitor {
        $(
          visitor.visit_slice(self.$nth);
        )+
      }
    }
  }
}

// scalars
impl_deinterleave_for_scalar!(i8);
impl_deinterleave_for_scalar!(i16);
impl_deinterleave_for_scalar!(i32);

impl_deinterleave_for_scalar!(u8);
impl_deinterleave_for_scalar!(u16);
impl_deinterleave_for_scalar!(u32);

impl_deinterleave_for_scalar!(f32);
impl_deinterleave_for_scalar!(f64);

impl_deinterleave_for_scalar!(bool);

// array
impl_deinterleave_for_arr!(i8);
impl_deinterleave_for_arr!(i16);
impl_deinterleave_for_arr!(i32);

impl_deinterleave_for_arr!(u8);
impl_deinterleave_for_arr!(u16);
impl_deinterleave_for_arr!(u32);

impl_deinterleave_for_arr!(f32);
impl_deinterleave_for_arr!(f64);

impl_deinterleave_for_arr!(bool);

impl_deinterleave_for_tuple!(A.0, B.1);
impl_deinterleave_for_tuple!(A.0, B.1, C.2);
impl_deinterleave_for_tuple!(A.0, B.1, C.2, D.3);
impl_deinterleave_for_tuple!(A.0, B.1, C.2, D.3, E.4);
impl_deinterleave_for_tuple!(A.0, B.1, C.2, D.3, E.4, F.5);
impl_deinterleave_for_tuple!(A.0, B.1, C.2, D.3, E.4, F.5, G.6);
impl_deinterleave_for_tuple!(A.0, B.1, C.2, D.3, E.4, F.5, G.6, H.7);
impl_deinterleave_for_tuple!(A.0, B.1, C.2, D.3, E.4, F.5, G.6, H.7, I.8);
impl_deinterleave_for_tuple!(A.0, B.1, C.2, D.3, E.4, F.5, G.6, H.7, I.8, J.9);
impl_deinterleave_for_tuple!(A.0, B.1, C.2, D.3, E.4, F.5, G.6, H.7, I.8, J.9, K.10);
impl_deinterleave_for_tuple!(A.0, B.1, C.2, D.3, E.4, F.5, G.6, H.7, I.8, J.9, K.10, L.11);
impl_deinterleave_for_tuple!(A.0, B.1, C.2, D.3, E.4, F.5, G.6, H.7, I.8, J.9, K.10, L.11, M.12);
impl_deinterleave_for_tuple!(A.0, B.1, C.2, D.3, E.4, F.5, G.6, H.7, I.8, J.9, K.10, L.11, M.12, N.13);
impl_deinterleave_for_tuple!(A.0, B.1, C.2, D.3, E.4, F.5, G.6, H.7, I.8, J.9, K.10, L.11, M.12, N.13, O.14);
impl_deinterleave_for_tuple!(
  A.0, B.1, C.2, D.3, E.4, F.5, G.6, H.7, I.8, J.9, K.10, L.11, M.12, N.13, O.14, P.15
);
