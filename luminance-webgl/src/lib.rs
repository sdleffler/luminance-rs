//! WebGL backend for luminance.
//!
//! This crate provides a [luminance] backend for [WebGL].
//!
//! [luminance]: https://crates.io/crates/luminance
//! [WebGL]: https://www.khronos.org/webgl

extern crate serde_derive;

#[macro_use]
mod slice;
pub mod webgl2;
