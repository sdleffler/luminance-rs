use gl;
use gl::types::*;
use std::marker::PhantomData;

use blending::{BlendingState, Equation, Factor};
use depth_test::DepthTest;
use face_culling::{FaceCullingMode, FaceCullingOrder, FaceCullingState};

/// The graphics state.
///
/// This type represents the current state of a given graphics context. It acts
/// as a forward-gate to all the exposed features from the low-level API but
/// adds a small cache layer over it to prevent from issuing twiche the same API
/// call (with the same parameters).
pub struct GraphicsState {
  _a: PhantomData<*const ()>, // !Send and !Sync
  
  // blending
  blending_state: BlendingState,
  blending_equation: Equation,
  blending_func: (Factor, Factor),

  // depth test
  depth_test: DepthTest,

  // face culling
  face_culling_state: FaceCullingState,
  face_culling_order: FaceCullingOrder,
  face_culling_mode: FaceCullingMode,

  //// texture
  //next_texture_unit: GLenum
}

impl GraphicsState {
  /// Get a `GraphicsContext` from the current OpenGL context.
  pub(crate) fn get_from_context() -> Result<Self, StateQueryError> {
    unsafe {
      let blending_state = get_ctx_blending_state()?;
      let blending_equation = get_ctx_blending_equation()?;
      let blending_func = get_ctx_blending_factors()?;
      let depth_test = get_ctx_depth_test()?;
      let face_culling_state = get_ctx_face_culling_state()?;
      let face_culling_order = get_ctx_face_culling_order()?;
      let face_culling_mode = get_ctx_face_culling_mode()?;
      //let next_texture_unit = get_ctx_next_texture_unit();

      Ok(GraphicsState {
        _a: PhantomData,
        blending_state,
        blending_equation,
        blending_func,
        depth_test,
        face_culling_state,
        face_culling_order,
        face_culling_mode,
      })
    }
  }

  // blending
  #[inline(always)]
  pub(crate) unsafe fn set_blending_state(&mut self, state: BlendingState) {
    if self.blending_state != state {
      match state {
        BlendingState::Enabled => gl::Enable(gl::BLEND),
        BlendingState::Disabled => gl::Disable(gl::BLEND)
      }

      self.blending_state = state;
    }
  }

  #[inline(always)]
  pub(crate) unsafe fn set_blending_equation(&mut self, equation: Equation) {
    if self.blending_equation != equation {
      gl::BlendEquation(from_blending_equation(equation));
      self.blending_equation = equation;
    }
  }

  #[inline(always)]
  pub(crate) unsafe fn set_blending_func(
    &mut self,
    src: Factor,
    dest: Factor,
  ) {
    if self.blending_func != (src, dest) {
      gl::BlendFunc(from_blending_factor(src), from_blending_factor(dest));
      self.blending_func = (src, dest);
    }
  }

  #[inline(always)]
  pub(crate) unsafe fn set_depth_test(&mut self, depth_test: DepthTest) {
    if self.depth_test != depth_test {
      match depth_test {
        DepthTest::Enabled => gl::Enable(gl::DEPTH_TEST),
        DepthTest::Disabled => gl::Disable(gl::DEPTH_TEST)
      }

      self.depth_test = depth_test;
    }
  }

  #[inline(always)]
  pub(crate) unsafe fn set_face_culling_state(&mut self, state: FaceCullingState) {
    if self.face_culling_state != state {
      match state {
        FaceCullingState::Enabled => gl::Enable(gl::CULL_FACE),
        FaceCullingState::Disabled => gl::Disable(gl::CULL_FACE)
      }

      self.face_culling_state = state;
    }
  }

  #[inline(always)]
  pub(crate) unsafe fn set_face_culling_order(&mut self, order: FaceCullingOrder) {
    if self.face_culling_order != order {
      match order {
        FaceCullingOrder::CW => gl::FrontFace(gl::CW),
        FaceCullingOrder::CCW => gl::FrontFace(gl::CCW)
      }

      self.face_culling_order = order;
    }
  }

  #[inline(always)]
  pub(crate) unsafe fn set_face_culling_mode(&mut self, mode: FaceCullingMode) {
    if self.face_culling_mode != mode {
      match mode {
        FaceCullingMode::Front => gl::CullFace(gl::FRONT),
        FaceCullingMode::Back => gl::CullFace(gl::BACK),
        FaceCullingMode::Both => gl::CullFace(gl::FRONT_AND_BACK)
      }

      self.face_culling_mode = mode;
    }
  }
}

#[inline]
fn from_blending_equation(equation: Equation) -> GLenum {
  match equation {
    Equation::Additive => gl::FUNC_ADD,
    Equation::Subtract => gl::FUNC_SUBTRACT,
    Equation::ReverseSubtract => gl::FUNC_REVERSE_SUBTRACT,
    Equation::Min => gl::MIN,
    Equation::Max => gl::MAX
  }
}

#[inline]
fn from_blending_factor(factor: Factor) -> GLenum {
  match factor {
    Factor::One => gl::ONE,
    Factor::Zero => gl::ZERO,
    Factor::SrcColor => gl::SRC_COLOR,
    Factor::SrcColorComplement => gl::ONE_MINUS_SRC_COLOR,
    Factor::DestColor => gl::DST_COLOR,
    Factor::DestColorComplement => gl::ONE_MINUS_DST_COLOR,
    Factor::SrcAlpha => gl::SRC_ALPHA,
    Factor::SrcAlphaComplement => gl::ONE_MINUS_SRC_ALPHA,
    Factor::DstAlpha => gl::DST_ALPHA,
    Factor::DstAlphaComplement => gl::ONE_MINUS_DST_ALPHA,
    Factor::SrcAlphaSaturate => gl::SRC_ALPHA_SATURATE
  }
}

/// An error that might happen when the context is queried.
#[derive(Debug)]
pub enum StateQueryError {
  UnknownBlendingState(GLboolean),
  UnknownBlendingEquation(GLenum),
  UnknownBlendingSrcFactor(GLenum),
  UnknownBlendingDstFactor(GLenum),
  UnknownDepthTestState(GLboolean),
  UnknownFaceCullingState(GLboolean),
  UnknownFaceCullingOrder(GLenum),
  UnknownFaceCullingMode(GLenum),
}

unsafe fn get_ctx_blending_state() -> Result<BlendingState, StateQueryError> {
  let state = gl::IsEnabled(gl::BLEND);

  match state {
    gl::TRUE => Ok(BlendingState::Enabled),
    gl::FALSE => Ok(BlendingState::Disabled),
    _ => Err(StateQueryError::UnknownBlendingState(state))
  }
}

unsafe fn get_ctx_blending_equation() -> Result<Equation, StateQueryError> {
  let mut data = gl::FUNC_ADD as GLint;
  gl::GetIntegerv(gl::BLEND_EQUATION_RGB, &mut data);

  let data = data as GLenum;
  match data {
    gl::FUNC_ADD => Ok(Equation::Additive),
    gl::FUNC_SUBTRACT => Ok(Equation::Subtract),
    gl::FUNC_REVERSE_SUBTRACT => Ok(Equation::ReverseSubtract),
    gl::MIN => Ok(Equation::Min),
    gl::MAX => Ok(Equation::Max),
    _ => Err(StateQueryError::UnknownBlendingEquation(data))
  }
}

unsafe fn get_ctx_blending_factors() -> Result<(Factor, Factor), StateQueryError> {
  let mut src = gl::ONE as GLint;
  let mut dst = gl::ZERO as GLint;

  gl::GetIntegerv(gl::BLEND_SRC_RGB, &mut src);
  gl::GetIntegerv(gl::BLEND_DST_RGB, &mut dst);

  let src_k = from_gl_blending_factor(src as GLenum).map_err(StateQueryError::UnknownBlendingSrcFactor)?;
  let dst_k = from_gl_blending_factor(dst as GLenum).map_err(StateQueryError::UnknownBlendingDstFactor)?;

  Ok((src_k, dst_k))
}

#[inline]
fn from_gl_blending_factor(factor: GLenum) -> Result<Factor, GLenum> {
  match factor {
    gl::ONE => Ok(Factor::One),
    gl::ZERO => Ok(Factor::Zero),
    gl::SRC_COLOR => Ok(Factor::SrcColor),
    gl::ONE_MINUS_SRC_COLOR => Ok(Factor::SrcColorComplement),
    gl::DST_COLOR => Ok(Factor::DestColor),
    gl::ONE_MINUS_DST_COLOR => Ok(Factor::DestColorComplement),
    gl::SRC_ALPHA => Ok(Factor::SrcAlpha),
    gl::ONE_MINUS_SRC_ALPHA => Ok(Factor::SrcAlphaComplement),
    gl::DST_ALPHA => Ok(Factor::DstAlpha),
    gl::ONE_MINUS_DST_ALPHA => Ok(Factor::DstAlphaComplement),
    gl::SRC_ALPHA_SATURATE => Ok(Factor::SrcAlphaSaturate),
    _ => Err(factor)
  }
}

unsafe fn get_ctx_depth_test() -> Result<DepthTest, StateQueryError> {
  let state = gl::IsEnabled(gl::DEPTH_TEST);

  match state {
    gl::TRUE => Ok(DepthTest::Enabled),
    gl::FALSE => Ok(DepthTest::Disabled),
    _ => Err(StateQueryError::UnknownDepthTestState(state))
  }
}

unsafe fn get_ctx_face_culling_state() -> Result<FaceCullingState, StateQueryError> {
  let state = gl::IsEnabled(gl::CULL_FACE);

  match state {
    gl::TRUE => Ok(FaceCullingState::Enabled),
    gl::FALSE => Ok(FaceCullingState::Disabled),
    _ => Err(StateQueryError::UnknownFaceCullingState(state))
  }
}

unsafe fn get_ctx_face_culling_order() -> Result<FaceCullingOrder, StateQueryError> {
  let mut order = gl::CCW as GLint;
  gl::GetIntegerv(gl::FRONT_FACE, &mut order);

  let order = order as GLenum;
  match order {
    gl::CCW => Ok(FaceCullingOrder::CCW),
    gl::CW => Ok(FaceCullingOrder::CW),
    _ => Err(StateQueryError::UnknownFaceCullingOrder(order))
  }
}

unsafe fn get_ctx_face_culling_mode() -> Result<FaceCullingMode, StateQueryError> {
  let mut mode = gl::BACK as GLint;
  gl::GetIntegerv(gl::CULL_FACE_MODE, &mut mode);

  let mode = mode as GLenum;
  match mode {
    gl::FRONT => Ok(FaceCullingMode::Front),
    gl::BACK => Ok(FaceCullingMode::Back),
    gl::FRONT_AND_BACK => Ok(FaceCullingMode::Both),
    _ => Err(StateQueryError::UnknownFaceCullingMode(mode))
  }
}
