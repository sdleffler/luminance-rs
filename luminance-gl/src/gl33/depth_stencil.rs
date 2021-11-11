use gl::types::*;

use luminance::depth_stencil::{Comparison, StencilOp};

pub(crate) fn comparison_to_glenum(dc: Comparison) -> GLenum {
  match dc {
    Comparison::Never => gl::NEVER,
    Comparison::Always => gl::ALWAYS,
    Comparison::Equal => gl::EQUAL,
    Comparison::NotEqual => gl::NOTEQUAL,
    Comparison::Less => gl::LESS,
    Comparison::LessOrEqual => gl::LEQUAL,
    Comparison::Greater => gl::GREATER,
    Comparison::GreaterOrEqual => gl::GEQUAL,
  }
}

pub(crate) fn glenum_to_comparison(a: GLenum) -> Option<Comparison> {
  match a {
    gl::NEVER => Some(Comparison::Never),
    gl::ALWAYS => Some(Comparison::Always),
    gl::EQUAL => Some(Comparison::Equal),
    gl::NOTEQUAL => Some(Comparison::NotEqual),
    gl::LESS => Some(Comparison::Less),
    gl::LEQUAL => Some(Comparison::LessOrEqual),
    gl::GREATER => Some(Comparison::Greater),
    gl::GEQUAL => Some(Comparison::GreaterOrEqual),
    _ => None,
  }
}

pub(crate) fn stencil_op_to_glenum(op: StencilOp) -> GLenum {
  match op {
    StencilOp::Keep => gl::KEEP,
    StencilOp::Zero => gl::ZERO,
    StencilOp::Replace => gl::REPLACE,
    StencilOp::Increment => gl::INCR,
    StencilOp::IncrementWrap => gl::INCR_WRAP,
    StencilOp::Decrement => gl::DECR,
    StencilOp::DecrementWrap => gl::DECR_WRAP,
    StencilOp::Invert => gl::INVERT,
  }
}

pub(crate) fn glenum_to_stencil_op(a: GLenum) -> Option<StencilOp> {
  match a {
    gl::KEEP => Some(StencilOp::Keep),
    gl::ZERO => Some(StencilOp::Zero),
    gl::REPLACE => Some(StencilOp::Replace),
    gl::INCR => Some(StencilOp::Increment),
    gl::INCR_WRAP => Some(StencilOp::IncrementWrap),
    gl::DECR => Some(StencilOp::Decrement),
    gl::DECR_WRAP => Some(StencilOp::DecrementWrap),
    gl::INVERT => Some(StencilOp::Invert),
    _ => None,
  }
}
