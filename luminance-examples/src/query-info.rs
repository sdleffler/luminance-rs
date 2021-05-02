//! This program demonstrates the Query API, allowing to get information about the GPU and the backend.
//!
//! https://docs.rs/luminance

use luminance::context::GraphicsContext as _;
use luminance_glfw::GlfwSurface;
use luminance_windowing::{WindowDim, WindowOpt};

fn main() {
  let dim = WindowDim::Windowed {
    width: 960,
    height: 540,
  };
  let surface = GlfwSurface::new_gl33(
    "Hello, world; from OpenGL 3.3!",
    WindowOpt::default().set_dim(dim),
  )
  .expect("GLFW surface creation");
  let mut context = surface.context;
  let q = context.query();

  println!("Backend author: {:?}", q.backend_author());
  println!("Backend name: {:?}", q.backend_name());
  println!("Backend version: {:?}", q.backend_version());
  println!(
    "Backend shading language version: {:?}",
    q.backend_shading_lang_version()
  );
  println!(
    "Maximum number of elements in a texture array: {:?}",
    q.max_texture_array_elements()
  )
}
