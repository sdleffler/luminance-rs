//! Generalized free tuples.
//!
//! The `Chain` type is used to create chain of types. Because **Rust** doesn’t support type
//! operators, `Chain` is used to create static tuples that can have an arbitrary size while
//! remaining of the same form. That’s very useful to implement traits for any kind of types chain.
//! Feel free to dig in the documention of `Chain` for further details.
//!
//! Plus, if your compiler supports the `type_macros`*feature*, you can use the `chain!` macro to
//! build types chain without having to nest `Chain` in each others, which is very handy.

/// The generalized free tuple.
///
/// You can create arbitrary chains by nesting `Chain` types, or use the `chain!` type macro if your
/// compiler allows you to.
///
/// # Examples
///
/// ```
/// type Foo = Chain<i32,Chain<bool,f32>>;
/// type Bar = Chain<Chain<i32, bool>, f32>;
/// type Zoo = chain![i32, bool, f32]; // Zoo == Foo
/// ```
pub struct Chain<A, B>(A, B);

/// If your compiler supports the `type_macros`*feature*, you can use this macro to create chains
/// without the syntactic nesting boilerplate.
///
/// # Examples
///
/// ```
/// type Foo = Chain<i32,Chain<bool,f32>>;
/// type Zoo = chain![i32, bool, f32]; // exactly the same type
/// ```
#[macro_export]
macro_rules! chain {
  ($t:ty) => {( $t )};
  
  ($a:ty, $($r:tt)*) => {( Chain<$a, chain![$($r)*]> )}
}

