//! Query backend interface.
//!
//! This interface provides various means to query some metrics and data from the GPU, such as the maximum number of
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
    }
  }
}

/// Backends that support querying.
pub unsafe trait Query {
  /// The implementation author, most of the time referred to as “vendor” or “compagny” responsible for the driver the
  /// implementation uses.
  fn backend_author(&self) -> Result<String, QueryError>;

  /// The backend name.
  fn backend_name(&self) -> Result<String, QueryError>;

  /// The backend version.
  fn backend_version(&self) -> Result<String, QueryError>;

  /// The shading language version.
  fn backend_shading_lang_version(&self) -> Result<String, QueryError>;
}
