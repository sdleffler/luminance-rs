#![allow(dead_code)]

/// This module demonstrates #304, which is about triggering a segfault when an OpenGL context quits while GPU scarce
/// resources allocated / created and bound to it are still alive. Those will live on for a while and trying to use
/// them or even destroy them (because of the Drop implementor) will generate UB / segfaults.
use luminance_front::{
  context::GraphicsContext as _,
  tess::{Mode, Tess},
};
use luminance_glfw::GlfwSurface;
use luminance_windowing::WindowOpt;

// Since drop order stabilization, we know that surface will be dropped first. Reversing the order of the field makes
// the segfault disappear.
struct Scene {
  surface: GlfwSurface,
  tess: Tess<(), ()>,
}

pub fn fixture() {
  let mut surface = GlfwSurface::new_gl33("GL_ARB_gpu_shader_fp64", WindowOpt::default()).unwrap();

  let tess = surface
    .new_tess()
    .set_mode(Mode::TriangleFan)
    .set_vertex_nb(4)
    .build()
    .unwrap();

  let _ = Scene { surface, tess };
}
