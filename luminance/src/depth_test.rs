//! Depth test related features.

/// Whether or not depth test should be enabled.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DepthTest {
  /// The depth test is enabled.
  Enabled,
  /// The depth test is disabled.
  Disabled,
}
