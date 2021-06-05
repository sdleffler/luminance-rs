//! This program demonstrates how to render a triangle without sending anything to the GPU. This is
//! a not-so-famous technique to reduce the bandwidth and procedurally generate all the required
//! data to perform the render. The trick lives in ordering the GPU to render a certain number of
//! vertices and “spawn” the vertices’ data directly in the vertex shader by using the identifier of
//! the vertex currently being mapped over.
//!
//! Press <escape> to quit or close the window.
//!
//! https://docs.rs/luminance

use crate::{Example, InputAction, LoopFeedback, PlatformServices};
use luminance_front::{
  context::GraphicsContext,
  pipeline::PipelineState,
  render_state::RenderState,
  shader::Program,
  tess::{Mode, Tess},
};

const VS: &'static str = include_str!("attributeless-vs.glsl");
const FS: &'static str = include_str!("simple-fs.glsl");

pub struct LocalExample {
  program: Program<(), (), ()>,
  tess: Tess<()>,
}

impl Example for LocalExample {
  fn bootstrap(
    _platform: &mut impl PlatformServices,
    context: &mut impl GraphicsContext<Backend = luminance_front::Backend>,
  ) -> Self {
    // we don’t use a Vertex type anymore (i.e. attributeless, so we use the unit () type)
    let program = context
      .new_shader_program::<(), (), ()>()
      .from_strings(VS, None, None, FS)
      .expect("program creation")
      .ignore_warnings();

    // yet, we still need to tell luminance to render a certain number of vertices (even if we send no
    // attributes / data); in our case, we’ll just render a triangle, which has three vertices
    let tess = context
      .new_tess()
      .set_render_vertex_nb(3)
      .set_mode(Mode::Triangle)
      .build()
      .unwrap();

    Self { program, tess }
  }

  fn render_frame(
    mut self,
    _time: f32,
    back_buffer: luminance_front::framebuffer::Framebuffer<luminance::texture::Dim2, (), ()>,
    actions: impl Iterator<Item = InputAction>,
    context: &mut impl GraphicsContext<Backend = luminance_front::Backend>,
  ) -> LoopFeedback<Self> {
    for action in actions {
      match action {
        InputAction::Quit => return LoopFeedback::Exit,
        _ => (),
      }
    }

    let program = &mut self.program;
    let tess = &self.tess;

    let render = context
      .new_pipeline_gate()
      .pipeline(
        &back_buffer,
        &PipelineState::default(),
        |_, mut shd_gate| {
          shd_gate.shade(program, |_, _, mut rdr_gate| {
            rdr_gate.render(&RenderState::default(), |mut tess_gate| {
              // render the tessellation to the surface the regular way and let the vertex shader’s
              // magic do the rest!
              tess_gate.render(tess)
            })
          })
        },
      )
      .assume();

    if render.is_ok() {
      LoopFeedback::Continue(self)
    } else {
      LoopFeedback::Exit
    }
  }
}
