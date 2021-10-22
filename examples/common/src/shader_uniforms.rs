//! This program shows how to render a triangle and change its position and color on the fly by
//! updating “shader uniforms”. Those are values stored on the GPU that remain constant for the
//! whole duration of a draw call (you typically change it between each draw call to customize each
//! draw).
//!
//! This example demonstrate how to add time to your shader to start building moving and animated
//! effects.
//!
//! Press the <up action>, <down action>, <left action> and <right action> to move the triangle on
//! the screen.
//!
//! <https://docs.rs/luminance>

use crate::{
  shared::{Semantics, Vertex, VertexColor, VertexPosition},
  Example, InputAction, LoopFeedback, PlatformServices,
};
use luminance::UniformInterface;
use luminance_front::{
  context::GraphicsContext,
  framebuffer::Framebuffer,
  pipeline::PipelineState,
  render_state::RenderState,
  shader::{types::Vec2, Program, Uniform},
  tess::{Mode, Tess},
  texture::Dim2,
  Backend,
};

const VS: &'static str = include_str!("displacement-vs.glsl");
const FS: &'static str = include_str!("displacement-fs.glsl");

// Only one triangle this time.
const TRI_VERTICES: [Vertex; 3] = [
  Vertex {
    pos: VertexPosition::new([0.5, -0.5]),
    rgb: VertexColor::new([1., 0., 0.]),
  },
  Vertex {
    pos: VertexPosition::new([0.0, 0.5]),
    rgb: VertexColor::new([0., 1., 0.]),
  },
  Vertex {
    pos: VertexPosition::new([-0.5, -0.5]),
    rgb: VertexColor::new([0., 0., 1.]),
  },
];

// Create a uniform interface. This is a type that will be used to customize the shader. In our
// case, we just want to pass the time and the position of the triangle, for instance.
//
// This macro only supports structs for now; you cannot use enums as uniform interfaces.
#[derive(Debug, UniformInterface)]
struct ShaderInterface {
  #[uniform(name = "t")]
  time: Uniform<f32>,
  triangle_pos: Uniform<Vec2<f32>>,
}

pub struct LocalExample {
  program: Program<Semantics, (), ShaderInterface>,
  triangle: Tess<Vertex>,
  triangle_pos: Vec2<f32>,
}

impl Example for LocalExample {
  fn bootstrap(
    _platform: &mut impl PlatformServices,
    context: &mut impl GraphicsContext<Backend = Backend>,
  ) -> Self {
    let program = context
      .new_shader_program()
      .from_strings(VS, None, None, FS)
      .expect("program creation")
      .ignore_warnings();
    let triangle = context
      .new_tess()
      .set_vertices(&TRI_VERTICES[..])
      .set_mode(Mode::Triangle)
      .build()
      .unwrap();
    let triangle_pos = Vec2::new(0., 0.);

    Self {
      program,
      triangle,
      triangle_pos,
    }
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
        InputAction::Left => self.triangle_pos[0] -= 0.1,
        InputAction::Right => self.triangle_pos[0] += 0.1,
        InputAction::Forward => self.triangle_pos[1] += 0.1,
        InputAction::Backward => self.triangle_pos[1] -= 0.1,
        _ => (),
      }
    }

    let program = &mut self.program;
    let triangle = &self.triangle;
    let triangle_pos = self.triangle_pos;

    let render = context
      .new_pipeline_gate()
      .pipeline(
        &back_buffer,
        &PipelineState::default(),
        |_, mut shd_gate| {
          // notice the iface free variable, which type is &ShaderInterface
          shd_gate.shade(program, |mut iface, uni, mut rdr_gate| {
            // update the time and triangle position on the GPU shader program
            iface.set(&uni.time, t);
            iface.set(&uni.triangle_pos, triangle_pos);

            rdr_gate.render(&RenderState::default(), |mut tess_gate| {
              // render the dynamically selected slice
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
