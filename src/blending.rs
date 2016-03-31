//! That module exports blending-related types and functions.
//!
//! Given two pixels *src* and *dst* – source and destination, we associate each pixel a blending
//! factor – respectively, *srcK* and *dstK*. *src* is the pixel being computed, and *dst* is the
//! pixel that is already stored in the framebuffer.
//!
//! The pixels can be blended in several ways. See the documentation of `Equation` for further
//! details.
//!
//! The factors are encoded with `Factor`.

/// Blending equation.
#[derive(Copy, Clone, Debug)]
pub enum Equation {
  /// `Additive` represents the following blending equation:
  ///
  /// > `blended = src * srcK + dst * dstK`
  Additive,
  /// `Subtract` represents the following blending equation:
  ///
  /// > `blended = src * srcK - dst * dstK`
  Subtract,
  /// Because subtracting is not commutative, `ReverseSubtract` represents the following additional
  /// blending equation:
  ///
  /// > `blended = dst * dstK - src * srcK`
  ReverseSubtract,
  /// `Min` represents the following blending equation:
  ///
  /// > `blended = min(src, dst)`
  Min,
  /// `Max` represents the following blending equation:
  ///
  /// > `blended = max(src, dst)`
  Max
}

#[derive(Copy, Clone, Debug)]
pub enum Factor {
  /// 1 * color = factor
  One,
  /// 0 * color = 0
  Zero,
  /// src * color
  SrcColor,
  /// (1 - src) * color
  NegativeSrcColor,
  /// dst * color
  DestColor,
  /// (1 - dst) * color
  NegativeDestColor,
  /// srcA * color
  SrcAlpha,
  /// (1 - src) * color
  NegativeSrcAlpha,
  /// dstA * color
  DstAlpha,
  /// (1 - dstA) * color
  NegativeDstAlpha,
  SrcAlphaSaturate
}
