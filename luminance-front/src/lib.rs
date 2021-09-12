//! [luminance], but with a backend type picked at compile-time.
//!
//! This crate re-exports _aliases_ to all [luminance] types, methods, and any kind of symbols
//! requiring a backend type variable (typically written `B`) by selecting the proper type based on
//! the platform you target and/or feature flags. Selection is done mainly automatically in the
//! `Cargo.toml` file, and can customize on a per-target basis which backend you want to select.
//!
//! > Important note: the existence of this crate was made a need so that people who don’t care
//! > about genericity can start using [luminance] without having to worry about backend
//! > implementations. It’s the case for people writing small binaries, “final” graphics libraries
//! > and/or 2D/3D/animation engines, etc. If you are writing a [luminance] middleware, please
//! > stick to the [luminance] crate and its polymorphic types.
//!
//! Some symbols are re-exported even though they are not polymorphic in a backend type variable in
//! [luminance]. That is only for convenience purposes.
//!
//! # Documentation
//!
//! Because this crate re-exports the content of [luminance], you are strongly advised to go read
//! the documentation on [luminance]. Documentation will not be duplicated here.
//!
//! # How to setup
//!
//! For a starter experience, you have nothing specific to do: simply add `luminance-front` as a
//! direct dependency and you should be good to go:
//!
//! ```ignore
//! [dependencies]
//! luminance-front = "…"
//! ```
//!
//! This will select a _default_ backend implementation for the target you currently compile for.
//! See the list of features below for further information.
//!
//! To switch target to use, you are advised to either put a `.cargo/config` file in a directory
//! inside your project, or compile with the `--target` option.
//!
//! The default setup will provide a default implementation that should work great on as
//! many machines as possible for all supported targets. If for some reason you want to pick another
//! implementation (for instance an older version of WebGL, OpenGL or an experimental, more modern
//! implementation), you will have to use specific platform features, such as `"gl33"`.
//!
//! ```ignore
//! [dependencies]
//! luminance-front = { version = "…", no-default-features = true, features = ["gl33", "webgl2"] }
//! ```
//!
//! As you can see, you can specify features for different targets at the same time. Target
//! features are checked in the `lib.rs`, so it’s possible to define both OpenGL and WebGL
//! features at the same time. The current target will narrow down which one to use.
//!
//! ## List of features
//!
//! - _Default_: `["gl33", "webgl2"]`.
//! - **OpenGL**:
//!   - `"gl33"`: OpenGL 3.3 implementation.
//! - **WebGL 2**:
//!   - `"webgl2"`: WebGL 2 implementation.
//!
//! [luminance]: https://crates.io/crates/luminance

pub mod context;
pub mod framebuffer;
pub mod pipeline;
pub mod query;
pub mod render_gate;
pub mod shader;
pub mod shading_gate;
pub mod tess;
pub mod tess_gate;
pub mod texture;

// re-export
pub use luminance::blending;
pub use luminance::depth_test;
pub use luminance::face_culling;
pub use luminance::pixel;
pub use luminance::render_state;
pub use luminance::scissor;
pub use luminance::vertex;

// select the backend type

#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), feature = "gl33"))]
pub type Backend = luminance_gl::GL33;

#[cfg(all(target_arch = "wasm32", feature = "webgl2"))]
pub type Backend = luminance_webgl::webgl2::WebGL2;
