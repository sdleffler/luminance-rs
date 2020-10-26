//! WebGL 2.0 backend support.

mod array_buffer;
pub mod buffer;
pub mod framebuffer;
pub mod pipeline;
pub mod pixel;
pub mod shader;
pub mod state;
pub mod tess;
pub mod texture;

pub use crate::webgl2::array_buffer::IntoArrayBuffer;
pub use crate::webgl2::state::StateQueryError;
use crate::webgl2::state::WebGL2State;
use std::cell::RefCell;
use std::rc::Rc;
use web_sys::WebGl2RenderingContext;

/// The WebGL2 backend.
#[derive(Debug)]
pub struct WebGL2 {
  pub(crate) state: Rc<RefCell<WebGL2State>>,
}

impl WebGL2 {
  pub fn new(ctx: WebGl2RenderingContext) -> Result<Self, StateQueryError> {
    WebGL2State::new(ctx).map(|state| WebGL2 {
      state: Rc::new(RefCell::new(state)),
    })
  }
}
