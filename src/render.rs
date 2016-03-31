use blending;
use framebuffer::{ColorSlot, DepthSlot, Framebuffer, HasFramebuffer};
use shader::program::HasProgram;
use tessellation::HasTessellation;
use texture::{Dimensionable, HasTexture, Layerable};

pub trait HasFrameCommand: HasFramebuffer + HasProgram + HasTessellation + HasTexture + Sized {
  fn run_frame_command<L, D, CS, DS>(cmd: FrameCommand<Self, L, D, CS, DS>)
    where L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          CS: ColorSlot<Self, L, D>,
          DS: DepthSlot<Self, L, D>;
}

pub fn run_frame_command<C, L, D, CS, DS>(cmd: FrameCommand<C, L, D, CS, DS>)
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
  pub shading_commands: &'a [ShadingCommand<'a, C>]
}

/*
impl<'a, C, L, D, CS, DS> FrameCommand<'a, C, L, D, CS, DS>
    where C: 'a + HasFramebuffer + HasProgram + HasTessellation + HasTexture,
          L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          CS: ColorSlot<C, L, D>,
          DS: DepthSlot<C, L, D> {
	fn new(framebuffer: Framebuffer<C, L, D, CS, DS>
}
*/

pub struct ShadingCommand<'a, C> where C: 'a + HasProgram + HasTessellation {
  pub program: &'a C::Program,
  pub update: Box<Fn()>,
  pub render_commands: &'a [RenderCommand<'a, C>]
}

pub struct RenderCommand<'a, C> where C: 'a + HasTessellation {
  pub blending: Option<(blending::Equation, blending::Factor, blending::Factor)>,
  pub depth_test: bool,
  pub update: Box<Fn()>,
  pub tessellation: &'a C::Tessellation,
  pub instances: u32,
  pub rasterization_size: Option<f32>
}
