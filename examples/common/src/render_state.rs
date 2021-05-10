//! This program shows how to tweak the render state in order to render two simple triangles with
//! different parameters.
//!
//! From this tutorial on, vertex types and semantics are taken from a common.rs file.
//!
//! Press the <main action> to switch which triangle is rendered atop of which.
//! Press the <auxiliary action> to activate additive blending or disable it.
//!
//! https://docs.rs/luminance

use crate::{
  shared::{Semantics, Vertex, VertexColor, VertexPosition},
  Example, InputAction, LoopFeedback, PlatformServices,
};
use luminance_front::{
  blending::{Blending, Equation, Factor},
  context::GraphicsContext,
  framebuffer::Framebuffer,
  pipeline::PipelineState,
  render_state::RenderState,
  shader::Program,
  tess::{Mode, Tess},
  texture::Dim2,
  Backend,
};

const VS: &'static str = include_str!("simple-vs.glsl");
const FS: &'static str = include_str!("simple-fs.glsl");

pub const TRI_RED_BLUE_VERTICES: [Vertex; 6] = [
  // first triangle – a red one
  Vertex {
    pos: VertexPosition::new([0.5, -0.5]),
    rgb: VertexColor::new([1., 0., 0.]),
  },
  Vertex {
    pos: VertexPosition::new([0.0, 0.5]),
    rgb: VertexColor::new([1., 0., 0.]),
  },
  Vertex {
    pos: VertexPosition::new([-0.5, -0.5]),
    rgb: VertexColor::new([1., 0., 0.]),
  },
  // second triangle, a blue one
  Vertex {
    pos: VertexPosition::new([-0.5, 0.5]),
    rgb: VertexColor::new([0., 0., 1.]),
  },
  Vertex {
    pos: VertexPosition::new([0.0, -0.5]),
    rgb: VertexColor::new([0., 0., 1.]),
  },
  Vertex {
    pos: VertexPosition::new([0.5, 0.5]),
    rgb: VertexColor::new([0., 0., 1.]),
  },
];

// Convenience type to demonstrate how the depth test influences the rendering of two triangles.
#[derive(Copy, Clone, Debug)]
enum DepthMethod {
  Under, // draw the red triangle under the blue one
  Atop,  // draw the red triangle atop the blue one
}

impl DepthMethod {
  fn toggle(self) -> Self {
    match self {
      DepthMethod::Under => DepthMethod::Atop,
      DepthMethod::Atop => DepthMethod::Under,
    }
  }
}

// toggle between no blending and additive blending
fn toggle_blending(blending: Option<Blending>) -> Option<Blending> {
  match blending {
    None => Some(Blending {
      equation: Equation::Additive,
      src: Factor::One,
      dst: Factor::One,
    }),
    _ => None,
  }
}

pub struct LocalExample {
  program: Program<Semantics, (), ()>,
  red_triangle: Tess<Vertex>,
  blue_triangle: Tess<Vertex>,
  blending: Option<Blending>,
  depth_method: DepthMethod,
}

impl Example for LocalExample {
  fn bootstrap(
    _platform: &mut impl PlatformServices,
    context: &mut impl GraphicsContext<Backend = Backend>,
  ) -> Self {
    let program = context
      .new_shader_program::<Semantics, (), ()>()
      .from_strings(VS, None, None, FS)
      .expect("program creation")
      .ignore_warnings();

    // create a red and blue triangles
    let red_triangle = context
      .new_tess()
      .set_vertices(&TRI_RED_BLUE_VERTICES[0..3])
      .set_mode(Mode::Triangle)
      .build()
      .unwrap();
    let blue_triangle = context
      .new_tess()
      .set_vertices(&TRI_RED_BLUE_VERTICES[3..6])
      .set_mode(Mode::Triangle)
      .build()
      .unwrap();

    let blending = None;
    let depth_method = DepthMethod::Under;

    Self {
      program,
      red_triangle,
      blue_triangle,
      blending,
      depth_method,
    }
  }

  fn render_frame(
    &mut self,
    _time: f32,
    back_buffer: Framebuffer<Dim2, (), ()>,
    actions: impl Iterator<Item = InputAction>,
    context: &mut impl GraphicsContext<Backend = Backend>,
  ) -> LoopFeedback {
    for action in actions {
      match action {
        InputAction::Quit => return LoopFeedback::Exit,

        InputAction::MainToggle => {
          self.depth_method = self.depth_method.toggle();
          log::info!("now rendering {:?}", self.depth_method);
        }

        InputAction::AuxiliaryToggle => {
          self.blending = toggle_blending(self.blending);
          log::info!("now blending with {:?}", self.blending);
        }

        _ => (),
      }
    }

    let program = &mut self.program;
    let red_triangle = &self.red_triangle;
    let blue_triangle = &self.blue_triangle;
    let blending = self.blending;
    let depth_method = self.depth_method;

    let render = context
      .new_pipeline_gate()
      .pipeline(
        &back_buffer,
        &PipelineState::default(),
        |_, mut shd_gate| {
          shd_gate.shade(program, |_, _, mut rdr_gate| {
            let render_state = RenderState::default()
            // let’s disable the depth test so that every fragment (i.e. pixels) will be rendered to every
            // time we have to draw a part of a triangle
            .set_depth_test(None)
            // set the blending we decided earlier
            .set_blending(blending);

            rdr_gate.render(&render_state, |mut tess_gate| match depth_method {
              DepthMethod::Under => {
                tess_gate.render(red_triangle)?;
                tess_gate.render(blue_triangle)
              }

              DepthMethod::Atop => {
                tess_gate.render(blue_triangle)?;
                tess_gate.render(red_triangle)
              }
            })
          })
        },
      )
      .assume();

    if render.is_ok() {
      LoopFeedback::Continue
    } else {
      LoopFeedback::Exit
    }
  }
}
