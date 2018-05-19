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
}

impl GraphicsState {
  /// Get a `GraphicsContext` from the current OpenGL context.
  pub(crate) fn get_from_context() -> Self {
    unsafe {
      let blending_state = get_ctx_blending_state();
      let blending_equation = get_ctx_blending_equation();
      let blending_func = get_ctx_blending_factors();
      let depth_test = get_ctx_depth_test();
      let face_culling_state = get_ctx_face_culling_state();
      let face_culling_order = get_ctx_face_culling_order();
      let face_culling_mode = get_ctx_face_culling_mode();

      GraphicsState {
        _a: PhantomData,
        blending_state,
        blending_equation,
        blending_func,
        depth_test,
        face_culling_state,
        face_culling_order,
        face_culling_mode,
      }
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

unsafe fn get_ctx_blending_state() -> BlendingState {
  let state = gl::IsEnabled(gl::BLEND);

  match state {
    gl::TRUE => BlendingState::Enabled,
    gl::FALSE => BlendingState::Disabled,
    _ => panic!("unknown blending state: {}", state)
  }
}

unsafe fn get_ctx_blending_equation() -> Equation {
  let mut data = gl::FUNC_ADD as GLint;
  gl::GetIntegerv(gl::BLEND_EQUATION_RGB, &mut data);

  match data as GLenum {
    gl::FUNC_ADD => Equation::Additive,
    gl::FUNC_SUBTRACT => Equation::Subtract,
    gl::FUNC_REVERSE_SUBTRACT => Equation::ReverseSubtract,
    gl::MIN => Equation::Min,
    gl::MAX => Equation::Max,
    _ => panic!("unknown blending equation: {}", data)
  }
}

unsafe fn get_ctx_blending_factors() -> (Factor, Factor) {
  let mut src = gl::ONE as GLint;
  let mut dst = gl::ZERO as GLint;

  gl::GetIntegerv(gl::BLEND_SRC_RGB, &mut src);
  gl::GetIntegerv(gl::BLEND_DST_RGB, &mut dst);

  (from_gl_blending_factor(src as GLenum), from_gl_blending_factor(dst as GLenum))
}

#[inline]
fn from_gl_blending_factor(factor: GLenum) -> Factor {
  match factor {
    gl::ONE => Factor::One,
    gl::ZERO => Factor::Zero,
    gl::SRC_COLOR => Factor::SrcColor,
    gl::ONE_MINUS_SRC_COLOR => Factor::SrcColorComplement,
    gl::DST_COLOR => Factor::DestColor,
    gl::ONE_MINUS_DST_COLOR => Factor::DestColorComplement,
    gl::SRC_ALPHA => Factor::SrcAlpha,
    gl::ONE_MINUS_SRC_ALPHA => Factor::SrcAlphaComplement,
    gl::DST_ALPHA => Factor::DstAlpha,
    gl::ONE_MINUS_DST_ALPHA => Factor::DstAlphaComplement,
    gl::SRC_ALPHA_SATURATE => Factor::SrcAlphaSaturate,
    _ => panic!("unknown blending factor: {}", factor)
  }
}

unsafe fn get_ctx_depth_test() -> DepthTest {
  let state = gl::IsEnabled(gl::DEPTH_TEST);

  match state {
    gl::TRUE => DepthTest::Enabled,
    gl::FALSE => DepthTest::Disabled,
    _ => panic!("unknown depth test: {}", state)
  }
}

unsafe fn get_ctx_face_culling_state() -> FaceCullingState {
  let state = gl::IsEnabled(gl::CULL_FACE);

  match state {
    gl::TRUE => FaceCullingState::Enabled,
    gl::FALSE => FaceCullingState::Disabled,
    _ => panic!("unknown face culling state: {}", state)
  }
}

unsafe fn get_ctx_face_culling_order() -> FaceCullingOrder {
  let mut data = gl::CCW as GLint;
  gl::GetIntegerv(gl::FRONT_FACE, &mut data);

  match data as GLenum {
    gl::CCW => FaceCullingOrder::CCW,
    gl::CW => FaceCullingOrder::CW,
    _ => panic!("unknown face culling order: {}", data)
  }
}

unsafe fn get_ctx_face_culling_mode() -> FaceCullingMode {
  let mut data = gl::BACK as GLint;
  gl::GetIntegerv(gl::CULL_FACE_MODE, &mut data);

  match data as GLenum {
    gl::FRONT => FaceCullingMode::Front,
    gl::BACK => FaceCullingMode::Back,
    gl::FRONT_AND_BACK => FaceCullingMode::Both,
    _ => panic!("unknown face culling mode: {}", data)
  }
}
