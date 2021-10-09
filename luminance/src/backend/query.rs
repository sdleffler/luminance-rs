//! Query backend interface.
//!
//! This interface provides various means to query some metrics and data from the backend, such as the maximum number of
//! active texture units, memory sizes, etc.

use std::fmt;

/// Query error.
#[derive(Debug)]
pub enum QueryError {
  /// No backend author information available.
  NoBackendAuthor,

  /// No backend name information available.
  NoBackendName,

  /// No backend version information available.
  NoBackendVersion,

  /// No backend shading language version information available.
  NoBackendShadingLanguageVersion,

  /// No maximum number of elements for texture arrays information available.
  NoMaxTextureArrayElements,
}

impl fmt::Display for QueryError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      QueryError::NoBackendAuthor => f.write_str("no backend author available"),
      QueryError::NoBackendName => f.write_str("no backend name available"),
      QueryError::NoBackendVersion => f.write_str("no backend version available"),
      QueryError::NoBackendShadingLanguageVersion => {
        f.write_str("no backend shading language version available")
      }
      QueryError::NoMaxTextureArrayElements => {
        f.write_str("no maximum number of elements for texture arrays available")
      }
    }
  }
}

/// Backends that support querying.
///
/// Querying provide metadata information about the backend, but can also provide more useful information, such as
/// capabilities, maximum limits, etc.
pub unsafe trait Query {
  /// The implementation author, most of the time referred to as “vendor” or “company” responsible for the driver the
  /// backend uses.
  fn backend_author(&self) -> Result<String, QueryError>;

  /// The backend name.
  fn backend_name(&self) -> Result<String, QueryError>;

  /// The backend version.
  fn backend_version(&self) -> Result<String, QueryError>;

  /// The shading language version supported by the backend.
  fn backend_shading_lang_version(&self) -> Result<String, QueryError>;

  /// The maximum number of elements a texture array can hold.
  fn max_texture_array_elements(&self) -> Result<usize, QueryError>;
}
