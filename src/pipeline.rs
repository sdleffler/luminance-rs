//! Dynamic rendering pipelines.
//!
//! This module gives you materials to build *dynamic* rendering **pipelines**. A `Pipeline`
//! represents a functional stream that consumes geometric data and rasterizes them.

use blending;
use framebuffer::{ColorSlot, DepthSlot, Framebuffer, HasFramebuffer};
use shader::program::{HasProgram, Program};
use tessellation::{HasTessellation, Tessellation};
use texture::{Dimensionable, HasTexture, Layerable};

/// Trait to implement to add `Pipeline` support.
//pub trait HasPipeline: HasFramebuffer + HasProgram + HasTessellation + HasTexture + Sized {
//  fn run_pipeline<L, D, CS, DS>(cmd: &Pipeline<Self, L, D, CS, DS>)
//    where L: Layerable,
//          D: Dimensionable,
//          D::Size: Copy,
//          CS: ColorSlot<Self, L, D>,
//          DS: DepthSlot<Self, L, D>;
//}

/// Run a `Pipeline`.
///
/// `L` refers to the `Layering` of the underlying `Framebuffer`.
///
/// `D` refers to the `Dim` of the underlying `Framebuffer`.
///
/// `CS` and `DS` are – respectively – the *color* and *depth* `Slot` of the underlying
/// `Framebuffer`.
//pub fn run_pipeline<C, L, D, CS, DS>(cmd: &Pipeline<C, L, D, CS, DS>)
//    where C: HasPipeline,
//          L: Layerable,
//          D: Dimensionable,
//          D::Size: Copy,
//          CS: ColorSlot<C, L, D>,
//          DS: DepthSlot<C, L, D> {
//  C::run_pipeline(cmd);
//}

pub fn run_pipeline() {}

/// A dynamic rendering pipeline. A *pipeline* is responsible of rendering into a `Framebuffer`.
///
/// `L` refers to the `Layering` of the underlying `Framebuffer`.
///
/// `D` refers to the `Dim` of the underlying `Framebuffer`.
///
/// `CS` and `DS` are – respectively – the *color* and *depth* `Slot` of the underlying
/// `Framebuffer`.
pub struct Pipeline<'a, C, L, D, CS, DS> 
    where C: 'a + HasFramebuffer + HasProgram + HasTessellation + HasTexture + EraseShadingCommand,
          L: 'a + Layerable,
          D: 'a + Dimensionable,
          D::Size: Copy,
          CS: 'a + ColorSlot<C, L, D>,
          DS: 'a + DepthSlot<C, L, D> {
  pub framebuffer: &'a Framebuffer<C, L, D, CS, DS>,
  pub clear_color: [f32; 4],
  pub shading_commands: ErasedShadingCommand<'a>
}

impl<'a, C, L, D, CS, DS> Pipeline<'a, C, L, D, CS, DS>
    where C: HasFramebuffer + HasProgram + HasTessellation + HasTexture + EraseShadingCommand,
          L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          CS: ColorSlot<C, L, D>,
          DS: DepthSlot<C, L, D> {
  pub fn new<T>(framebuffer: &'a Framebuffer<C, L, D, CS, DS>, clear_color: [f32; 4], shading_commands: Vec<ErasedShadingCommand<'a>>) -> Self {
    // erase shading commands
    let run_shading_commands = Box::new(move || {
      for cmd in shading_commands {
      }
    });

    Pipeline {
      framebuffer: framebuffer,
      clear_color: clear_color,
      shading_commands: run_shading_commands
    }
  }
}

/// Type erasure over `ShadingCommand`. The resulting closure is used to run a shading command.
pub type ErasedShadingCommand<'a> = Box<Fn() + 'a>;

pub trait EraseShadingCommand: HasProgram + HasTessellation + Sized {
  fn erase_shading_command<T>(shading_cmd: ShadingCommand<Self, T>) -> Box<Fn()>;
}

/// A dynamic *shading command*. A shading command gathers *render commands* under a shader
/// `Program`.
pub struct ShadingCommand<'a, C, T> where C: 'a + HasProgram + HasTessellation, T: 'a {
  pub program: &'a Program<C, T>,
  pub update: Box<Fn(&T) + 'a>,
  pub render_commands: Vec<RenderCommand<'a, C, T>>
}

impl<'a, C, T> ShadingCommand<'a, C, T> where C: 'a + HasProgram + HasTessellation {
  pub fn new<F: Fn(&T) + 'a>(program: &'a Program<C, T>, update: F, render_commands: Vec<RenderCommand<'a, C, T>>) -> Self {
    ShadingCommand {
      program: &program,
      update: Box::new(update),
      render_commands: render_commands
    }
  }
}

/// A render command, which holds information on how to rasterize tessellation.
pub struct RenderCommand<'a, C, T> where C: 'a + HasTessellation {
  pub blending: Option<(blending::Equation, blending::Factor, blending::Factor)>,
  pub depth_test: bool,
  pub update: Box<Fn(&T) + 'a>,
  pub tessellation: &'a Tessellation<C>,
  pub instances: u32,
  pub rasterization_size: Option<f32>
}

impl<'a, C, T> RenderCommand<'a, C, T> where C: 'a + HasTessellation {
  pub fn new<F: Fn(&T) + 'a>(blending: Option<(blending::Equation, blending::Factor, blending::Factor)>, depth_test: bool, update: F, tessellation: &'a Tessellation<C>, instances: u32, rasterization_size: Option<f32>) -> Self {
    RenderCommand {
      blending: blending,
      depth_test: depth_test,
      update: Box::new(update),
      tessellation: tessellation,
      instances: instances,
      rasterization_size: rasterization_size
    }
  }
}
