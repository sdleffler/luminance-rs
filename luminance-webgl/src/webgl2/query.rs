//! Query API implementation.

use crate::WebGL2;
use luminance::backend::query::{Query as QueryBackend, QueryError};

unsafe impl QueryBackend for WebGL2 {
  fn backend_author(&self) -> Result<String, QueryError> {
    self
      .state
      .borrow_mut()
      .get_vendor_name()
      .ok_or_else(|| QueryError::NoBackendAuthor)
  }

  fn backend_name(&self) -> Result<String, QueryError> {
    self
      .state
      .borrow_mut()
      .get_renderer_name()
      .ok_or_else(|| QueryError::NoBackendName)
  }

  fn backend_version(&self) -> Result<String, QueryError> {
    self
      .state
      .borrow_mut()
      .get_webgl_version()
      .ok_or_else(|| QueryError::NoBackendVersion)
  }

  fn backend_shading_lang_version(&self) -> Result<String, QueryError> {
    self
      .state
      .borrow_mut()
      .get_glsl_version()
      .ok_or_else(|| QueryError::NoBackendShadingLanguageVersion)
  }
}
