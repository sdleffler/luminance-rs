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

use luminance::context::GraphicsContext;
use luminance_front::{framebuffer::Framebuffer, texture::Dim2, Backend};

pub mod attributeless;
pub mod dynamic_uniform_interface;
pub mod hello_world;
pub mod offscreen;
pub mod render_state;
pub mod shader_uniform_adapt;
pub mod shader_uniforms;
pub mod shared;
pub mod sliced_tess;
pub mod texture;
pub mod vertex_instancing;

/// Example interface.
pub trait Example: Sized {
  /// List of features required by the example.
  fn features() -> Features {
    Features::default()
  }

  /// Bootstrap the example.
  fn bootstrap(
    platform: &mut impl PlatformServices,
    context: &mut impl GraphicsContext<Backend = Backend>,
  ) -> Self;

  /// Render a frame of the example.
  fn render_frame(
    self,
    time: f32,
    back_buffer: Framebuffer<Dim2, (), ()>,
    actions: impl Iterator<Item = InputAction>,
    context: &mut impl GraphicsContext<Backend = Backend>,
  ) -> LoopFeedback<Self>;
}

/// Feature set.
///
/// Features allow to provide more interactivity in the examples.
pub struct Features {
  textures: Vec<String>,
}

impl Default for Features {
  fn default() -> Self {
    Features {
      textures: Vec::new(),
    }
  }
}

impl Features {
  /// Create an empty feature set.
  pub fn none() -> Self {
    Features::default()
  }

  /// Add a texture to be loaded.
  pub fn texture(self, name: impl Into<String>) -> Self {
    let mut textures = self.textures;
    textures.push(name.into());

    Features { textures, ..self }
  }

  /// List of textures to be loaded.
  pub fn textures(&self) -> &[String] {
    &self.textures
  }
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
pub enum LoopFeedback<T> {
  Continue(T),
  Exit,
}

/// Various services provided by the platform.
pub trait PlatformServices {
  type FetchError: Error;

  /// Fetch a texture given its name.
  fn fetch_texture(&mut self, name: impl AsRef<str>) -> Result<&image::RgbImage, Self::FetchError>;
}
