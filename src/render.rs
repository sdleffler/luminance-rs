use blending;
use framebuffer::{ColorSlot, DepthSlot, Framebuffer, HasFramebuffer};
use rw::Writable;
use shader::program::HasProgram;
use tessellation::HasTessellation;
use texture::HasTexture;

pub trait HasFrameCommand: HasFramebuffer + HasProgram + HasTessellation + HasTexture + Sized {
  fn run_frame_command<A, CS, DS>(cmd: &FrameCommand<Self, A, CS, DS>)
    where A: Writable,
          CS: ColorSlot,
          DS: DepthSlot;
}

pub struct FrameCommand<C, A, CS, DS> 
    where C: HasFramebuffer + HasProgram + HasTessellation + HasTexture,
          CS: ColorSlot,
          DS: DepthSlot {
  pub framebuffer: Framebuffer<C, A, CS, DS>,
  pub shading_commands: Vec<ShadingCommand<C>>
}

pub struct ShadingCommand<C> where C: HasProgram + HasTessellation {
  pub program: C::Program,
  pub update: Box<Fn()>,
  pub render_commands: Vec<RenderCommand<C>>
}

pub struct RenderCommand<C> where C: HasTessellation {
  pub blending: Option<(blending::Equation, blending::Factor, blending::Factor)>,
  pub depth_test: bool,
  pub update: Box<Fn()>,
  pub tessellation: C::Tessellation
}
