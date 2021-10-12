use crate::{Example, InputAction, LoopFeedback, PlatformServices};
use luminance_front::{
  context::GraphicsContext,
  framebuffer::Framebuffer,
  pipeline::PipelineState,
  render_state::RenderState,
  scissor::ScissorRegion,
  shader::Program,
  tess::{Mode, Tess},
  texture::Dim2,
  Backend,
};

const VS: &str = "
const vec2[4] POSITIONS = vec2[](
  vec2(-1., -1.),
  vec2( 1., -1.),
  vec2( 1.,  1.),
  vec2(-1.,  1.)
);

void main() {
  gl_Position = vec4(POSITIONS[gl_VertexID], 0., 1.);
}";

const FS: &str = "
out vec4 frag;

void main() {
  frag = vec4(1., .5, .5, 1.);
}";

pub struct LocalExample {
  program: Program<(), (), ()>,
  tess: Tess<()>,
  is_active: bool,
}

impl Example for LocalExample {
  fn bootstrap(
    _: &mut impl PlatformServices,
    context: &mut impl GraphicsContext<Backend = Backend>,
  ) -> Self {
    let program = context
      .new_shader_program::<(), (), ()>()
      .from_strings(VS, None, None, FS)
      .unwrap()
      .ignore_warnings();

    let tess = context
      .new_tess()
      .set_mode(Mode::TriangleFan)
      .set_render_vertex_nb(4)
      .build()
      .unwrap();

    LocalExample {
      program,
      tess,
      is_active: true,
    }
  }

  fn render_frame(
    mut self,
    _: f32,
    back_buffer: Framebuffer<Dim2, (), ()>,
    actions: impl Iterator<Item = InputAction>,
    context: &mut impl GraphicsContext<Backend = Backend>,
  ) -> LoopFeedback<Self> {
    for action in actions {
      match action {
        InputAction::PrimaryReleased => {
          self.is_active = !self.is_active;
          log::info!(
            "scissor test is {}",
            if self.is_active { "active" } else { "inactive" },
          );
        }

        InputAction::Quit => return LoopFeedback::Exit,
        _ => (),
      }
    }

    let [width, height] = back_buffer.size();
    let (w2, h2) = (width as u32 / 2, height as u32 / 2);
    let program = &mut self.program;
    let tess = &self.tess;
    let is_active = self.is_active;

    let render = context
      .new_pipeline_gate()
      .pipeline(
        &back_buffer,
        &PipelineState::default(),
        |_, mut shd_gate| {
          shd_gate.shade(program, |_, _, mut rdr_gate| {
            if is_active {
              let rdr_st = RenderState::default().set_scissor(ScissorRegion {
                x: w2 - w2 / 2,
                y: h2 - h2 / 2,
                width: w2,
                height: h2,
              });

              rdr_gate.render(&rdr_st, |mut tess_gate| tess_gate.render(tess))
            } else {
              rdr_gate.render(&RenderState::default(), |mut tess_gate| {
                tess_gate.render(tess)
              })
            }
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
