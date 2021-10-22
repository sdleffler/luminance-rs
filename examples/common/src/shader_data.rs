//! This program shows how to use shader data to pass large chunks of data to shaders, that can be shared between
//! shaders and changed only once (instead for every shader that require that data).
//!
//! Shader data are often used to implement geometry instancing and various kinds of effects.
//!
//! <https://docs.rs/luminance>

use std::f32::consts::PI;

use crate::{
  shared::{Semantics, Vertex, VertexColor, VertexPosition},
  Example, InputAction, LoopFeedback, PlatformServices,
};
use luminance::UniformInterface;
use luminance_front::{
  context::GraphicsContext,
  framebuffer::Framebuffer,
  pipeline::{PipelineState, ShaderDataBinding},
  render_state::RenderState,
  shader::{types::Vec2, Program, ShaderData, Uniform},
  tess::{Mode, Tess, View as _},
  texture::Dim2,
  Backend,
};

const VS: &str = include_str!("./shader-data-2d-vs.glsl");
const FS: &str = include_str!("./simple-fs.glsl");

const VERTICES: [Vertex; 4] = [
  Vertex::new(
    VertexPosition::new([-0.01, -0.01]),
    VertexColor::new([0.5, 1., 0.5]),
  ),
  Vertex::new(
    VertexPosition::new([0.01, -0.01]),
    VertexColor::new([0.5, 1., 0.5]),
  ),
  Vertex::new(
    VertexPosition::new([0.01, 0.01]),
    VertexColor::new([0.5, 1., 0.5]),
  ),
  Vertex::new(
    VertexPosition::new([-0.01, 0.01]),
    VertexColor::new([0.5, 1., 0.5]),
  ),
];

#[derive(Debug, UniformInterface)]
struct ShaderInterface {
  #[uniform(name = "Positions")]
  positions: Uniform<ShaderDataBinding<Vec2<f32>>>,
}

pub struct LocalExample {
  square: Tess<Vertex>,
  program: Program<Semantics, (), ShaderInterface>,
  shader_data: ShaderData<Vec2<f32>>,
}

impl Example for LocalExample {
  fn bootstrap(
    _: &mut impl PlatformServices,
    ctx: &mut impl GraphicsContext<Backend = Backend>,
  ) -> Self {
    let square = ctx
      .new_tess()
      .set_vertices(&VERTICES[..])
      .set_mode(Mode::TriangleFan)
      .build()
      .expect("square tessellation");

    let program = ctx
      .new_shader_program()
      .from_strings(VS, None, None, FS)
      .expect("shader program")
      .ignore_warnings();

    let shader_data = ctx
      .new_shader_data([Vec2::new(0., 0.); 100])
      .expect("shader data");

    Self {
      square,
      program,
      shader_data,
    }
  }

  fn render_frame(
    mut self,
    time: f32,
    back_buffer: Framebuffer<Dim2, (), ()>,
    actions: impl Iterator<Item = InputAction>,
    ctx: &mut impl GraphicsContext<Backend = Backend>,
  ) -> LoopFeedback<Self> {
    for action in actions {
      match action {
        InputAction::Quit => return LoopFeedback::Exit,
        _ => (),
      }
    }

    let square = &self.square;
    let program = &mut self.program;
    let shader_data = &mut self.shader_data;

    // update the positions of the squares
    let new_positions = (0..100).map(|i| {
      let i = i as f32;
      let phi = i * 2. * PI * 0.01 + time * 0.2;
      let radius = 0.8;

      Vec2::new(phi.cos() * radius, phi.sin() * radius)
    });
    shader_data
      .replace(new_positions)
      .expect("replace shader data");

    let render = ctx
      .new_pipeline_gate()
      .pipeline(
        &back_buffer,
        &PipelineState::default(),
        |pipeline, mut shd_gate| {
          // bind the shader data so that we can update it
          let bound_shader_data = pipeline
            .bind_shader_data(shader_data)
            .expect("bound shader data");

          shd_gate.shade(program, |mut iface, uni, mut rdr_gate| {
            iface.set(&uni.positions, bound_shader_data.binding());

            rdr_gate.render(&RenderState::default(), |mut tess_gate| {
              tess_gate.render(square.inst_view(.., 100).expect("instanced tess"))
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
