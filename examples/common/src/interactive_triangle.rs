//! This example demonstrates how to _map_ GPU tessellations to alter their attributes. It consists
//! of a triangle that you can move around by grabbing one of its three vertices and moving them
//! around.
//!
//! Press down the left click of your mouse / trackpad when your mouse is close to a vertex to grab
//! it, move it around and release the left click to put it at the place your cursor is.
//! Press <escape> to quit or close the window.
//!
//! <https://docs.rs/luminance>

use crate::{
  shared::{Semantics, Vertex, VertexColor, VertexPosition},
  Example, InputAction, LoopFeedback, PlatformServices,
};
use luminance_front::{
  context::GraphicsContext,
  framebuffer::Framebuffer,
  pipeline::PipelineState,
  render_state::RenderState,
  shader::Program,
  tess::{Mode, Tess},
  texture::Dim2,
  Backend,
};

const VS: &str = include_str!("simple-vs.glsl");
const FS: &str = include_str!("simple-fs.glsl");

const TRI_VERTICES: [Vertex; 3] = [
  Vertex::new(
    VertexPosition::new([0.5, -0.5]),
    VertexColor::new([0., 1., 0.]),
  ),
  Vertex::new(
    VertexPosition::new([0.0, 0.5]),
    VertexColor::new([0., 0., 1.]),
  ),
  Vertex::new(
    VertexPosition::new([-0.5, -0.5]),
    VertexColor::new([1., 0., 0.]),
  ),
];

// when selecting a vertex, the vertex is “snapped” only if the distance (in pixels) between the
// position of the click and the vertex is minimal to this value (expressed in pixels, since it’s a
// distance)
const CLICK_RADIUS_PX: f32 = 20.;

// a simple convenient function to compute a distance between two [f32; 2]
fn distance(a: &[f32; 2], b: &[f32; 2]) -> f32 {
  let x = b[0] - a[0];
  let y = b[1] - a[1];

  (x * x + y * y).sqrt()
}

// convert from screen space to window space
fn screen_to_window(a: &[f32; 2], w: f32, h: f32) -> [f32; 2] {
  [(1. + a[0]) * 0.5 * w as f32, (1. - a[1]) * 0.5 * h as f32]
}

// convert from window space to screen space
fn window_to_screen(a: &[f32; 2], w: f32, h: f32) -> [f32; 2] {
  [a[0] / w * 2. - 1., 1. - a[1] / h * 2.]
}

pub struct LocalExample {
  program: Program<Semantics, (), ()>,
  triangle: Tess<Vertex>,
  // current cursor position
  cursor_pos: Option<[f32; 2]>,
  // when we press down a button, if we are to select a vertex, we need to know which one; this
  // variable contains its index (0, 1 or 2)
  selected: Option<usize>,
  // used to perform window-space / screen-space coordinates conversion
  window_dim: [f32; 2],
}

impl Example for LocalExample {
  fn bootstrap(
    _: &mut impl PlatformServices,
    context: &mut impl GraphicsContext<Backend = Backend>,
  ) -> Self {
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

    let cursor_pos = None;
    let selected = None;

    Self {
      program,
      triangle,
      cursor_pos,
      selected,
      window_dim: [800., 800.],
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
        InputAction::Quit => return LoopFeedback::Exit,

        // if we press down the primary action, we want to check whether a vertex is nearby; to do so,
        // we map the triangle’s vertices and look for one; we take the one with the minimal
        // distance that satisfies the distance rule defined at the top of this file
        // (CLICK_RADIUS_PX)
        InputAction::PrimaryPressed => {
          if let Some(ref cursor_pos) = self.cursor_pos {
            let vertices = self.triangle.vertices().unwrap();

            for i in 0..3 {
              let [w, h] = self.window_dim;

              // convert the vertex position from screen space into window space
              let ws_pos = screen_to_window(&vertices[i].pos, w, h);

              if distance(&ws_pos, cursor_pos) <= CLICK_RADIUS_PX {
                println!("selecting vertex i={}", i);
                self.selected = Some(i);
                break;
              }
            }
          }
        }

        InputAction::PrimaryReleased => {
          self.selected = None;
        }

        InputAction::CursorMoved { x, y } => {
          let pos = [x, y];
          self.cursor_pos = Some(pos);

          if let Some(selected) = self.selected {
            let mut vertices = self.triangle.vertices_mut().unwrap();
            let [w, h] = self.window_dim;

            vertices[selected].pos = VertexPosition::new(window_to_screen(&pos, w, h));
          }
        }

        InputAction::Resized { width, height } => {
          self.window_dim = [width as _, height as _];
        }

        _ => (),
      }
    }

    let program = &mut self.program;
    let triangle = &self.triangle;

    let render = context
      .new_pipeline_gate()
      .pipeline(
        &back_buffer,
        &PipelineState::default(),
        |_, mut shd_gate| {
          shd_gate.shade(program, |_, _, mut rdr_gate| {
            rdr_gate.render(&RenderState::default(), |mut tess_gate| {
              tess_gate.render(triangle)
            })
          })
        },
      )
      .assume();

    // Finally, swap the backbuffer with the frontbuffer in order to render our triangles onto your
    // screen.
    if render.is_ok() {
      LoopFeedback::Continue(self)
    } else {
      LoopFeedback::Exit
    }
  }
}
