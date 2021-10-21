//! OpenGL backends.
//!
//! This crate exportes [OpenGL](https://www.khronos.org/opengl/) backends for
//! [luminance](https://crates.io/crates/luminance). This crate can be used via two mechanisms:
//!
//! - Automatically selected for you by [luminance-front](https://crates.io/crates/luminance-front). This is the option
//!   you should probably go to in most of the cases, where you want the compiler to pick the backend type for you
//!   without caring too much about it.
//! - Manually picked. In this case, you will want to browse the content of this crate to use a _backend type_.

pub mod gl33;

pub use gl33::GL33;
