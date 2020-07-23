//! This example demonstrates how to _map_ GPU tessellations to alter their attributes. It consists
//! of a triangle that you can move around by grabbing one of its three vertices and moving them
//! around.
//!
//! Press down the left click of your mouse / trackpad when your mouse is close to a vertex to grab
//! it, move it around and release the left click to put it at the place your cursor is.
//! Press <escape> to quit or close the window.
//!
//! https://docs.rs/luminance

mod common;

use common::{Semantics, Vertex, VertexColor, VertexPosition};
use glfw::{Action, Context as _, Key, MouseButton, WindowEvent};
use luminance::context::GraphicsContext as _;
use luminance::pipeline::PipelineState;
use luminance::render_state::RenderState;
use luminance::tess::Mode;
use luminance_glfw::GlfwSurface;
use luminance_windowing::{WindowDim, WindowOpt};

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

fn main() {
  let dim = WindowDim::Windowed {
    width: 960,
    height: 540,
  };
  let mut surface = GlfwSurface::new_gl33(
    "Hello, world; from OpenGL 3.3!",
    WindowOpt::default().set_dim(dim),
  )
  .expect("GLFW surface creation");

  let mut program = surface
    .new_shader_program::<Semantics, (), ()>()
    .from_strings(VS, None, None, FS)
    .expect("program creation")
    .ignore_warnings();

  let mut triangle = surface
    .new_tess()
    .set_vertices(&TRI_VERTICES[..])
    .set_mode(Mode::Triangle)
    .build()
    .unwrap();

  let mut back_buffer = surface.back_buffer().unwrap();
  let mut resize = false;

  // current cursor position
  let mut cursor_pos = None;

  // when we press down a button, if we are to select a vertex, we need to know which one; this
  // variable contains its index (0, 1 or 2)
  let mut selected = None;

  'app: loop {
    surface.window.glfw.poll_events();
    for (_, event) in surface.events_rx.try_iter() {
      match event {
        WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => break 'app,

        // if we press down the left button, we want to check whether a vertex is nearby; to do so,
        // we map the triangle’s vertices and look for one; we take the one with the minimal
        // distance that satisfies the distance rule defined at the top of this file
        // (CLICK_RADIUS_PX)
        WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => {
          if let Some(ref cursor_pos) = cursor_pos {
            let vertices = triangle.vertices().unwrap();

            for i in 0..3 {
              let (w, h) = surface.window.get_framebuffer_size();

              // convert the vertex position from screen space into window space
              let ws_pos = screen_to_window(&vertices[i].pos, w as _, h as _);

              if distance(&ws_pos, cursor_pos) <= CLICK_RADIUS_PX {
                println!("selecting vertex i={}", i);
                selected = Some(i);
                break;
              }
            }
          }
        }

        WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _) => {
          selected = None;
        }

        // update the cursor position when we move it
        WindowEvent::CursorPos(x, y) => {
          let pos = [x as _, y as _];
          cursor_pos = Some(pos);

          if let Some(selected) = selected {
            let mut vertices = triangle.vertices_mut().unwrap();
            let (w, h) = surface.window.get_framebuffer_size();

            vertices[selected].pos = VertexPosition::new(window_to_screen(&pos, w as _, h as _));
          }
        }

        WindowEvent::FramebufferSize(..) => {
          resize = true;
        }

        _ => (),
      }
    }

    if resize {
      back_buffer = surface.back_buffer().unwrap();
      resize = false;
    }

    let render = surface
      .new_pipeline_gate()
      .pipeline(
        &back_buffer,
        &PipelineState::default(),
        |_, mut shd_gate| {
          shd_gate.shade(&mut program, |_, _, mut rdr_gate| {
            rdr_gate.render(&RenderState::default(), |mut tess_gate| {
              tess_gate.render(&triangle)
            })
          })
        },
      )
      .assume();

    // Finally, swap the backbuffer with the frontbuffer in order to render our triangles onto your
    // screen.
    if render.is_ok() {
      surface.window.swap_buffers();
    } else {
      break 'app;
    }
  }
}
