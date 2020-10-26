use glfw::{Action, Context as _, Key, WindowEvent};
use luminance::scissor::ScissorRegion;
use luminance_front::{
  context::GraphicsContext as _, pipeline::PipelineState, render_state::RenderState, tess::Mode,
};
use luminance_glfw::GlfwSurface;
use luminance_windowing::WindowOpt;

const VS: &str = "
const vec2[4] POSITIONS = vec2[](
  vec2(-1., -1.),
  vec2( 1., -1.),
  vec2( 1.,  1.),
  vec2(-1.,  1.)
);

void main() {
  gl_Position = vec4(POSITIONS[gl_VertexID], 0., 1.);
}";

const FS: &str = "
out vec3 frag;

void main() {
  frag = vec3(1., .5, .5);
}";

pub fn fixture() {
  let mut surface = GlfwSurface::new_gl33("Scissor test", WindowOpt::default()).unwrap();

  let mut program = surface
    .new_shader_program::<(), (), ()>()
    .from_strings(VS, None, None, FS)
    .unwrap()
    .ignore_warnings();

  let tess = surface
    .new_tess()
    .set_mode(Mode::TriangleFan)
    .set_vertex_nb(4)
    .build()
    .unwrap();

  'app: loop {
    surface.window.glfw.poll_events();
    for (_, event) in surface.events_rx.try_iter() {
      match event {
        WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => break 'app,
        _ => (),
      }
    }

    let back_buffer = surface.back_buffer().unwrap();
    let (width, height) = surface.window.get_framebuffer_size();
    let (w2, h2) = (width as u32 / 2, height as u32 / 2);
    let rdr_st = RenderState::default().set_scissor(ScissorRegion {
      x: w2 - w2 / 2,
      y: h2 - h2 / 2,
      width: w2,
      height: h2,
    });

    let render = surface
      .new_pipeline_gate()
      .pipeline(
        &back_buffer,
        &PipelineState::default(),
        |_, mut shd_gate| {
          shd_gate.shade(&mut program, |_, _, mut rdr_gate| {
            rdr_gate.render(&rdr_st, |mut tess_gate| tess_gate.render(&tess))
          })
        },
      )
      .assume();

    if render.is_ok() {
      surface.window.swap_buffers();
    } else {
      break 'app;
    }
  }
}
