use luminance_front::context::GraphicsContext;
use luminance_front::tess::TessError;
use luminance_glfw::GlfwSurface;
use luminance_windowing::WindowOpt;

pub fn fixture() -> bool {
  let mut surface =
    GlfwSurface::new_gl33("GL_ARB_gpu_shader_fp64 test", WindowOpt::default()).unwrap();
  let tess = surface.new_tess().build();

  if let Err(TessError::NoData) = tess {
    true
  } else {
    false
  }
}
