//! [SDL2](https://crates.io/crates/sdl2) backend for [luminance](https://crates.io/crates/luminance)
//! and [luminance-windowing](https://crates.io/crates/luminance-windowing).

#![deny(missing_docs)]

use gl;
use luminance::context::GraphicsContext;
use luminance::framebuffer::Framebuffer;
use luminance::framebuffer::FramebufferError;
use luminance::texture::Dim2;
pub use luminance_gl::gl33::StateQueryError;
use luminance_gl::GL33;
pub use luminance_windowing::{CursorMode, WindowDim, WindowOpt};
pub use sdl2;
use std::fmt;
use std::os::raw::c_void;

/// Error that can be risen while creating a surface.
#[derive(Debug)]
pub enum Sdl2SurfaceError {
  /// Initialization of the surface went wrong.
  InitError(String),
  /// Window creation failed.
  WindowCreationFailed(sdl2::video::WindowBuildError),
  /// Failed to create an OpenGL context.
  GlContextInitFailed(String),
  /// No available video mode.
  VideoInitError(String),
  /// The graphics state is not available.
  ///
  /// This error is generated when the initialization code is called on a thread on which the
  /// graphics state has already been acquired.
  GraphicsStateError(StateQueryError),
}

impl fmt::Display for Sdl2SurfaceError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      Sdl2SurfaceError::InitError(ref e) => write!(f, "initialization error: {}", e),
      Sdl2SurfaceError::WindowCreationFailed(ref e) => write!(f, "failed to create window: {}", e),
      Sdl2SurfaceError::GlContextInitFailed(ref e) => {
        write!(f, "failed to create OpenGL context: {}", e)
      }
      Sdl2SurfaceError::VideoInitError(ref e) => {
        write!(f, "failed to initialize video system: {}", e)
      }
      Sdl2SurfaceError::GraphicsStateError(ref e) => {
        write!(f, "failed to get graphics state: {}", e)
      }
    }
  }
}

/// A [luminance] GraphicsContext backed by SDL2 and OpenGL 3.3 Core.
///
/// ```
/// use luminance_sdl2::{GL33Surface, WindowOpt, WindowDim, CursorMode};
///
/// let opts = WindowOpt::default()
///     .set_dim(WindowDim::Fullscreen)
///     .set_cursor_mode(CursorMode::Disabled);
///
/// let surface = GL33Surface::new("Example window", opts)
///     .expect("failed to create surface");
///
/// let event_pump = surface.sdl().event_pump()
///     .expect("failed to initialize event subsystem");
/// ```
///
/// [luminance]: https://crates.io/crates/luminance
pub struct GL33Surface {
  sdl: sdl2::Sdl,
  window: sdl2::video::Window,
  gl: GL33,
  // This struct needs to stay alive until we are done with OpenGL stuff.
  _gl_context: sdl2::video::GLContext,
}

impl GL33Surface {
  /// Create a new surface.
  pub fn new(title: &str, options: WindowOpt) -> Result<Self, Sdl2SurfaceError> {
    let sdl = sdl2::init().map_err(Sdl2SurfaceError::InitError)?;

    let video_system = sdl.video().map_err(Sdl2SurfaceError::VideoInitError)?;

    let gl_attr = video_system.gl_attr();

    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_flags().forward_compatible().set();
    gl_attr.set_context_major_version(3);
    gl_attr.set_context_minor_version(3);

    match options.num_samples {
      Some(num) => {
        gl_attr.set_multisample_buffers(1);
        gl_attr.set_multisample_samples(num as u8);
      }
      None => {
        gl_attr.set_multisample_buffers(0);
        gl_attr.set_multisample_samples(0);
      }
    }

    let mouse = sdl.mouse();
    match options.cursor_mode {
      CursorMode::Visible => {
        mouse.show_cursor(true);
        mouse.set_relative_mouse_mode(false);
      }
      CursorMode::Invisible => {
        mouse.show_cursor(false);
        mouse.set_relative_mouse_mode(false);
      }
      CursorMode::Disabled => {
        mouse.show_cursor(false);
        mouse.set_relative_mouse_mode(true);
      }
    }

    let window = {
      let mut builder;
      match options.dim {
        WindowDim::Windowed { width, height } => {
          builder = video_system.window(title, width, height)
        }
        WindowDim::Fullscreen => {
          // I don't think it matters what dimensions we pass here.
          builder = video_system.window(title, 800, 600);
          builder.fullscreen();
        }
        WindowDim::FullscreenRestricted { width, height } => {
          builder = video_system.window(title, width, height);
          builder.fullscreen_desktop();
        }
      }
      builder
        .opengl()
        .build()
        .map_err(Sdl2SurfaceError::WindowCreationFailed)?
    };

    let _gl_context = window
      .gl_create_context()
      .map_err(Sdl2SurfaceError::GlContextInitFailed)?;

    gl::load_with(|s| video_system.gl_get_proc_address(s) as *const c_void);

    let gl = GL33::new().map_err(Sdl2SurfaceError::GraphicsStateError)?;
    let surface = GL33Surface {
      sdl,
      window,
      gl,
      _gl_context,
    };

    Ok(surface)
  }

  /// The entry point to most of the SDL2 API.
  pub fn sdl(&self) -> &sdl2::Sdl {
    &self.sdl
  }

  /// The underlying SDL2 window of this surface.
  pub fn window(&self) -> &sdl2::video::Window {
    &self.window
  }

  /// Get the back buffer.
  pub fn back_buffer(&mut self) -> Result<Framebuffer<GL33, Dim2, (), ()>, FramebufferError> {
    let (w, h) = self.window.drawable_size();
    Framebuffer::back_buffer(self, [w, h])
  }
}

unsafe impl GraphicsContext for GL33Surface {
  type Backend = GL33;

  fn backend(&mut self) -> &mut Self::Backend {
    &mut self.gl
  }
}
