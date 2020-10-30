use cgmath::{Matrix4, Vector3};
use luminance::{shader::BuiltProgram, Semantics, Vertex};
use luminance_front::context::GraphicsContext as _;
use luminance_front::pipeline::PipelineState;
use luminance_front::render_state::RenderState;
use luminance_front::shader::Program;
use luminance_front::tess::{Interleaved, Mode, Tess};
use luminance_web_sys::WebSysWebGL2Surface;

const VS: &'static str = include_str!("vs.glsl");
const FS: &'static str = include_str!("fs.glsl");

#[derive(Clone, Copy, Debug, Eq, PartialEq, Semantics)]
pub enum Semantics {
  // - Reference vertex positions with the "co" variable in vertex shaders.
  // - The underlying representation is [f32; 2], which is a vec2 in GLSL.
  // - The wrapper type you can use to handle such a semantics is VertexPosition.
  #[sem(name = "co", repr = "[f32; 2]", wrapper = "VertexPosition")]
  Position,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "Semantics")]
struct Vertex {
  pos: VertexPosition,
}

// The vertices.
const TRI_VERTICES: [Vertex; 3] = [
  Vertex::new(VertexPosition::new([0.5, -0.5])),
  Vertex::new(VertexPosition::new([0.0, 0.5])),
  Vertex::new(VertexPosition::new([-0.5, -0.5])),
];

struct Scene {
  surface: WebSysWebGL2Surface,
  program: Program<Semantics, (), ()>,
  triangle: Tess<Vertex, (), (), Interleaved>,
}

fn get_scene(canvas_name: &str) -> Scene {
  let mut surface = WebSysWebGL2Surface::new(canvas_name).expect("web-sys surface");
  web_sys::console::log_1(&"got surface".into());

  let BuiltProgram { program, warnings } = surface
    .new_shader_program::<Semantics, (), ()>()
    .from_strings(VS, None, None, FS)
    .expect("program creation");
  web_sys::console::log_1(&"got shader program".into());

  for warning in warnings {
    web_sys::console::warn_1(&warning.to_string().into());
  }

  let triangle = surface
    .new_tess()
    .set_vertices(&TRI_VERTICES[..])
    .set_mode(Mode::Triangle)
    .build()
    .unwrap();
  web_sys::console::log_1(&"got triangle".into());

  Scene {
    surface,
    program,
    triangle,
  }
}

fn render_scene(scene: &mut Scene) {
  let Scene {
    surface,
    ref mut program,
    ref triangle,
  } = scene;
  let back_buffer = surface.back_buffer().unwrap();
  let translation_mat = Matrix4::from_translation(Vector3::new(0.5, 0., 0.));

  scene
    .surface
    .new_pipeline_gate()
    .pipeline(
      &back_buffer,
      &PipelineState::default(),
      |_, mut shd_gate| {
        shd_gate.shade(program, |mut iface, _, mut rdr_gate| {
          let uni = iface.query().unwrap().ask("translation_mat").unwrap();
          let mat: [[f32; 4]; 4] = translation_mat.into();
          iface.set(&uni, mat);

          rdr_gate.render(&RenderState::default(), |mut tess_gate| {
            tess_gate.render(triangle)
          })
        })
      },
    )
    .assume()
    .into_result()
    .unwrap();

  web_sys::console::log_1(&"render.".into());
}

pub fn fixture(canvas_name: &str) {
  let mut scene = get_scene(canvas_name);
  render_scene(&mut scene);
}
