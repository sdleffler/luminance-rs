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

pub struct FrameCommand<'a, C, L, D, CS, DS> 
    where C: 'a + HasFramebuffer + HasProgram + HasTessellation + HasTexture,
          L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          CS: ColorSlot<C, L, D>,
          DS: DepthSlot<C, L, D> {
  pub framebuffer: Framebuffer<C, L, D, CS, DS>,
  pub shading_commands: &'a [ShadingCommand<'a, C>]
}

pub struct ShadingCommand<'a, C> where C: 'a + HasProgram + HasTessellation {
  pub program: C::Program,
  pub update: Box<Fn()>,
  pub render_commands: &'a [RenderCommand<C>]
}

pub struct RenderCommand<C> where C: HasTessellation {
  pub blending: Option<(blending::Equation, blending::Factor, blending::Factor)>,
  pub depth_test: bool,
  pub update: Box<Fn()>,
  pub tessellation: C::Tessellation
}
