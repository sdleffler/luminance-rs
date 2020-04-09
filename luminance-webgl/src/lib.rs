//! WebGL backend for luminance.
//!
//! This crate provides a [luminance] backend for [WebGL].
//!
//! [luminance]: https://crates.io/crates/luminance
//! [WebGL]: https://www.khronos.org/webgl

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate stdweb;

#[macro_use]
extern crate stdweb_derive;

pub mod webgl2;
