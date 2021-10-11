//! > This program is a sequel to 08-shader-uniforms-adapt. Be sure to have read it first.
//!
//! This example shows you how to lookup dynamically uniforms into shaders to implement various kind
//! of situations. This feature is very likely to be interesting for anyone who would like to
//! implement a GUI, where the interface of the shader programs are not known statically, for
//! instance.
//!
//! This example looks up the time and the triangle position on the fly, without using the uniform
//! interface.
//!
//! Press the <left>, <right>, <up>, <down> actions to move the triangle on the screen.
//!
//! https://docs.rs/luminance

use crate::{
  shared::{Semantics, Vertex, VertexColor, VertexPosition},
  Example, InputAction, LoopFeedback, PlatformServices,
};
use luminance_front::{
  context::GraphicsContext,
  framebuffer::Framebuffer,
  pipeline::PipelineState,
  render_state::RenderState,
  shader::{types::Vec2, Program},
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

pub struct LocalExample {
  program: Program<Semantics, (), ()>,
  triangle: Tess<Vertex>,
  triangle_pos: Vec2<f32>,
}

impl Example for LocalExample {
  fn bootstrap(
    _: &mut impl PlatformServices,
    context: &mut impl GraphicsContext<Backend = Backend>,
  ) -> Self {
    // notice that we don’t set a uniform interface here: we’re going to look it up on the fly
    let program = context
      .new_shader_program::<Semantics, (), ()>()
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

        InputAction::Left => {
          self.triangle_pos[0] -= 0.1;
        }

        InputAction::Right => {
          self.triangle_pos[0] += 0.1;
        }

        InputAction::Forward => {
          self.triangle_pos[1] += 0.1;
        }

        InputAction::Backward => {
          self.triangle_pos[1] -= 0.1;
        }

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
          shd_gate.shade(program, |mut iface, _, mut rdr_gate| {
            let mut query = iface.query().unwrap();
            let time_u = query.ask::<f32>("t");
            let triangle_pos_u = query.ask::<Vec2<f32>>("triangle_pos");

            if let Ok(ref time_u) = time_u {
              iface.set(time_u, t);
            }

            if let Ok(ref triangle_pos_u) = triangle_pos_u {
              iface.set(triangle_pos_u, triangle_pos);
            }

            // the `ask` function is type-safe: if you try to get a uniform which type is not
            // correctly reified from the source, you get a TypeMismatch runtime error
            //if let Err(e) = query.ask::<i32>("triangle_pos") {
            //  eprintln!("{:?}", e);
            //}

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
