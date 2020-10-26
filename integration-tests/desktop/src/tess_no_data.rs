use luminance_front::context::GraphicsContext as _;
use luminance_front::tess::TessError;
use luminance_glfw::GlfwSurface;
use luminance_windowing::WindowOpt;

pub fn fixture() {
  let mut surface = GlfwSurface::new_gl33("Tess no data", WindowOpt::default()).unwrap();
  let tess = surface.new_tess().build();

  if let Err(TessError::NoData) = tess {
  } else {
    panic!();
  }
}
