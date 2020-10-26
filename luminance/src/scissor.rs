//! Scissor test and related types.
//!
//! The scissor test is a special test performed at rendering time. It allows to define a region of the screen for which
//! fragments will be discarded.

/// The region outside of which fragments will be discarded.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ScissorRegion {
  /// The x screen position of the scissor region.
  pub x: u32,

  /// The y screen position of the scissor region.
  pub y: u32,

  /// The screen width of the scissor region.
  pub width: u32,

  /// The screen height of the scissor region.
  pub height: u32,
}
