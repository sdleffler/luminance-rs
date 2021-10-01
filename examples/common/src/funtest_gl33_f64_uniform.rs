use crate::{Example, InputAction, LoopFeedback, PlatformServices};
use luminance::UniformInterface;
use luminance_front::{
  context::GraphicsContext,
  framebuffer::Framebuffer,
  pipeline::PipelineState,
  render_state::RenderState,
  shader::{types::Vec3, Program, Uniform},
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
out vec3 frag;

uniform dvec3 color;

void main() {
  frag = vec3(color);
}";

#[derive(Debug, UniformInterface)]
struct ShaderInterface {
  color: Uniform<Vec3<f64>>,
}

pub struct LocalExample {
  program: Program<(), (), ShaderInterface>,
  tess: Tess<()>,
}

impl Example for LocalExample {
  fn bootstrap(
    _: &mut impl PlatformServices,
    context: &mut impl GraphicsContext<Backend = Backend>,
  ) -> Self {
    let program = context
      .new_shader_program::<(), (), ShaderInterface>()
      .from_strings(VS, None, None, FS)
      .unwrap()
      .ignore_warnings();

    let tess = context
      .new_tess()
      .set_mode(Mode::TriangleFan)
      .set_render_vertex_nb(4)
      .build()
      .unwrap();

    LocalExample { program, tess }
  }

  fn render_frame(
    mut self,
    t: f32,
    back_buffer: Framebuffer<Dim2, (), ()>,
    actions: impl Iterator<Item = InputAction>,
    context: &mut impl GraphicsContext<Backend = Backend>,
  ) -> LoopFeedback<Self> {
    for action in actions {
      match action {
        InputAction::Quit => return LoopFeedback::Exit,
        _ => (),
      }
    }

    let t = t as f64;
    let color = Vec3::new(t.cos(), 0.3, t.sin());
    let program = &mut self.program;
    let tess = &self.tess;

    let render = context
      .new_pipeline_gate()
      .pipeline(
        &back_buffer,
        &PipelineState::default(),
        |_, mut shd_gate| {
          shd_gate.shade(program, |mut iface, uni, mut rdr_gate| {
            iface.set(&uni.color, color);

            rdr_gate.render(&RenderState::default(), |mut tess_gate| {
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
