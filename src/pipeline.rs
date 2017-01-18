//! Dynamic rendering pipelines.
//!
//! This module gives you materials to build *dynamic* rendering **pipelines**. A `Pipeline`
//! represents a functional stream that consumes geometric data and rasterizes them.

use buffer::UniformBufferProxy;
use blending;
use framebuffer::{ColorSlot, DepthSlot, Framebuffer, HasFramebuffer};
use shader::program::{HasProgram, Program};
use tess::{HasTess, Tess};
use texture::{Dimensionable, HasTexture, Layerable, TextureProxy};

/// Trait to implement to add `Pipeline` support.
pub trait HasPipeline: HasFramebuffer + HasProgram + HasTess + HasTexture + Sized {
  /// Execute a pipeline command, resulting in altering the embedded framebuffer.
  fn run_pipeline<L, D, CS, DS>(cmd: &Pipeline<Self, L, D, CS, DS>)
    where L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          CS: ColorSlot<Self, L, D>,
          DS: DepthSlot<Self, L, D>;
  /// Execute a shading command.
  fn run_shading_command<'a>(shading_cmd: &Pipe<'a, Self, ShadingCommand<Self>>);
}

/// A dynamic rendering pipeline. A *pipeline* is responsible of rendering into a `Framebuffer`.
///
/// `L` refers to the `Layering` of the underlying `Framebuffer`.
///
/// `D` refers to the `Dim` of the underlying `Framebuffer`.
///
/// `CS` and `DS` are – respectively – the *color* and *depth* `Slot` of the underlying
/// `Framebuffer`.
pub struct Pipeline<'a, C, L, D, CS, DS>
    where C: 'a + HasFramebuffer + HasProgram + HasTess + HasTexture,
          L: 'a + Layerable,
          D: 'a + Dimensionable,
          D::Size: Copy,
          CS: 'a + ColorSlot<C, L, D>,
          DS: 'a + DepthSlot<C, L, D> {
  /// The embedded framebuffer.
  pub framebuffer: &'a Framebuffer<C, L, D, CS, DS>,
  /// The color used to clean the framebuffer when  executing the pipeline.
  pub clear_color: [f32; 4],
  /// Texture set.
  pub texture_set: &'a[TextureProxy<'a, C>],
  /// Buffer set.
  pub buffer_set: &'a[UniformBufferProxy<'a>],
  /// Shading commands to render into the embedded framebuffer.
  pub shading_commands: Vec<Pipe<'a, C, ShadingCommand<'a, C>>>
}

impl<'a, C, L, D, CS, DS> Pipeline<'a, C, L, D, CS, DS>
    where C: HasPipeline,
          L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          CS: ColorSlot<C, L, D>,
          DS: DepthSlot<C, L, D> {
  /// Create a new pipeline.
  pub fn new(framebuffer: &'a Framebuffer<C, L, D, CS, DS>, clear_color: [f32; 4],
             texture_set: &'a[TextureProxy<'a, C>], buffer_set: &'a[UniformBufferProxy<'a>],
             shading_commands: Vec<Pipe<'a, C, ShadingCommand<'a, C>>>) -> Self {
    Pipeline {
      framebuffer: framebuffer,
      clear_color: clear_color,
      texture_set: texture_set,
      buffer_set: buffer_set,
      shading_commands: shading_commands
    }
  }

  /// Run a `Pipeline`.
  pub fn run(&self) {
    C::run_pipeline(self);
  }
}

/// A dynamic *shading command*. A shading command gathers *render commands* under a shader
/// `Program`.
pub struct ShadingCommand<'a, C> where C: 'a + HasProgram + HasTess {
  /// Embedded program.
  pub program: &'a Program<C>,
  /// Render commands to execute for this shading command.
  pub render_commands: Vec<Pipe<'a, C, RenderCommand<'a, C>>>
}

impl<'a, C> ShadingCommand<'a, C> where C: 'a + HasProgram + HasTess {
  /// Create a new shading command.
  pub fn new(program: &'a Program<C>, render_commands: Vec<Pipe<'a, C, RenderCommand<'a, C>>>) -> Self {
    ShadingCommand {
      program: program,
      render_commands: render_commands
    }
  }
}

/// A render command, which holds information on how to rasterize tessellations.
pub struct RenderCommand<'a, C> where C: 'a + HasProgram + HasTess {
  /// Color blending configuration. Set to `None` if you don’t want any color blending. Set it to
  /// `Some(equation, source, destination)` if you want to perform a color blending with the
  /// `equation` formula and with the `source` and `destination` blending factors.
  pub blending: Option<(blending::Equation, blending::Factor, blending::Factor)>,
  /// Should a depth test be performed?
  pub depth_test: bool,
  /// The embedded tessellations.
  pub tessellations: Vec<Pipe<'a, C, &'a Tess<C>>>,
  /// Number of instances of the tessellation to render.
  pub instances: u32,
  /// Rasterization size for points and lines.
  pub rasterization_size: Option<f32>
}

impl<'a, C> RenderCommand<'a, C> where C: 'a + HasProgram + HasTess {
  /// Create a new render command.
  pub fn new(blending: Option<(blending::Equation, blending::Factor, blending::Factor)>,
             depth_test: bool, tessellations: Vec<Pipe<'a, C, &'a Tess<C>>>, instances: u32,
             rasterization_size: Option<f32>) -> Self {
    RenderCommand {
      blending: blending,
      depth_test: depth_test,
      tessellations: tessellations,
      instances: instances,
      rasterization_size: rasterization_size
    }
  }
}

/// A pipe used to build up a `Pipeline` by connecting its inner layers.
pub struct Pipe<'a, C, T> where C: HasProgram {
  pub update_program: Box<Fn(&Program<C>) + 'a>,
  pub next: T
}

impl<'a, C, T> Pipe<'a, C, T> where C: HasProgram {
  pub fn new<F>(update_program: F, next: T) -> Self where F: Fn(&Program<C>) + 'a {
    Pipe {
      update_program: Box::new(update_program),
      next: next
    }
  }
}
