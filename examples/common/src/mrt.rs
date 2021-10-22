//! This program shows how to render a single triangle into an offscreen framebuffer with two
//! target textures, and how to render the contents of these textures into the back
//! buffer (i.e. the screen), combining data from both.
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
  pipeline::{PipelineState, TextureBinding},
  pixel::{NormR8UI, NormRGB8UI, NormUnsigned},
  render_state::RenderState,
  shader::{Program, Uniform},
  tess::{Mode, Tess},
  texture::{Dim2, Sampler},
  Backend,
};

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
  texture_color: Uniform<TextureBinding<Dim2, NormUnsigned>>,
  #[uniform(name = "source_texture_white")]
  texture_white: Uniform<TextureBinding<Dim2, NormUnsigned>>,
}

pub struct LocalExample {
  program: Program<Semantics, (), ()>,
  copy_program: Program<(), (), ShaderInterface>,
  triangle: Tess<Vertex>,
  quad: Tess<()>,
  offscreen_buffer: Framebuffer<Dim2, (NormRGB8UI, NormR8UI), ()>,
}

impl Example for LocalExample {
  fn bootstrap(
    _: &mut impl PlatformServices,
    context: &mut impl GraphicsContext<Backend = Backend>,
  ) -> Self {
    let program = context
      .new_shader_program::<Semantics, (), ()>()
      .from_strings(VS, None, None, FS)
      .unwrap()
      .ignore_warnings();

    let copy_program = context
      .new_shader_program::<(), (), ShaderInterface>()
      .from_strings(COPY_VS, None, None, COPY_FS)
      .unwrap()
      .ignore_warnings();

    let triangle = context
      .new_tess()
      .set_vertices(&TRI_VERTICES[..])
      .set_mode(Mode::Triangle)
      .build()
      .unwrap();

    // we’ll need an attributeless quad to fetch in full screen
    let quad = context
      .new_tess()
      .set_render_vertex_nb(4)
      .set_mode(Mode::TriangleFan)
      .build()
      .unwrap();

    // the offscreen buffer; defined with a dummy 10×10 dimension
    let offscreen_buffer = context
      .new_framebuffer::<Dim2, (NormRGB8UI, NormR8UI), ()>([800, 800], 0, Sampler::default())
      .expect("framebuffer creation");

    Self {
      program,
      copy_program,
      triangle,
      quad,
      offscreen_buffer,
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

        InputAction::Resized { width, height } => {
          // simply ask another offscreen framebuffer at the right dimension (no allocation / reallocation)
          self.offscreen_buffer = context
            .new_framebuffer([width, height], 0, Sampler::default())
            .expect("framebuffer recreation");
        }

        _ => (),
      }
    }

    let program = &mut self.program;
    let copy_program = &mut self.copy_program;
    let triangle = &self.triangle;
    let quad = &self.quad;
    let offscreen_buffer = &mut self.offscreen_buffer;

    // we get an object to create pipelines (we’ll need two)
    let mut builder = context.new_pipeline_gate();

    // render the triangle in the offscreen framebuffer first
    let render = builder
      .pipeline(
        offscreen_buffer,
        &PipelineState::default(),
        |_, mut shd_gate| {
          shd_gate.shade(program, |_, _, mut rdr_gate| {
            rdr_gate.render(&RenderState::default(), |mut tess_gate| {
              // we render the triangle here by asking for the whole triangle
              tess_gate.render(triangle)
            })
          })
        },
      )
      .assume();

    if render.is_err() {
      return LoopFeedback::Exit;
    }

    // read from the offscreen framebuffer and output it into the back buffer
    let render = builder
      .pipeline(
        &back_buffer,
        &PipelineState::default(),
        |pipeline, mut shd_gate| {
          // we must bind the offscreen framebuffer color content so that we can pass it to a shader
          let (color, white) = offscreen_buffer.color_slot();

          let bound_color = pipeline.bind_texture(color)?;

          let bound_white = pipeline.bind_texture(white)?;

          shd_gate.shade(copy_program, |mut iface, uni, mut rdr_gate| {
            // we update the texture with the bound texture
            iface.set(&uni.texture_color, bound_color.binding());
            iface.set(&uni.texture_white, bound_white.binding());

            rdr_gate.render(&RenderState::default(), |mut tess_gate| {
              // this will render the attributeless quad with both the offscreen framebuffer color
              // slots bound for the shader to fetch from
              tess_gate.render(quad)
            })
          })
        },
      )
      .assume();

    // finally, swap the backbuffer with the frontbuffer in order to render our triangles onto your
    // screen
    if render.is_ok() {
      LoopFeedback::Continue(self)
    } else {
      LoopFeedback::Exit
    }
  }
}
