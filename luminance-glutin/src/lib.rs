//! The [glutin] windowing implementation for [luminance-windowing].
//!
//! [glutin]: https://crates.io/crates/glutin
//! [luminance-windowing]: https://crates.io/crates/luminance-windowing

#![deny(missing_docs)]

use gl;
pub use glutin::dpi::PhysicalSize;
use glutin::{
  Api, ContextBuilder, EventsLoop, GlProfile, GlRequest, PossiblyCurrent, WindowBuilder,
  WindowedContext,
};
pub use glutin::{ContextError, CreationError};
use luminance::context::GraphicsContext;
use luminance::framebuffer::Framebuffer;
use luminance::state::{GraphicsState, StateQueryError};
use luminance::texture::{Dim2, Flat};
pub use luminance_windowing::{CursorMode, Surface, WindowDim, WindowOpt};
use std::cell::RefCell;
use std::fmt;
use std::os::raw::c_void;
use std::rc::Rc;

/// Error that might occur when creating a Glutin surface.
#[derive(Debug)]
pub enum GlutinError {
  /// Something went wrong when creating the Glutin surface. The carried [`CreationError`] provides
  /// more information.
  CreationError(CreationError),
  /// OpenGL context error.
  ContextError(ContextError),
  /// Graphics state error that might occur when querying the initial state.
  GraphicsStateError(StateQueryError),
}

impl fmt::Display for GlutinError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      GlutinError::CreationError(ref e) => write!(f, "Glutin surface creation error: {}", e),
      GlutinError::ContextError(ref e) => write!(f, "Glutin OpenGL context creation error: {}", e),
      GlutinError::GraphicsStateError(ref e) => {
        write!(f, "OpenGL graphics state initialization error: {}", e)
      }
    }
  }
}

impl From<CreationError> for GlutinError {
  fn from(e: CreationError) -> Self {
    GlutinError::CreationError(e)
  }
}

impl From<ContextError> for GlutinError {
  fn from(e: ContextError) -> Self {
    GlutinError::ContextError(e)
  }
}

/// The Glutin surface.
///
/// You want to create such an object in order to use any [luminance] construct.
///
/// [luminance]: https://crates.io/crates/luminance
pub struct GlutinSurface {
  /// The windowed context.
  pub ctx: WindowedContext<PossiblyCurrent>,
  /// Associated events loop.
  pub event_loop: EventsLoop,
  gfx_state: Rc<RefCell<GraphicsState>>,
}

unsafe impl GraphicsContext for GlutinSurface {
  fn state(&self) -> &Rc<RefCell<GraphicsState>> {
    &self.gfx_state
  }
}

impl GlutinSurface {
  /// Create a new [`GlutinSurface`] from scratch.
  pub fn new(dim: WindowDim, title: &str, win_opt: WindowOpt) -> Result<Self, GlutinError> {
    let event_loop = EventsLoop::new();

    let window_builder = WindowBuilder::new().with_title(title);
    let window_builder = match dim {
      WindowDim::Windowed(w, h) => window_builder.with_dimensions((w, h).into()),
      WindowDim::Fullscreen => {
        window_builder.with_fullscreen(Some(event_loop.get_primary_monitor()))
      }
      WindowDim::FullscreenRestricted(w, h) => window_builder
        .with_dimensions((w, h).into())
        .with_fullscreen(Some(event_loop.get_primary_monitor())),
    };

    let windowed_ctx = ContextBuilder::new()
      .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
      .with_gl_profile(GlProfile::Core)
      .with_multisampling(win_opt.num_samples().unwrap_or(0) as u16)
      .with_double_buffer(Some(true))
      .build_windowed(window_builder, &event_loop)?;

    let ctx = unsafe { windowed_ctx.make_current().map_err(|(_, e)| e)? };

    // init OpenGL
    gl::load_with(|s| ctx.get_proc_address(s) as *const c_void);

    match win_opt.cursor_mode() {
      CursorMode::Visible => ctx.window().hide_cursor(false),
      // glutin doesn’t support disabled cursors; default to invisible
      CursorMode::Invisible | CursorMode::Disabled => ctx.window().hide_cursor(true),
    }

    ctx.window().show();

    let gfx_state = GraphicsState::new().map_err(GlutinError::GraphicsStateError)?;
    let surface = GlutinSurface {
      ctx,
      event_loop,
      gfx_state: Rc::new(RefCell::new(gfx_state)),
    };

    Ok(surface)
  }

  /// Get the underlying size (in physical pixels) of the surface.
  ///
  /// This is equivalent to getting the inner size of the windowed context and converting it to
  /// a physical size by using the HiDPI factor of the windowed context.
  pub fn size(&self) -> [u32; 2] {
    let logical = self.ctx.window().get_inner_size().unwrap();
    let (w, h) = PhysicalSize::from_logical(logical, self.ctx.window().get_hidpi_factor()).into();
    [w, h]
  }

  /// Get access to the back buffer.
  pub fn back_buffer(&mut self) -> Result<Framebuffer<Flat, Dim2, (), ()>, GlutinError> {
    Ok(Framebuffer::back_buffer(self, self.size()))
  }

  /// Swap the back and front buffers.
  pub fn swap_buffers(&mut self) {
    let _ = self.ctx.swap_buffers();
  }
}
