use crate::{Example, InputAction, LoopFeedback, PlatformServices};
use cgmath::{Matrix4, Vector3};
use luminance::{Semantics, Vertex};
use luminance_front::{
  context::GraphicsContext,
  framebuffer::Framebuffer,
  pipeline::PipelineState,
  render_state::RenderState,
  shader::{types::Mat44, BuiltProgram, Program},
  tess::{Interleaved, Mode, Tess},
  texture::Dim2,
  Backend,
};

const VS: &str = "
in vec2 co;

uniform mat4 translation_mat;
uniform float aspect_ratio;

void main() {
  vec2 p = co;
  p.y *= aspect_ratio;
  gl_Position = translation_mat * vec4(p, 0., 1.);
}
";

const FS: &str = "
out vec4 frag;

void main() {
  frag = vec4(1., .5, .5, 1.);
}
";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Semantics)]
pub enum Semantics {
  // - Reference vertex positions with the "co" variable in vertex shaders.
  // - The underlying representation is [f32; 2], which is a vec2 in GLSL.
  // - The wrapper type you can use to handle such a semantics is VertexPosition.
  #[sem(name = "co", repr = "[f32; 2]", wrapper = "VertexPosition")]
  Position,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "Semantics")]
struct Vertex {
  pos: VertexPosition,
}

// The vertices.
const TRI_VERTICES: [Vertex; 3] = [
  Vertex::new(VertexPosition::new([0.5, -0.5])),
  Vertex::new(VertexPosition::new([0.0, 0.5])),
  Vertex::new(VertexPosition::new([-0.5, -0.5])),
];

pub struct LocalExample {
  program: Program<Semantics, (), ()>,
  triangle: Tess<Vertex, (), (), Interleaved>,
}

impl Example for LocalExample {
  fn bootstrap(
    _: &mut impl PlatformServices,
    context: &mut impl GraphicsContext<Backend = Backend>,
  ) -> Self {
    let BuiltProgram { program, warnings } = context
      .new_shader_program::<Semantics, (), ()>()
      .from_strings(VS, None, None, FS)
      .expect("program creation");

    for warning in warnings {
      log::info!("{}", warning);
    }

    let triangle = context
      .new_tess()
      .set_vertices(&TRI_VERTICES[..])
      .set_mode(Mode::Triangle)
      .build()
      .unwrap();

    LocalExample { program, triangle }
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
        InputAction::Quit => return LoopFeedback::Exit,
        _ => (),
      }
    }

    let translation_mat = Matrix4::from_translation(Vector3::new(0.5, 0., 0.));
    let program = &mut self.program;
    let triangle = &self.triangle;

    let render = context
      .new_pipeline_gate()
      .pipeline(
        &back_buffer,
        &PipelineState::default(),
        |_, mut shd_gate| {
          shd_gate.shade(program, |mut iface, _, mut rdr_gate| {
            let uni = iface.query().unwrap().ask("translation_mat").unwrap();
            let mat = Mat44::new(translation_mat);
            iface.set(&uni, mat);

            // aspect ratio
            let [width, height] = back_buffer.size();
            let aspect_ratio_uni = iface.query().unwrap().ask("aspect_ratio").unwrap();
            iface.set(&aspect_ratio_uni, width as f32 / height as f32);

            rdr_gate.render(&RenderState::default(), |mut tess_gate| {
              tess_gate.render(triangle)
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
