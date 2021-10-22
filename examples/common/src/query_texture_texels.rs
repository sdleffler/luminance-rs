//! This program shows how to render two simple triangles, query the texels from the rendered
//! framebuffer and output them in a texture.
//!
//! This example is requires a file system to run as it will write the output to it.
//!
//! <https://docs.rs/luminance>

use crate::{
  shared::{Semantics, Vertex, VertexColor, VertexPosition},
  Example, LoopFeedback, PlatformServices,
};
use image::{save_buffer, ColorType};
use luminance::context::GraphicsContext;
use luminance::Vertex;
use luminance_front::{
  framebuffer::Framebuffer,
  pipeline::PipelineState,
  pixel::NormRGBA8UI,
  render_state::RenderState,
  shader::Program,
  tess::{Mode, Tess},
  texture::{Dim2, Sampler},
  Backend,
};

// We get the shader at compile time from local files
const VS: &'static str = include_str!("simple-vs.glsl");
const FS: &'static str = include_str!("simple-fs.glsl");

// The vertices. We define two triangles.
const TRI_VERTICES: [Vertex; 6] = [
  // first triangle – an RGB one
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
  // second triangle, a purple one, positioned differently
  Vertex {
    pos: VertexPosition::new([-0.5, 0.5]),
    rgb: VertexColor::new([1., 0.2, 1.]),
  },
  Vertex {
    pos: VertexPosition::new([0.0, -0.5]),
    rgb: VertexColor::new([0.2, 1., 1.]),
  },
  Vertex {
    pos: VertexPosition::new([0.5, 0.5]),
    rgb: VertexColor::new([0.2, 0.2, 1.]),
  },
];

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "Semantics")]
struct Positions {
  pos: VertexPosition,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "Semantics")]
struct Colors {
  color: VertexColor,
}

pub struct LocalExample {
  program: Program<Semantics, (), ()>,
  triangles: Tess<Vertex>,
  framebuffer: luminance_front::framebuffer::Framebuffer<Dim2, NormRGBA8UI, ()>,
}

impl Example for LocalExample {
  fn bootstrap(
    _: &mut impl PlatformServices,
    context: &mut impl GraphicsContext<Backend = Backend>,
  ) -> Self {
    // we need a program to “shade” our triangles and to tell luminance which is the input vertex
    // type, and we’re not interested in the other two type variables for this sample
    let program = context
      .new_shader_program::<Semantics, (), ()>()
      .from_strings(VS, None, None, FS)
      .expect("program creation")
      .ignore_warnings();

    // create tessellation for direct geometry; that is, tessellation that will render vertices by
    // taking one after another in the provided slice
    let triangles = context
      .new_tess()
      .set_vertices(&TRI_VERTICES[..])
      .set_mode(Mode::Triangle)
      .build()
      .unwrap();

    // the back buffer, which we will make our render into (we make it mutable so that we can change
    // it whenever the window dimensions change)
    let framebuffer = context
      .new_framebuffer::<Dim2, NormRGBA8UI, ()>([960, 540], 0, Sampler::default())
      .unwrap();

    Self {
      program,
      triangles,
      framebuffer,
    }
  }

  fn render_frame(
    mut self,
    _: f32,
    _: Framebuffer<Dim2, (), ()>,
    _: impl Iterator<Item = crate::InputAction>,
    context: &mut impl GraphicsContext<Backend = Backend>,
  ) -> LoopFeedback<Self> {
    let program = &mut self.program;
    let triangles = &self.triangles;
    let framebuffer = &mut self.framebuffer;

    // create a new dynamic pipeline that will render to the back buffer and must clear it with
    // pitch black prior to do any render to it
    context
      .new_pipeline_gate()
      .pipeline(framebuffer, &PipelineState::default(), |_, mut shd_gate| {
        // start shading with our program
        shd_gate.shade(program, |_, _, mut rdr_gate| {
          // start rendering things with the default render state provided by luminance
          rdr_gate.render(&RenderState::default(), |mut tess_gate| {
            // pick the right tessellation to use depending on the mode chosen
            // render the tessellation to the surface
            tess_gate.render(triangles)
          })
        })
      })
      .assume()
      .into_result()
      .expect("offscreen render");

    // the backbuffer contains our texels
    let texels = framebuffer.color_slot().get_raw_texels().unwrap();
    // create a .png file and output it
    save_buffer("./rendered.png", &texels, 960, 540, ColorType::Rgba8).unwrap();

    LoopFeedback::Exit
  }
}
