//! This program is a showcase to demonstrate how you can use a texture from an image loaded from the disk.
//! For the purpose of simplicity, the image is stretched to match your window resolution.
//!
//! > Note: for this example, it is recommended to compile with --release to speed up image loading.
//!
//! <https://docs.rs/luminance>

use crate::{
  shared::{load_texture, RGBTexture},
  Example, Features, InputAction, LoopFeedback, PlatformServices,
};
use luminance::UniformInterface;
use luminance_front::{
  blending::{Blending, Equation, Factor},
  context::GraphicsContext,
  framebuffer::Framebuffer,
  pipeline::{PipelineState, TextureBinding},
  pixel::NormUnsigned,
  render_state::RenderState,
  shader::{Program, Uniform},
  tess::{Mode, Tess},
  texture::Dim2,
  Backend,
};

const VS: &'static str = include_str!("texture-vs.glsl");
const FS: &'static str = include_str!("texture-fs.glsl");

// we also need a special uniform interface here to pass the texture to the shader
#[derive(UniformInterface)]
struct ShaderInterface {
  tex: Uniform<TextureBinding<Dim2, NormUnsigned>>,
}

pub struct LocalExample {
  image: RGBTexture,
  program: Program<(), (), ShaderInterface>,
  tess: Tess<()>,
}

impl Example for LocalExample {
  fn features() -> Features {
    Features::none().texture("source.jpg")
  }

  fn bootstrap(
    platform: &mut impl PlatformServices,
    context: &mut impl GraphicsContext<Backend = Backend>,
  ) -> Self {
    let image = load_texture(context, platform, "source.jpg").expect("texture to display");

    // set the uniform interface to our type so that we can read textures from the shader
    let program = context
      .new_shader_program::<(), (), ShaderInterface>()
      .from_strings(VS, None, None, FS)
      .expect("program creation")
      .ignore_warnings();

    // we’ll use an attributeless render here to display a quad on the screen (two triangles); there
    // are over ways to cover the whole screen but this is easier for you to understand; the
    // TriangleFan creates triangles by connecting the third (and next) vertex to the first one
    let tess = context
      .new_tess()
      .set_render_vertex_nb(4)
      .set_mode(Mode::TriangleFan)
      .build()
      .unwrap();

    LocalExample {
      image,
      program,
      tess,
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
        _ => (),
      }
    }

    let tex = &mut self.image;
    let program = &mut self.program;
    let tess = &self.tess;
    let render_st = &RenderState::default().set_blending(Blending {
      equation: Equation::Additive,
      src: Factor::SrcAlpha,
      dst: Factor::Zero,
    });

    let render = context
      .new_pipeline_gate()
      .pipeline(
        &back_buffer,
        &PipelineState::default(),
        |pipeline, mut shd_gate| {
          // bind our fancy texture to the GPU: it gives us a bound texture we can use with the shader
          let bound_tex = pipeline.bind_texture(tex)?;

          shd_gate.shade(program, |mut iface, uni, mut rdr_gate| {
            // update the texture; strictly speaking, this update doesn’t do much: it just tells the GPU
            // to use the texture passed as argument (no allocation or copy is performed)
            iface.set(&uni.tex, bound_tex.binding());

            rdr_gate.render(render_st, |mut tess_gate| {
              // render the tessellation to the surface the regular way and let the vertex shader’s
              // magic do the rest!
              tess_gate.render(tess)
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
