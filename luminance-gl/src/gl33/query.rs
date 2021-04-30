//! Query API implementation for OpenGL 3.3.

use crate::GL33;
use luminance::backend::query::{Query as QueryBackend, QueryError};

unsafe impl QueryBackend for GL33 {
  fn backend_author(&self) -> Result<String, QueryError> {
    let name = self.state.borrow_mut().get_vendor_name();
    Ok(name)
  }

  fn backend_name(&self) -> Result<String, QueryError> {
    let name = self.state.borrow_mut().get_renderer_name();
    Ok(name)
  }

  fn backend_version(&self) -> Result<String, QueryError> {
    let name = self.state.borrow_mut().get_gl_version();
    Ok(name)
  }

  fn backend_shading_lang_version(&self) -> Result<String, QueryError> {
    let name = self.state.borrow_mut().get_glsl_version();
    Ok(name)
  }
}
