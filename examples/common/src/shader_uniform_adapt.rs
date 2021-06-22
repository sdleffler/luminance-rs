//! > This program is a sequel to 04-shader-uniforms. Be sure to have read it first.
//!
//! This example shows you how to change the type of a shader program’s interface on the fly without
//! changing the GPU object. This might be wanted whenever you need to use a different type which
//! fields overlap the former type you used, or to implement a form of dynamic introspection. By
//! readapting the uniform interface (to the same type), you can use a *value-driven* approach to
//! add new uniforms on the fly, which comes in very handy when writing UI systems for instance.
//!
//! The program should start black so press space and enjoy.
//!
//! Press the <left>, <right>, <up>, <down> actions to move the triangle on the screen.
//! Press the <main> action to switch between uniform interfaces.
//! Press <escape> to quit or close the window.
//!
//! https://docs.rs/luminance

use crate::{
  shared::{Semantics, Vertex, VertexColor, VertexPosition},
  Example, InputAction, LoopFeedback, PlatformServices,
};
use luminance::{context::GraphicsContext, UniformInterface};
use luminance_front::{
  framebuffer::Framebuffer,
  pipeline::PipelineState,
  render_state::RenderState,
  shader::{AdaptationFailure, Program, Uniform},
  tess::{Mode, Tess},
  texture::Dim2,
  Backend,
};

const VS: &'static str = include_str!("adapt-vs.glsl");
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

/// First uniform interface.
#[derive(Debug, UniformInterface)]
struct ShaderInterface1 {
  #[uniform(name = "t")]
  time: Uniform<f32>,
  triangle_size: Uniform<f32>,
}

/// Second uniform interface.
#[derive(Debug, UniformInterface)]
struct ShaderInterface2 {
  #[uniform(name = "t")]
  time: Uniform<f32>,
  triangle_pos: Uniform<[f32; 2]>,
}

// Which interface to use?
enum ProgramMode {
  First(Program<Semantics, (), ShaderInterface1>),
  Second(Program<Semantics, (), ShaderInterface2>),
}

impl ProgramMode {
  fn toggle(self) -> Self {
    match self {
      ProgramMode::First(p) => match p.adapt() {
        Ok(program) => ProgramMode::Second(program.ignore_warnings()),
        Err(AdaptationFailure { program, error }) => {
          eprintln!("unable to switch to second uniform interface: {:?}", error);
          ProgramMode::First(program)
        }
      },

      ProgramMode::Second(p) => match p.adapt() {
        Ok(program) => ProgramMode::First(program.ignore_warnings()),
        Err(AdaptationFailure { program, error }) => {
          eprintln!("unable to switch to first uniform interface: {:?}", error);
          ProgramMode::Second(program)
        }
      },
    }
  }
}

pub struct LocalExample {
  program: ProgramMode,
  triangle: Tess<Vertex>,
  triangle_pos: [f32; 2],
}

impl Example for LocalExample {
  fn bootstrap(
    _: &mut impl PlatformServices,
    context: &mut impl GraphicsContext<Backend = Backend>,
  ) -> Self {
    let program = ProgramMode::First(
      context
        .new_shader_program::<Semantics, (), ShaderInterface1>()
        .from_strings(VS, None, None, FS)
        .expect("program creation")
        .ignore_warnings(),
    );

    let triangle = context
      .new_tess()
      .set_vertices(&TRI_VERTICES[..])
      .set_mode(Mode::Triangle)
      .build()
      .unwrap();

    let triangle_pos = [0., 0.];

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

        InputAction::MainToggle => {
          self.program = self.program.toggle();
        }

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
          match program {
            // if we use the first interface, we just need to pass the time and the triangle position
            ProgramMode::First(ref mut program) => {
              shd_gate.shade(program, |mut iface, uni, mut rdr_gate| {
                iface.set(&uni.time, t);
                iface.set(&uni.triangle_size, t.cos().powf(2.));

                rdr_gate.render(&RenderState::default(), |mut tess_gate| {
                  tess_gate.render(triangle)
                })
              })
            }

            // if we use the second interface, we just need to pass the time and we will make the size
            // grow by using the time
            ProgramMode::Second(ref mut program) => {
              shd_gate.shade(program, |mut iface, uni, mut rdr_gate| {
                iface.set(&uni.time, t);
                // iface.set(&uni.triangle_size, t.cos().powf(2.)); // uncomment this to see a nice error ;)
                iface.set(&uni.triangle_pos, triangle_pos);

                rdr_gate.render(&RenderState::default(), |mut tess_gate| {
                  tess_gate.render(triangle)
                })
              })
            }
          }
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
