//! Generalized free tuples.
//!
//! The `GTup` type is used to create generalized tuples. Because **Rust** doesn’t support type
//! operators, `GTup` is used to create static tuples that can have an arbitrary size while
//! remaining of the same recursive form. That’s very useful to implement traits for any kind of
//! tuples.
//!
//! Feel free to dig in the documention of `GTup` for further details.

/// The generalized free tuple.
///
/// You can create arbitrary chains by nesting `GTup` types, or use the `gtup!` type macro.
///
/// > Note: `GTup` is right-associative.
///
/// # Examples
///
/// ```
/// type Foo = GTup<i32, GTup<bool, f32>>;
/// type Bar = GTup<GTup<i32, bool>, f32>;
/// type Zoo = gtup![i32, bool, f32]; // Zoo == Foo
/// ```
pub struct GTup<A, B>(pub A, pub B);

#[macro_export]
macro_rules! gtup_ty {
    ($t:ty) => { $t };
    ($a:ty, $($r:tt)*) => { GTup<$a, gtup_ty!($($r)*)> }
}

#[macro_export]
macro_rules! gtup_value {
    ($t:expr) => { $t };
    ($a:expr, $($r:tt)*) => { GTup($a, gtup_value!($($r)*)) }
}

/// Generalized free tuple macro.
///
/// If your compiler supports the `type_macros`*feature*, you can use this macro to create tuples 
/// without the syntactic nesting boilerplate.
///
/// Furthermore, you can create values with this macro as well.
///
/// # Examples
///
/// ```
/// // type
/// type Foo = GTup<i32, Chain<bool, f32>>;
/// type Zoo = gtup!(:i32, bool, f32); // exactly the same type
///
/// // value
/// let triple = gtup!(42, false, 3.1415);
/// ```
#[macro_export]
macro_rules! gtup {
    (:$($a:tt)*) => { gtup_ty!($($a)*) };
    ($($a:tt)*) => { gtup_value!($($a)*) }
}
