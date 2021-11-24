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

use std::error::Error;

use luminance::{
  backend::framebuffer::FramebufferBackBuffer, context::GraphicsContext, framebuffer::Framebuffer,
  texture::Dim2,
};
use luminance_front::Backend;

// examples
pub mod attributeless;
pub mod displacement_map;
pub mod dynamic_uniform_interface;
pub mod hello_world;
pub mod interactive_triangle;
pub mod mrt;
pub mod offscreen;
pub mod polymorphic_hello_world;
pub mod query_info;
pub mod query_texture_texels;
pub mod render_state;
pub mod shader_data;
pub mod shader_uniform_adapt;
pub mod shader_uniforms;
pub mod shared;
pub mod skybox;
pub mod sliced_tess;
pub mod stencil;
pub mod texture;
pub mod vertex_instancing;

// functional tests
#[cfg(feature = "funtest")]
pub mod funtest_360_manually_drop_framebuffer;
#[cfg(feature = "funtest")]
pub mod funtest_483_indices_mut_corruption;
#[cfg(feature = "funtest")]
pub mod funtest_flatten_slice;
#[cfg(all(feature = "funtest", feature = "funtest-gl33-f64-uniform"))]
pub mod funtest_gl33_f64_uniform;
#[cfg(feature = "funtest")]
pub mod funtest_pixel_array_encoding;
#[cfg(feature = "funtest")]
pub mod funtest_scissor_test;
#[cfg(feature = "funtest")]
pub mod funtest_tess_no_data;

/// Example interface.
pub trait Example<B = Backend>: Sized
where
  B: FramebufferBackBuffer,
{
  /// Bootstrap the example.
  fn bootstrap(
    platform: &mut impl PlatformServices,
    context: &mut impl GraphicsContext<Backend = B>,
  ) -> Self;

  /// Render a frame of the example.
  fn render_frame(
    self,
    time: f32,
    back_buffer: Framebuffer<B, Dim2, (), ()>,
    actions: impl Iterator<Item = InputAction>,
    context: &mut impl GraphicsContext<Backend = B>,
  ) -> LoopFeedback<Self>;
}

/// A type used to pass “inputs” to examples.
#[derive(Clone, Debug)]
pub enum InputAction {
  /// Quit the application.
  Quit,

  /// Primary action. Typically used to fire a weapon, select an object, etc. Typically used along with a position on
  /// screen.
  PrimaryPressed,

  /// Primary action. Typically used to fire a weapon, select an object, etc. Typically used along with a position on
  /// screen.
  PrimaryReleased,

  /// Main action. Typically used to switch an effect on and off or to cycle through it.
  MainToggle,

  /// Auxiliary action. Often used to showcase / toggle smaller parts of a bigger effect.
  AuxiliaryToggle,

  /// Forward direction. Typically used to move forward.
  Forward,

  /// Forward direction. Typically used to move backward.
  Backward,

  /// Left direction. Typically used to move something left, move left, etc.
  Left,

  /// Right direction. Typically used to move something right, move right, etc.
  Right,

  /// Up direction. Typically used to move something up, go up, etc.
  Up,

  /// Down direction. Typically used to move something down, go down, etc.
  Down,

  /// Cursor moved. The cursor is a 2D-coordinate pointer on the screen that can be actioned by moving a stick, a mouse,
  /// etc.
  CursorMoved { x: f32, y: f32 },

  /// Framebuffer size changed.
  Resized { width: u32, height: u32 },

  /// Vertical scrolling.
  VScroll { amount: f32 },
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum LoopFeedback<T> {
  Continue(T),
  Exit,
}

/// Various services provided by the platform.
pub trait PlatformServices {
  type FetchError: Error;

  /// Fetch the next texture, if available.
  fn fetch_texture(&mut self) -> Result<image::RgbImage, Self::FetchError>;
}
