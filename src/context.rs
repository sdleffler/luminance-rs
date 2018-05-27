//! Graphics context.
//!
//! A graphics context is an object that abstracts all the low-level operations that happen on a
//! graphics device (it can be a GPU or a software implementation, for instance).
//!
//! This crate doesn’t provide you with creating such contexts. Instead, you must do it yourself
//! or rely on crates doing it for you.
//!
//! # On context and threads
//!
//! This crate is designed to work with the following principles:
//!
//!   - An object which type implements `GraphicsContext` must be `!Send` and `!Sync`. This
//!     enforces that it cannot be moved nor shared between threads.
//!   - You can only create a single context per thread. Doing otherwise is undefined behavior.
//!   - You can create as many contexts as you want as long as they respectively live on a
//!     separate thread. In other terms, if you want `n` contexts, you need `n` threads.
//!
//! That last property might seem to be a drawback to you but is required to remove a lot of
//! dynamic branches in the implementation and reduce the number of required safety checks.

use std::cell::RefCell;
use std::rc::Rc;

use pipeline::Builder;
use state::{GraphicsState, StateQueryError};

/// Class of graphics context.
///
/// Such a context must not be Send nor Sync, which means that you cannot share it between
/// threads in any way (move / borrow).
pub unsafe trait GraphicsContext {
  /// Get access to the graphics state of this context.
  ///
  /// This must return a `Rc<RefCell<GraphicsState>>` because the state will be shared by OpenGL
  /// objects to ensure consistency with its state.
  fn state(&self) -> &Rc<RefCell<GraphicsState>>;

  /// Create a new pipeline builder.
  ///
  /// A pipeline builder is the only way to create new pipelines and issue draws.
  fn pipeline_builder(&self) -> Builder {
    Builder::new(self.state().clone())
  }

  /// Swap the back and front buffers.
  fn swap_buffers(&mut self);
}

thread_local!(static TLS_ACQUIRE_CONTEXT: RefCell<Option<()>> = RefCell::new(Some(())));

/// Class of `GraphicsContext` builders.
///
/// This trait must be implemented in order to generate graphics contexts.
pub trait WithGraphicsState {
  /// The custom 'GraphicsContext' to return.
  type Output: GraphicsContext;

  /// Call the builder and consume it. You’re handed a function that gives you a `GraphicsState` in
  /// order to return it in your object.
  fn call_once<F>(self, f: F) -> Self::Output where F: FnOnce() -> Result<GraphicsState, StateQueryError>;
}

/// Acquire a context per-thread (if none was previously initialized yet).
///
/// This function must be called by any implementor of `GraphicsContext` in order to generate the
/// context object. This prevents from having two contexts allocated at the same time on the same
/// thread.
///
/// Called in a thread, this will lazily evaluate the closure passed as argument only if no call to
/// this function was made prior to the current call.
pub fn thread_acquire_context<F>(f: F) -> Option<F::Output> where F: WithGraphicsState {
  TLS_ACQUIRE_CONTEXT.with(|rc| {
    let mut inner = rc.borrow_mut();

    match *inner {
      Some(_) => {
        inner.take();
        Some(f.call_once(GraphicsState::get_from_context))
      },

      None => None
    }
  })
}
