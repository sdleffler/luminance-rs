//! This program demonstrates how to render a triangle without sending anything to the GPU. This is
//! a not-so-famous technique to reduce the bandwidth and procedurally generate all the required
//! data to perform the render. The trick lives in ordering the GPU to render a certain number of
//! vertices and “spawn” the vertices’ data directly in the vertex shader by using the identifier of
//! the vertex currently being mapped over.
//!
//! Press <escape> to quit or close the window.
//!
//! https://docs.rs/luminance

use glfw::{Action, Context as _, Key, WindowEvent};
use luminance::context::GraphicsContext as _;
use luminance::pipeline::PipelineState;
use luminance::render_state::RenderState;
use luminance::tess::Mode;
use luminance_glfw::GlfwSurface;
use luminance_windowing::{WindowDim, WindowOpt};

const VS: &'static str = include_str!("attributeless-vs.glsl");
const FS: &'static str = include_str!("simple-fs.glsl");

fn main() {
  let dim = WindowDim::Windowed {
    width: 960,
    height: 540,
  };
  let surface = GlfwSurface::new_gl33("Hello, world!", WindowOpt::default().set_dim(dim))
    .expect("GLFW surface creation");
  let mut context = surface.context;
  let events = surface.events_rx;

  // we don’t use a Vertex type anymore (i.e. attributeless, so we use the unit () type)
  let mut program = context
    .new_shader_program::<(), (), ()>()
    .from_strings(VS, None, None, FS)
    .expect("program creation")
    .ignore_warnings();

  // yet, we still need to tell luminance to render a certain number of vertices (even if we send no
  // attributes / data); in our case, we’ll just render a triangle, which has three vertices
  let tess = context
    .new_tess()
    .set_vertex_nb(3)
    .set_mode(Mode::Triangle)
    .build()
    .unwrap();

  let mut back_buffer = context.back_buffer().unwrap();

  'app: loop {
    context.window.glfw.poll_events();
    for (_, event) in glfw::flush_messages(&events) {
      match event {
        WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => break 'app,

        WindowEvent::FramebufferSize(..) => {
          back_buffer = context.back_buffer().unwrap();
        }

        _ => (),
      }
    }

    let render = context
      .new_pipeline_gate()
      .pipeline(
        &back_buffer,
        &PipelineState::default(),
        |_, mut shd_gate| {
          shd_gate.shade(&mut program, |_, _, mut rdr_gate| {
            rdr_gate.render(&RenderState::default(), |mut tess_gate| {
              // render the tessellation to the surface the regular way and let the vertex shader’s
              // magic do the rest!
              tess_gate.render(&tess)
            })
          })
        },
      )
      .assume();

    if render.is_ok() {
      context.window.swap_buffers();
    } else {
      break 'app;
    }
  }
}
