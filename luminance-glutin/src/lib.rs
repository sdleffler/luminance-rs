//! The [glutin] windowing implementation for [luminance-windowing].
//!
//! [glutin]: https://crates.io/crates/glutin
//! [luminance-windowing]: https://crates.io/crates/luminance-windowing

#![deny(missing_docs)]

use gl;
pub use glutin;
use glutin::{
  event_loop::EventLoop,
  window::WindowBuilder,
  Api, ContextBuilder, ContextError, CreationError, GlProfile, GlRequest, NotCurrent,
  PossiblyCurrent, WindowedContext,
};

use luminance::context::GraphicsContext;
use luminance::framebuffer::Framebuffer;
use luminance::state::{GraphicsState, StateQueryError};
use luminance::texture::Dim2;
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
  gfx_state: Rc<RefCell<GraphicsState>>,
}

unsafe impl GraphicsContext for GlutinSurface {
  fn state(&self) -> &Rc<RefCell<GraphicsState>> {
    &self.gfx_state
  }
}

impl GlutinSurface {
  /// Create a new [`GlutinSurface`] by consuming a [`WindowBuilder`].
  ///
  /// This is an alternative method to [`new`] that is more flexible as you have access to the
  /// whole `glutin` types.
  ///
  /// `window_builder` is the default object when passed to your closure and `ctx_builder` is
  /// already initialized for the OpenGL context (you’re not supposed to change it!).
  pub fn from_builders<WB, CB>(window_builder: WB, ctx_builder: CB) -> Result<(Self, EventLoop<()>), GlutinError>
  where
    WB: FnOnce(WindowBuilder) -> WindowBuilder,
    CB: FnOnce(ContextBuilder<NotCurrent>) -> ContextBuilder<NotCurrent>,
  {
    let event_loop = EventLoop::new();

    let window_builder = window_builder(WindowBuilder::new());

    let windowed_ctx = ctx_builder(
      ContextBuilder::new()
        .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
        .with_gl_profile(GlProfile::Core),
    )
    .build_windowed(window_builder, &event_loop)?;

    let ctx = unsafe { windowed_ctx.make_current().map_err(|(_, e)| e)? };

    // init OpenGL
    gl::load_with(|s| ctx.get_proc_address(s) as *const c_void);

    ctx.window().set_visible(true);

    let gfx_state = GraphicsState::new().map_err(GlutinError::GraphicsStateError)?;
    let surface = GlutinSurface {
      ctx,
      gfx_state: Rc::new(RefCell::new(gfx_state)),
    };

    Ok((surface, event_loop))
  }

  /// Create a new [`GlutinSurface`] from scratch.
  pub fn new(window_builder: WindowBuilder, samples: u16) -> Result<(Self, EventLoop<()>), GlutinError> {
    let event_loop = EventLoop::new();

    let windowed_ctx = ContextBuilder::new()
      .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
      .with_gl_profile(GlProfile::Core)
      .with_multisampling(samples)
      .with_double_buffer(Some(true))
      .build_windowed(window_builder, &event_loop)?;

    let ctx = unsafe { windowed_ctx.make_current().map_err(|(_, e)| e)? };

    // init OpenGL
    gl::load_with(|s| ctx.get_proc_address(s) as *const c_void);

    ctx.window().set_visible(true);

    let gfx_state = GraphicsState::new().map_err(GlutinError::GraphicsStateError)?;
    let surface = GlutinSurface {
      ctx,
      gfx_state: Rc::new(RefCell::new(gfx_state)),
    };

    Ok((surface, event_loop))
  }

  /// Get the underlying size (in physical pixels) of the surface.
  ///
  /// This is equivalent to getting the inner size of the windowed context and converting it to
  /// a physical size by using the HiDPI factor of the windowed context.
  pub fn size(&self) -> [u32; 2] {
    let size = self.ctx.window().inner_size();
    [size.width, size.height]
  }

  /// Get access to the back buffer.
  pub fn back_buffer(&mut self) -> Result<Framebuffer<Dim2, (), ()>, GlutinError> {
    Ok(Framebuffer::back_buffer(self, self.size()))
  }

  /// Swap the back and front buffers.
  pub fn swap_buffers(&mut self) {
    let _ = self.ctx.swap_buffers();
  }
}
