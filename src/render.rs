use blending;
use framebuffer::{ColorSlot, DepthSlot, Framebuffer, HasFramebuffer};
use shader::program::{HasProgram, Program};
use tessellation::{HasTessellation, Tessellation};
use texture::{Dimensionable, HasTexture, Layerable};

pub trait HasFrameCommand: HasFramebuffer + HasProgram + HasTessellation + HasTexture + Sized {
  fn run_frame_command<L, D, CS, DS>(cmd: &FrameCommand<Self, L, D, CS, DS>)
    where L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          CS: ColorSlot<Self, L, D>,
          DS: DepthSlot<Self, L, D>;
}

pub fn run_frame_command<C, L, D, CS, DS>(cmd: &FrameCommand<C, L, D, CS, DS>)
    where C: HasFrameCommand,
          L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          CS: ColorSlot<C, L, D>,
          DS: DepthSlot<C, L, D> {
  C::run_frame_command(cmd);
}

pub struct FrameCommand<'a, C, L, D, CS, DS> 
    where C: 'a + HasFramebuffer + HasProgram + HasTessellation + HasTexture,
          L: 'a + Layerable,
          D: 'a + Dimensionable,
          D::Size: Copy,
          CS: 'a + ColorSlot<C, L, D>,
          DS: 'a + DepthSlot<C, L, D> {
  pub framebuffer: &'a Framebuffer<C, L, D, CS, DS>,
  pub shading_commands: Vec<ShadingCommand<'a, C>>
}

impl<'a, C, L, D, CS, DS> FrameCommand<'a, C, L, D, CS, DS>
    where C: HasFramebuffer + HasProgram + HasTessellation + HasTexture,
          L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          CS: ColorSlot<C, L, D>,
          DS: DepthSlot<C, L, D> {
  pub fn new(framebuffer: &'a Framebuffer<C, L, D, CS, DS>, shading_commands: Vec<ShadingCommand<'a, C>>) -> Self {
    FrameCommand {
      framebuffer: framebuffer,
      shading_commands: shading_commands
    }
  }
}

pub struct ShadingCommand<'a, C> where C: 'a + HasProgram + HasTessellation {
  pub program: &'a C::Program,
  pub update: Box<Fn() + 'a>,
  pub render_commands: Vec<RenderCommand<'a, C>>
}

impl<'a, C> ShadingCommand<'a, C> where C: 'a + HasProgram + HasTessellation {
  pub fn new<F: Fn() + 'a>(program: &'a Program<C>, update: F, render_commands: Vec<RenderCommand<'a, C>>) -> Self {
    ShadingCommand {
      program: &program.repr,
      update: Box::new(update),
      render_commands: render_commands
    }
  }
}

pub struct RenderCommand<'a, C> where C: 'a + HasTessellation {
  pub blending: Option<(blending::Equation, blending::Factor, blending::Factor)>,
  pub depth_test: bool,
  pub update: Box<Fn() + 'a>,
  pub tessellation: &'a Tessellation<C>,
  pub instances: u32,
  pub rasterization_size: Option<f32>
}

impl<'a, C> RenderCommand<'a, C> where C: 'a + HasTessellation {
  pub fn new<F: Fn() + 'a>(blending: Option<(blending::Equation, blending::Factor, blending::Factor)>, depth_test: bool, update: F, tessellation: &'a Tessellation<C>, instances: u32, rasterization_size: Option<f32>) -> Self {
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
