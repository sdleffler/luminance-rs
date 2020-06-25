//! This program shows how to render a single triangle into an offscreen framebuffer with two
//! target textures, and how to render the contents of these textures into the back
//! buffer (i.e. the screen), combining data from both.
//!
//! Press <escape> to quit or close the window.
//!
//! https://docs.rs/luminance

mod common;

use crate::common::{Semantics, Vertex, VertexColor, VertexPosition};
use glfw::{Action, Context as _, Key, WindowEvent};
use luminance::context::GraphicsContext as _;
use luminance::framebuffer::Framebuffer;
use luminance::pipeline::{PipelineState, TextureBinding};
use luminance::pixel::{Floating, RGBA32F};
use luminance::render_state::RenderState;
use luminance::shader::{BuiltProgram, Uniform};
use luminance::tess::Mode;
use luminance::texture::{Dim2, Sampler};
use luminance_derive::UniformInterface;
use luminance_glfw::GlfwSurface;
use luminance_windowing::{WindowDim, WindowOpt};

// we get the shader at compile time from local files
const VS: &'static str = include_str!("simple-vs.glsl");
const FS: &'static str = include_str!("multi-fs.glsl");

// copy shader, at compile time as well
const COPY_VS: &'static str = include_str!("copy-vs.glsl");
const COPY_FS: &'static str = include_str!("copy-multi-fs.glsl");

// a single triangle is enough here
const TRI_VERTICES: [Vertex; 3] = [
  // triangle – an RGB one
  Vertex {
    pos: VertexPosition::new([0.5, -0.5]),
    rgb: VertexColor::new([0., 1., 0.]),
  },
  Vertex {
    pos: VertexPosition::new([0.0, 0.5]),
    rgb: VertexColor::new([0., 0., 1.]),
  },
  Vertex {
    pos: VertexPosition::new([-0.5, -0.5]),
    rgb: VertexColor::new([1., 0., 0.]),
  },
];

// the shader uniform interface is defined there
#[derive(UniformInterface)]
struct ShaderInterface {
  // we only need the source texture (from the framebuffer) to fetch from
  #[uniform(name = "source_texture_color")]
  texture_color: Uniform<TextureBinding<Dim2, Floating>>,
  #[uniform(name = "source_texture_white")]
  texture_white: Uniform<TextureBinding<Dim2, Floating>>,
}

fn main() {
  let mut surface = GlfwSurface::new_gl33(
    WindowDim::Windowed {
      width: 960,
      height: 540,
    },
    "Hello, world!",
    WindowOpt::default(),
  )
  .expect("GLFW surface creation");

  let mut program = match surface
    .new_shader_program::<Semantics, (), ()>()
    .from_strings(VS, None, None, FS) {
        Ok(BuiltProgram {
            program,
            warnings,
        }) => {
            for warning in &warnings {
                eprintln!("triangle shader warning: {:?}", warning);
            }
            program
        }
        Err(e) => {
            panic!("{}", e);
        }
    };

  let mut copy_program = match surface
    .new_shader_program::<(), (), ShaderInterface>()
    .from_strings(COPY_VS, None, None, COPY_FS) {
        Ok(BuiltProgram {
            program,
            warnings,
        }) => {
            for warning in &warnings {
                eprintln!("copy shader warning: {:?}", warning);
            }
            program
        }
        Err(e) => {
            panic!("{}", e);
        }
    };

  let triangle = surface
    .new_tess()
    .set_vertices(&TRI_VERTICES[..])
    .set_mode(Mode::Triangle)
    .build()
    .unwrap();

  // we’ll need an attributeless quad to fetch in full screen
  let quad = surface
    .new_tess()
    .set_vertex_nb(4)
    .set_mode(Mode::TriangleFan)
    .build()
    .unwrap();

  // “screen“ we want to render into our offscreen render
  let mut back_buffer = surface.back_buffer().unwrap();

  // offscreen buffer with two color slots that we will render in the first place
  let (w, h) = surface.window.get_framebuffer_size();
  let mut offscreen_buffer = surface
    .new_framebuffer::<Dim2, (RGBA32F, RGBA32F), ()>([w as u32, h as u32], 0, Sampler::default())
    .expect("framebuffer creation");

  // hack to update the offscreen buffer if needed; this is needed because we cannot update the
  // offscreen buffer from within the event loop
  let mut resize = false;

  'app: loop {
    // for all the events on the surface
    surface.window.glfw.poll_events();
    for (_, event) in surface.events_rx.try_iter() {
      match event {
        WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => break 'app,

        WindowEvent::FramebufferSize(..) => {
          resize = true;
        }

        _ => (),
      }
    }

    // if the window got resized
    if resize {
      // simply ask another backbuffer at the right dimension (no allocation / reallocation)
      back_buffer = surface.back_buffer().unwrap();

      // ditto for the offscreen framebuffer
      let (w, h) = surface.window.get_framebuffer_size();
      offscreen_buffer =
        Framebuffer::new(&mut surface, [w as u32, h as u32], 0, Sampler::default())
          .expect("framebuffer recreation");

      resize = false;
    }

    // we get an object to create pipelines (we’ll need two)
    let mut builder = surface.new_pipeline_gate();

    // render the triangle in the offscreen framebuffer first
    let render = builder.pipeline(
      &offscreen_buffer,
      &PipelineState::default(),
      |_, mut shd_gate| {
        shd_gate.shade(&mut program, |_, _, mut rdr_gate| {
          rdr_gate.render(&RenderState::default(), |mut tess_gate| {
            // we render the triangle here by asking for the whole triangle
            tess_gate.render(&triangle);
          });
        });
      },
    );

    if render.is_err() {
      break 'app;
    }

    // read from the offscreen framebuffer and output it into the back buffer
    let render = builder.pipeline(
      &back_buffer,
      &PipelineState::default(),
      |pipeline, mut shd_gate| {
        // we must bind the offscreen framebuffer color content so that we can pass it to a shader
        let (color, white) = offscreen_buffer.color_slot();

        let bound_color = pipeline.bind_texture(color).unwrap();

        let bound_white = pipeline.bind_texture(white).unwrap();

        shd_gate.shade(&mut copy_program, |mut iface, uni, mut rdr_gate| {
          // we update the texture with the bound texture
          iface.set(&uni.texture_color, bound_color.binding());
          iface.set(&uni.texture_white, bound_white.binding());

          rdr_gate.render(&RenderState::default(), |mut tess_gate| {
            // this will render the attributeless quad with both the offscreen framebuffer color
            // slots bound for the shader to fetch from
            tess_gate.render(&quad);
          });
        });
      },
    );

    // finally, swap the backbuffer with the frontbuffer in order to render our triangles onto your
    // screen
    if render.is_ok() {
      surface.window.swap_buffers();
    } else {
      break 'app;
    }
  }
}
