//! This program is a showcase to demonstrate how you can use texture from an image loaded from the
//! disk and re-use it to load another image with a different size.
//!
//! Two texture paths are read from the command line interface and are the sole arguments.
//!
//! For the purpose of simplicity, the images ares stretched to match your window resolution.
//!
//! Press <space> to switch to the other image.
//! Press <escape> to quit or close the window.
//!
//! > Note: for this example, it is recommended to compile with --release to speed up image loading.
//!
//! https://docs.rs/luminance

use glfw::{Action, Context as _, Key, WindowEvent};
use luminance::backend::texture::Texture as TextureBackend;
use luminance::blending::{Blending, Equation, Factor};
use luminance::context::GraphicsContext;
use luminance::pipeline::{PipelineState, TextureBinding};
use luminance::pixel::{NormRGB8UI, NormUnsigned};
use luminance::render_state::RenderState;
use luminance::shader::Uniform;
use luminance::tess::Mode;
use luminance::texture::{Dim2, GenMipmaps, Sampler, Texture};
use luminance::UniformInterface;
use luminance_glfw::GlfwSurface;
use luminance_windowing::{WindowDim, WindowOpt};
use std::env; // used to get the CLI arguments
use std::path::Path;

const VS: &'static str = include_str!("texture-vs.glsl");
const FS: &'static str = include_str!("texture-fs.glsl");

fn main() {
  let texture_paths: Vec<_> = env::args().skip(1).collect();
  if texture_paths.len() == 2 {
    run(Path::new(&texture_paths[0]), Path::new(&texture_paths[1]));
  } else {
    eprintln!("missing texture paths");
  }
}

// we also need a special uniform interface here to pass the texture to the shader
#[derive(UniformInterface)]
struct ShaderInterface {
  tex: Uniform<TextureBinding<Dim2, NormUnsigned>>,
}

fn run(texture_path: &Path, texture_path2: &Path) {
  let img = read_image(texture_path).expect("error while reading first image on disk");
  let img2 = read_image(texture_path2).expect("error while reading second image on disk");
  let mut current_image = 0;
  let (width, height) = img.dimensions();
  let images = [img, img2];

  let dim = WindowDim::Windowed { width, height };
  let surface = GlfwSurface::new_gl33("Hello, world!", WindowOpt::default().set_dim(dim))
    .expect("GLFW surface creation");
  let mut context = surface.context;
  let events = surface.events_rx;

  let mut tex = load_from_disk(&mut context, &images[0]);

  // set the uniform interface to our type so that we can read textures from the shader
  let mut program = context
    .new_shader_program::<(), (), ShaderInterface>()
    .from_strings(VS, None, None, FS)
    .expect("program creation")
    .ignore_warnings();

  // we’ll use an attributeless render here to display a quad on the screen (two triangles); there
  // are over ways to cover the whole screen but this is easier for you to understand; the
  // TriangleFan creates triangles by connecting the third (and next) vertex to the first one
  let tess = context
    .new_tess()
    .set_vertex_nb(4)
    .set_mode(Mode::TriangleFan)
    .build()
    .unwrap();

  let mut back_buffer = context.back_buffer().unwrap();
  let render_st = &RenderState::default().set_blending(Blending {
    equation: Equation::Additive,
    src: Factor::SrcAlpha,
    dst: Factor::Zero,
  });

  println!("rendering!");

  'app: loop {
    context.window.glfw.poll_events();
    for (_, event) in glfw::flush_messages(&events) {
      match event {
        WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => break 'app,

        WindowEvent::FramebufferSize(..) => {
          back_buffer = context.back_buffer().unwrap();
        }

        WindowEvent::Key(Key::Space, _, Action::Release, _) => {
          // reload the image
          current_image = 1 - current_image;
          reload_image(&images[current_image], &mut tex);
        }

        _ => (),
      }
    }

    // here, we need to bind the pipeline variable; it will enable us to bind the texture to the GPU
    // and use it in the shader
    let render = context
      .new_pipeline_gate()
      .pipeline(
        &back_buffer,
        &PipelineState::default(),
        |pipeline, mut shd_gate| {
          // bind our fancy texture to the GPU: it gives us a bound texture we can use with the shader
          let bound_tex = pipeline.bind_texture(&mut tex)?;

          shd_gate.shade(&mut program, |mut iface, uni, mut rdr_gate| {
            // update the texture; strictly speaking, this update doesn’t do much: it just tells the GPU
            // to use the texture passed as argument (no allocation or copy is performed)
            iface.set(&uni.tex, bound_tex.binding());

            rdr_gate.render(render_st, |mut tess_gate| {
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

// read the texture into memory as a whole bloc (i.e. no streaming)
fn read_image(path: &Path) -> Option<image::RgbImage> {
  image::open(path).map(|img| img.flipv().to_rgb8()).ok()
}

fn load_from_disk<B>(
  context: &mut B,
  img: &image::RgbImage,
) -> Texture<B::Backend, Dim2, NormRGB8UI>
where
  B: GraphicsContext,
  B::Backend: TextureBackend<Dim2, NormRGB8UI>,
{
  let (width, height) = img.dimensions();
  let texels = img.as_raw();

  // create the luminance texture; the third argument is the number of mipmaps we want (leave it
  // to 0 for now) and the latest is the sampler to use when sampling the texels in the
  // shader (we’ll just use the default one)
  //
  // the GenMipmaps argument disables mipmap generation (we don’t care so far)
  let tex = context
    .new_texture_raw(
      [width, height],
      0,
      Sampler::default(),
      GenMipmaps::No,
      &texels,
    )
    .expect("luminance texture creation");

  tex
}

fn reload_image(
  img: &image::RgbImage,
  tex: &mut Texture<impl TextureBackend<Dim2, NormRGB8UI>, Dim2, NormRGB8UI>,
) {
  let (width, height) = img.dimensions();
  let texels = img.as_raw();

  // redimension the texture
  tex
    .resize_raw([width, height], 0, GenMipmaps::No, &texels)
    .expect("texture resizing");
}
