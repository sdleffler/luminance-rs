//! GPU queries.
//!
//! GPU queries allow to get information about the backend and the GPU in a straight-forward way.

use crate::{
  backend::query::{Query as QueryBackend, QueryError},
  context::GraphicsContext,
};

/// Query object.
///
/// Such an object allows to query various parts of the backend and GPU.
#[derive(Debug)]
pub struct Query<'a, B>
where
  B: ?Sized,
{
  backend: &'a B,
}

impl<'a, B> Query<'a, B>
where
  B: ?Sized + QueryBackend,
{
  /// Create a new [`Query`] for a given context.
  pub fn new(ctxt: &'a mut impl GraphicsContext<Backend = B>) -> Self {
    let backend = ctxt.backend();
    Self { backend }
  }

  /// The implementation author, most of the time referred to as “vendor” or “compagny” responsible for the driver the
  /// implementation uses.
  pub fn backend_author(&self) -> Result<String, QueryError> {
    self.backend.backend_author()
  }

  /// The backend name.
  pub fn backend_name(&self) -> Result<String, QueryError> {
    self.backend.backend_name()
  }

  /// The backend version.
  pub fn backend_version(&self) -> Result<String, QueryError> {
    self.backend.backend_version()
  }

  /// The shading language version.
  pub fn backend_shading_lang_version(&self) -> Result<String, QueryError> {
    self.backend.backend_shading_lang_version()
  }

  /// Maximum number of elements a texture array can hold.
  pub fn max_texture_array_elements(&self) -> Result<usize, QueryError> {
    self.backend.max_texture_array_elements()
  }
}
