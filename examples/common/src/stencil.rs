//! This example shows how to use the stencil buffer to implement a glowing effect when moving the cursor on the
//! triangle.

use crate::{
  shared::{Semantics, Vertex, VertexColor, VertexPosition},
  Example, InputAction, LoopFeedback, PlatformServices,
};
use luminance::UniformInterface;
use luminance_front::{
  context::GraphicsContext,
  depth_stencil::{Comparison, StencilOp, StencilOperations, StencilTest},
  framebuffer::Framebuffer,
  pipeline::{PipelineState, TextureBinding},
  pixel::{Depth32FStencil8, NormRGBA32UI, NormUnsigned},
  render_state::RenderState,
  shader::{types::Vec3, Program, Uniform},
  tess::{Mode, Tess},
  texture::{Dim2, Sampler},
  Backend,
};

const VS: &str = r#"
in vec2 co;

uniform float scale;

void main() {
  gl_Position = vec4(co * scale, 0., 1.);
}
"#;

const FS: &str = r#"
out vec4 frag;

uniform vec3 color;

void main() {
  frag = vec4(color, 1.);
}
"#;

const COPY_VS: &str = include_str!("copy-vs.glsl");
const COPY_FS: &str = include_str!("copy-fs.glsl");

const VERTICES: [Vertex; 3] = [
  Vertex::new(
    VertexPosition::new([-0.5, -0.5]),
    VertexColor::new([1., 1., 1.]),
  ),
  Vertex::new(
    VertexPosition::new([0.5, -0.5]),
    VertexColor::new([1., 1., 1.]),
  ),
  Vertex::new(
    VertexPosition::new([0., 0.5]),
    VertexColor::new([1., 1., 1.]),
  ),
];

#[derive(Debug, UniformInterface)]
struct StencilInterface {
  scale: Uniform<f32>,
  color: Uniform<Vec3<f32>>,
}

#[derive(Debug, UniformInterface)]
struct ShaderCopyInterface {
  source_texture: Uniform<TextureBinding<Dim2, NormUnsigned>>,
}

pub struct LocalExample {
  program: Program<Semantics, (), StencilInterface>,
  copy_program: Program<(), (), ShaderCopyInterface>,
  framebuffer: Framebuffer<Dim2, NormRGBA32UI, Depth32FStencil8>,
  triangle: Tess<Vertex>,
  attributeless: Tess<()>,
}

impl Example for LocalExample {
  fn bootstrap(
    _platform: &mut impl PlatformServices,
    context: &mut impl GraphicsContext<Backend = Backend>,
  ) -> Self {
    let program = context
      .new_shader_program()
      .from_strings(VS, None, None, FS)
      .expect("program")
      .ignore_warnings();

    let copy_program = context
      .new_shader_program()
      .from_strings(COPY_VS, None, None, COPY_FS)
      .expect("program")
      .ignore_warnings();

    let framebuffer = context
      .new_framebuffer([1, 1], 1, Sampler::default())
      .expect("framebuffer");

    let triangle = context
      .new_tess()
      .set_mode(Mode::Triangle)
      .set_vertices(VERTICES)
      .build()
      .expect("triangle");

    let attributeless = context
      .new_tess()
      .set_mode(Mode::TriangleFan)
      .set_render_vertex_nb(4)
      .build()
      .expect("attributeless");

    LocalExample {
      program,
      framebuffer,
      copy_program,
      triangle,
      attributeless,
    }
  }

  fn render_frame(
    mut self,
    time: f32,
    back_buffer: Framebuffer<Dim2, (), ()>,
    actions: impl Iterator<Item = InputAction>,
    context: &mut impl GraphicsContext<Backend = Backend>,
  ) -> LoopFeedback<Self> {
    for action in actions {
      match action {
        InputAction::Quit => return LoopFeedback::Exit,

        InputAction::Resized { width, height } => {
          self.framebuffer = context
            .new_framebuffer([width, height], 1, Sampler::default())
            .expect("framebuffer");
        }

        _ => (),
      }
    }

    let framebuffer = &mut self.framebuffer;
    let program = &mut self.program;
    let copy_program = &mut self.copy_program;
    let triangle = &self.triangle;
    let attributeless = &self.attributeless;

    let mut pipeline_gate = context.new_pipeline_gate();
    let render = pipeline_gate
      .pipeline(framebuffer, &PipelineState::default(), |_, mut shd_gate| {
        shd_gate.shade(program, |mut iface, uni, mut rdr_gate| {
          // first we do a regular render in the framebuffer; we will write stencil bits to 1
          iface.set(&uni.scale, 1.);
          iface.set(&uni.color, Vec3::new(1., 1., 1.));

          rdr_gate.render(
            &RenderState::default()
              // we pass the stencil test if the value is < 1
              .set_stencil_test(StencilTest::new(Comparison::Less, 1, 0xFF))
              .set_stencil_operations(
                StencilOperations::default().on_depth_stencil_pass(StencilOp::Replace),
              ),
            |mut tess_gate| tess_gate.render(triangle),
          )?;

          // then, render again but slightly upscaled
          iface.set(&uni.scale, 1. + (time * 3.).cos().abs() * 0.1);
          iface.set(&uni.color, Vec3::new(0., 1., 0.));

          rdr_gate.render(
            &RenderState::default()
              // we pass the stencil test if the value is == 0
              .set_stencil_test(StencilTest::new(Comparison::Equal, 0, 0xFF))
              .set_stencil_operations(
                StencilOperations::default().on_depth_stencil_pass(StencilOp::Replace),
              ),
            |mut tess_gate| tess_gate.render(triangle),
          )
        })
      })
      .assume();

    if render.is_err() {
      return LoopFeedback::Exit;
    }

    let render = pipeline_gate
      .pipeline(
        &back_buffer,
        &PipelineState::default(),
        |pipeline, mut shd_gate| {
          let source = pipeline
            .bind_texture(framebuffer.color_slot())
            .expect("offscreen bound texture");

          shd_gate.shade(copy_program, |mut iface, uni, mut rdr_gate| {
            iface.set(&uni.source_texture, source.binding());

            rdr_gate.render(&RenderState::default(), |mut tess_gate| {
              tess_gate.render(attributeless)
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
