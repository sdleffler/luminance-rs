//! That module exports blending-related types and functions.
//! 
//! Given two pixels *src* and *dst* – source and destination, we associate each pixel a blending
//! factor – respectively, *srcK* and *dstK*. *src* is the pixel being computed, and *dst* is the
//! pixel that is already stored in the framebuffer.
//! 
//! The pixels can be blended in several ways. See the documentation of `Mode` for further details.
//! 
//! The factors are encoded with `Factor`.
pub enum Mode {
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
