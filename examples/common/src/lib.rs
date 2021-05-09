//! Luminance examples.
//!
//! This project provides a set of examples that can be run on any platform. The examples are made platform-agnostic on
//! purpose, so that running them on e.g. a WebGL or OpenGL backend can be done once for the whole set of examples.
//!
//! # Example architecture
//!
//! Examples are simple modules exposed from this crate. They do not depend on any platform-specific concepts, such as
//! system events or system window capacities. For that reason, whenever an example requires user interaction, an
//! abstract type is used from this crate, which is exposed by the platform code running the example.
//!
//! Examples are responsible in allocating the luminance resources and implementing any loop / one-shot effects by using
//! the [`Example`] trait.
//!
//! # Error handling
//!
//! Examples being examples, they showcase the happy path of the code, not the failure path. For this reason, for now,
//! errors are not handled in any way and just rely on using `.unwrap()` / `.expect()`. This is bad style and will
//! eventually change, so keep in mind that:
//!
//! - If you want to write solid and smart Rust code, you want to handle errors, not rely on panics.
//! - This is example code, so don’t blindly copy it, try to understand it first.

use luminance::context::GraphicsContext;
use luminance_front::{framebuffer::Framebuffer, texture::Dim2, Backend};

pub mod hello_world;
pub mod render_state;
pub mod shader_uniforms;
mod shared;
pub mod sliced_tess;

/// Example interface.
pub trait Example {
  /// Bootstrap the example.
  fn bootstrap(context: &mut impl GraphicsContext<Backend = Backend>) -> Self;

  /// Render a frame of the example.
  fn render_frame(
    &mut self,
    time: f32,
    back_buffer: Framebuffer<Dim2, (), ()>,
    actions: impl Iterator<Item = InputAction>,
    context: &mut impl GraphicsContext<Backend = Backend>,
  ) -> LoopFeedback;
}

/// A type used to pass “inputs” to examples.
#[derive(Clone, Debug)]
pub enum InputAction {
  /// Quit the application.
  Quit,

  /// Main action. Typically used to switch an effect on and off or to cycle through it.
  MainToggle,

  /// Auxiliary action. Often used to showcase / toggle smaller parts of a bigger effect.
  AuxiliaryToggle,

  /// Up direction. Typically used to move something up, move up, etc.
  Up,

  /// Down direction. Typically used to move something down, move down, etc.
  Down,

  /// Left direction. Typically used to move something left, move left, etc.
  Left,

  /// Right direction. Typically used to move something right, move right, etc.
  Right,

  /// Framebuffer size changed.
  Resized { width: u32, height: u32 },
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum LoopFeedback {
  Continue,
  Exit,
}
