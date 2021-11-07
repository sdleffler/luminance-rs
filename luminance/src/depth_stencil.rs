//! Depth test related features.

/// Comparison to perform for depth / stencil operations. `a` is the incoming fragment’s data and b is the fragment’s
/// data that is already stored.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Comparison {
  /// Test never succeeds.
  Never,
  /// Test always succeeds.
  Always,
  /// Test succeeds if `a == b`.
  Equal,
  /// Test succeeds if `a != b`.
  NotEqual,
  /// Test succeeds if `a < b`.
  Less,
  /// Test succeeds if `a <= b`.
  LessOrEqual,
  /// Test succeeds if `a > b`.
  Greater,
  /// Test succeeds if `a >= b`.
  GreaterOrEqual,
}

/// Whether or not writes should be performed when rendering.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Write {
  /// Write values.
  On,
  /// Do not write values.
  Off,
}

/// The stencil test is a bit weird. It’s a [`Comparison`] as well as the « stencil mask ».
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct StencilTest {
  /// Comparison to apply to make a fragment pass the test.
  pub comparison: Comparison,

  /// Reference value for the comparison.
  pub reference: u8,

  /// The mask to apply on the fragment stencil value.
  pub mask: u8,
}

impl StencilTest {
  /// Create a new [`StencilTest`] from the comparison, reference and mask values.
  pub fn new(comparison: Comparison, reference: u8, mask: u8) -> Self {
    Self {
      comparison,
      reference,
      mask,
    }
  }
}

/// The stencil operations are executed whenever a stencil test passes.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct StencilOperations {
  /// Action to take when the depth test passes but not the stencil test.
  pub depth_passes_stencil_fails: StencilOp,

  /// Action to take when the stencil test passes but not the depth test.
  pub depth_fails_stencil_passes: StencilOp,

  /// Action to take when both the depth and stencil tests pass.
  pub depth_stencil_pass: StencilOp,
}

impl StencilOperations {
  /// Create [`Default`] [`StencilOperations`].
  pub fn new() -> Self {
    Self::default()
  }

  /// Set the [`StencilOp`] to do when the depth test passes but stencil test fails:
  pub fn on_depth_passes_stencil_fails(self, op: StencilOp) -> Self {
    Self {
      depth_passes_stencil_fails: op,
      ..self
    }
  }

  /// Set the [`StencilOp`] to do when the depth test fails but stencil test passes:
  pub fn on_depth_fails_stencil_passes(self, op: StencilOp) -> Self {
    Self {
      depth_fails_stencil_passes: op,
      ..self
    }
  }

  /// Set the [`StencilOp`] to do when both the depth test and stencil test pass:
  pub fn on_depth_stencil_pass(self, op: StencilOp) -> Self {
    Self {
      depth_stencil_pass: op,
      ..self
    }
  }
}

/// Default implementation for [`StencilOperations`]:
///
/// - when depth test passes but stencil fail: [`StencilOp::Keep`].
/// - when depth test fails but stencil passes: [`StencilOp::Keep`].
/// - when both depth test and stencil test pass: [`StencilOp::Keep`].
impl Default for StencilOperations {
  fn default() -> Self {
    Self {
      depth_passes_stencil_fails: StencilOp::Keep,
      depth_fails_stencil_passes: StencilOp::Keep,
      depth_stencil_pass: StencilOp::Keep,
    }
  }
}

/// Possible stencil operations.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum StencilOp {
  /// Keep the current value.
  Keep,

  /// Set the stencil value to zero.
  Zero,

  /// Replace the stencil value.
  Replace,

  /// Increment the stencil value.
  ///
  /// If the stencil value reaches the maximum possible value, it is clamped.
  Increment,

  /// Increment the stencil value.
  ///
  /// If the stencil value reaches the maximum possible value, it wraps around back to `0`.
  IncrementWrap,

  /// Decrement the stencil value.
  ///
  /// If the stencil value reaches 0, it is clamped.
  Decrement,

  /// Decrement the stencil value.
  ///
  /// If the stencil value reaches 0, it wraps back to the maximum value.
  DecrementWrap,

  /// Bit-wise inversion.
  Invert,
}
