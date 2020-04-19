//! Graphics state.

use std::cell::RefCell;
use std::convert::TryInto;
use std::fmt;
use std::marker::PhantomData;
use web_sys::{WebGl2RenderingContext, WebGlBuffer};

use luminance::blending::{Equation, Factor};
use luminance::depth_test::DepthComparison;
use luminance::face_culling::{FaceCullingMode, FaceCullingOrder};

// TLS synchronization barrier for `GLState`.
thread_local!(static TLS_ACQUIRE_GFX_STATE: RefCell<Option<()>> = RefCell::new(Some(())));

pub(crate) struct BindingStack {
  pub(crate) next_texture_unit: u32,
  pub(crate) free_texture_units: Vec<u32>,
  pub(crate) next_buffer_binding: u32,
  pub(crate) free_buffer_bindings: Vec<u32>,
}

impl BindingStack {
  // Create a new, empty binding stack.
  fn new() -> Self {
    BindingStack {
      next_texture_unit: 0,
      free_texture_units: Vec::new(),
      next_buffer_binding: 0,
      free_buffer_bindings: Vec::new(),
    }
  }
}

/// The graphics state.
///
/// This type represents the current state of a given graphics context. It acts
/// as a forward-gate to all the exposed features from the low-level API but
/// adds a small cache layer over it to prevent from issuing the same API call (with
/// the same parameters).
pub struct WebGL2State {
  _phantom: PhantomData<*const ()>, // !Send and !Sync

  // WebGL context
  pub(crate) ctx: WebGl2RenderingContext,

  // binding stack
  binding_stack: BindingStack,

  // // viewport
  // viewport: [GLint; 4],

  // // clear buffers
  // clear_color: [GLfloat; 4],

  // // blending
  // blending_state: BlendingState,
  // blending_equation: Equation,
  // blending_func: (Factor, Factor),

  // // depth test
  // depth_test: DepthTest,
  // depth_test_comparison: DepthComparison,

  // // face culling
  // face_culling_state: FaceCullingState,
  // face_culling_order: FaceCullingOrder,
  // face_culling_mode: FaceCullingMode,

  // // vertex restart
  // vertex_restart: VertexRestart,

  // // patch primitive vertex number
  // patch_vertex_nb: usize,

  // // texture
  // current_texture_unit: GLenum,
  // bound_textures: Vec<(GLenum, GLuint)>,

  // // texture buffer used to optimize texture creation; regular textures typically will never ask
  // // for fetching from this set but framebuffers, who often generate several textures, might use
  // // this opportunity to get N textures (color, depth and stencil) at once, in a single CPU / GPU
  // // roundtrip
  // //
  // // fishy fishy
  // texture_swimming_pool: Vec<GLuint>,

  // // uniform buffer
  // bound_uniform_buffers: Vec<GLuint>,

  // array buffer
  bound_array_buffer: Option<WebGlBuffer>,
  // // element buffer
  // bound_element_array_buffer: GLuint,

  // // framebuffer
  // bound_draw_framebuffer: GLuint,

  // // vertex array
  // bound_vertex_array: GLuint,

  // // shader program
  // current_program: GLuint,
}

impl WebGL2State {
  /// Create a new `GLState`.
  ///
  /// > Note: keep in mind you can create only one per thread. However, if you’re building without
  /// > standard library, this function will always return successfully. You have to take extra care
  /// > in this case.
  pub(crate) fn new(ctx: WebGl2RenderingContext) -> Result<Self, StateQueryError> {
    TLS_ACQUIRE_GFX_STATE.with(|rc| {
      let mut inner = rc.borrow_mut();

      match *inner {
        Some(_) => {
          inner.take();
          Self::get_from_context(ctx)
        }

        None => Err(StateQueryError::UnavailableGLState),
      }
    })
  }

  /// Get a `GraphicsContext` from the current OpenGL context.
  fn get_from_context(ctx: WebGl2RenderingContext) -> Result<Self, StateQueryError> {
    let binding_stack = BindingStack::new();
    //let viewport = Self::get_ctx_viewport(ctx)?;
    //let clear_color = Self::get_ctx_clear_color(ctx)?;
    //let blending_state = Self::get_ctx_blending_state(ctx)?;
    //let blending_equation = Self::get_ctx_blending_equation(ctx)?;
    //let blending_func = Self::get_ctx_blending_factors(ctx)?;
    //let depth_test = Self::get_ctx_depth_test(ctx)?;
    //let depth_test_comparison = DepthComparison::Less;
    //let face_culling_state = Self::get_ctx_face_culling_state(ctx)?;
    //let face_culling_order = Self::get_ctx_face_culling_order(ctx)?;
    //let face_culling_mode = Self::get_ctx_face_culling_mode(ctx)?;
    //let vertex_restart = Self::get_ctx_vertex_restart(ctx)?;
    //let patch_vertex_nb = 0;
    //let current_texture_unit = Self::get_ctx_current_texture_unit(ctx)?;
    //let bound_textures = vec![(WebGl2RenderingContext::TEXTURE_2D, 0); 48]; // 48 is the platform minimal requirement
    //let texture_swimming_pool = Vec::new();
    //let bound_uniform_buffers = vec![0; 36]; // 36 is the platform minimal requirement
    let bound_array_buffer = None;
    // let bound_element_array_buffer = 0;
    // let bound_draw_framebuffer = Self::get_ctx_bound_draw_framebuffer(ctx)?;
    // let bound_vertex_array = Self::get_ctx_bound_vertex_array(ctx)?;
    // let current_program = Self::get_ctx_current_program(ctx)?;

    Ok(WebGL2State {
      _phantom: PhantomData,
      ctx,
      binding_stack,
      // viewport,
      // clear_color,
      // blending_state,
      // blending_equation,
      // blending_func,
      // depth_test,
      // depth_test_comparison,
      // face_culling_state,
      // face_culling_order,
      // face_culling_mode,
      // vertex_restart,
      // patch_vertex_nb,
      // current_texture_unit,
      // bound_textures,
      // texture_swimming_pool,
      // bound_uniform_buffers,
      bound_array_buffer,
      // bound_element_array_buffer,
      // bound_draw_framebuffer,
      // bound_vertex_array,
      // current_program,
    })
  }

  // fn get_ctx_viewport(ctx: &WebGl2RenderingContext) -> Result<[GLint; 4], StateQueryError> {
  //   let viewport: Vec<_> = ctx
  //     .get_parameter(WebGl2RenderingContext::VIEWPORT)
  //     .try_into()
  //     .map_err(|_| StateQueryError::UnknownViewportInitialState)?;

  //   if viewport.len() != 4 {
  //     return Err(StateQueryError::UnknownViewportInitialState);
  //   }

  //   Ok([viewport[0], viewport[1], viewport[2], viewport[3]])
  // }

  // fn get_ctx_clear_color(ctx: &WebGl2RenderingContext) -> Result<[GLfloat; 4], StateQueryError> {
  //   let color: Vec<f64> = ctx
  //     .get_parameter(WebGl2RenderingContext::COLOR_CLEAR_VALUE)
  //     .try_into()
  //     .map_err(|_| StateQueryError::UnknownClearColorInitialState)?;

  //   if color.len() != 4 {
  //     return Err(StateQueryError::UnknownClearColorInitialState);
  //   }

  //   Ok([color[0] as _, color[1] as _, color[2] as _, color[3] as _])
  // }

  // fn get_ctx_blending_state(
  //   ctx: &WebGl2RenderingContext,
  // ) -> Result<BlendingState, StateQueryError> {
  //   let enabled = ctx.is_enabled(WebGl2RenderingContext::BLEND);

  //   let state = if enabled {
  //     BlendingState::On
  //   } else {
  //     BlendingState::Off
  //   };

  //   Ok(state)
  // }

  // fn get_ctx_blending_equation(ctx: &WebGl2RenderingContext) -> Result<Equation, StateQueryError> {
  //   let data: GLenum = ctx
  //     .get_parameter(WebGl2RenderingContext::BLEND_EQUATION_RGB)
  //     .try_into()
  //     .unwrap();

  //   match data {
  //     WebGl2RenderingContext::FUNC_ADD => Ok(Equation::Additive),
  //     WebGl2RenderingContext::FUNC_SUBTRACT => Ok(Equation::Subtract),
  //     WebGl2RenderingContext::FUNC_REVERSE_SUBTRACT => Ok(Equation::ReverseSubtract),
  //     WebGl2RenderingContext::MIN => Ok(Equation::Min),
  //     WebGl2RenderingContext::MAX => Ok(Equation::Max),
  //     _ => Err(StateQueryError::UnknownBlendingEquation(data)),
  //   }
  // }

  // fn get_ctx_blending_factors(
  //   ctx: &WebGl2RenderingContext,
  // ) -> Result<(Factor, Factor), StateQueryError> {
  //   let src: GLint = ctx
  //     .get_parameter(WebGl2RenderingContext::BLEND_SRC_RGB)
  //     .try_into()
  //     .unwrap();
  //   let dst: GLint = ctx
  //     .get_parameter(WebGl2RenderingContext::BLEND_DST_RGB)
  //     .try_into()
  //     .unwrap();

  //   let src_k = Self::from_gl_blending_factor(src as GLenum)
  //     .map_err(StateQueryError::UnknownBlendingSrcFactor)?;
  //   let dst_k = Self::from_gl_blending_factor(dst as GLenum)
  //     .map_err(StateQueryError::UnknownBlendingDstFactor)?;

  //   Ok((src_k, dst_k))
  // }

  // #[inline]
  // fn from_gl_blending_factor(factor: GLenum) -> Result<Factor, GLenum> {
  //   match factor {
  //     WebGl2RenderingContext::ONE => Ok(Factor::One),
  //     WebGl2RenderingContext::ZERO => Ok(Factor::Zero),
  //     WebGl2RenderingContext::SRC_COLOR => Ok(Factor::SrcColor),
  //     WebGl2RenderingContext::ONE_MINUS_SRC_COLOR => Ok(Factor::SrcColorComplement),
  //     WebGl2RenderingContext::DST_COLOR => Ok(Factor::DestColor),
  //     WebGl2RenderingContext::ONE_MINUS_DST_COLOR => Ok(Factor::DestColorComplement),
  //     WebGl2RenderingContext::SRC_ALPHA => Ok(Factor::SrcAlpha),
  //     WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA => Ok(Factor::SrcAlphaComplement),
  //     WebGl2RenderingContext::DST_ALPHA => Ok(Factor::DstAlpha),
  //     WebGl2RenderingContext::ONE_MINUS_DST_ALPHA => Ok(Factor::DstAlphaComplement),
  //     WebGl2RenderingContext::SRC_ALPHA_SATURATE => Ok(Factor::SrcAlphaSaturate),
  //     _ => Err(factor),
  //   }
  // }

  // fn get_ctx_depth_test(ctx: &WebGl2RenderingContext) -> Result<DepthTest, StateQueryError> {
  //   let enabled = ctx.is_enabled(WebGl2RenderingContext::DEPTH_TEST);

  //   let test = if enabled {
  //     DepthTest::On
  //   } else {
  //     DepthTest::Off
  //   };

  //   Ok(test)
  // }

  // fn get_ctx_face_culling_state(
  //   ctx: &WebGl2RenderingContext,
  // ) -> Result<FaceCullingState, StateQueryError> {
  //   let enabled = ctx.is_enabled(WebGl2RenderingContext::CULL_FACE);

  //   let state = if enabled {
  //     FaceCullingState::On
  //   } else {
  //     FaceCullingState::Off
  //   };

  //   Ok(state)
  // }

  // fn get_ctx_face_culling_order(
  //   ctx: &WebGl2RenderingContext,
  // ) -> Result<FaceCullingOrder, StateQueryError> {
  //   let order: GLenum = ctx
  //     .get_parameter(WebGl2RenderingContext::FRONT_FACE)
  //     .try_into()
  //     .unwrap();

  //   match order {
  //     WebGl2RenderingContext::CCW => Ok(FaceCullingOrder::CCW),
  //     WebGl2RenderingContext::CW => Ok(FaceCullingOrder::CW),
  //     _ => Err(StateQueryError::UnknownFaceCullingOrder(order)),
  //   }
  // }

  // fn get_ctx_face_culling_mode(
  //   ctx: &WebGl2RenderingContext,
  // ) -> Result<FaceCullingMode, StateQueryError> {
  //   let mode: GLenum = ctx
  //     .get_parameter(WebGl2RenderingContext::CULL_FACE_MODE)
  //     .try_into()
  //     .unwrap();

  //   match mode {
  //     WebGl2RenderingContext::FRONT => Ok(FaceCullingMode::Front),
  //     WebGl2RenderingContext::BACK => Ok(FaceCullingMode::Back),
  //     WebGl2RenderingContext::FRONT_AND_BACK => Ok(FaceCullingMode::Both),
  //     _ => Err(StateQueryError::UnknownFaceCullingMode(mode)),
  //   }
  // }

  // fn get_ctx_vertex_restart(_: &WebGl2RenderingContext) -> Result<VertexRestart, StateQueryError> {
  //   // implementation note: WebGL2 doesn’t allow to enable nor disable primitive restart as it’s
  //   // always on
  //   Ok(VertexRestart::On)
  // }

  // fn get_ctx_current_texture_unit(ctx: &WebGl2RenderingContext) -> Result<GLenum, StateQueryError> {
  //   let active_texture = ctx
  //     .get_parameter(WebGl2RenderingContext::TEXTURE0)
  //     .try_into()
  //     .unwrap();
  //   Ok(active_texture)
  // }

  // fn get_ctx_bound_draw_framebuffer(
  //   ctx: &WebGl2RenderingContext,
  // ) -> Result<GLuint, StateQueryError> {
  //   let bound = ctx
  //     .get_parameter(WebGl2RenderingContext::DRAW_FRAMEBUFFER_BINDING)
  //     .try_into()
  //     .unwrap();
  //   Ok(bound)
  // }

  fn get_ctx_bound_vertex_array(
    ctx: &WebGl2RenderingContext,
  ) -> Result<WebGlBuffer, StateQueryError> {
    ctx
      .get_parameter(WebGl2RenderingContext::VERTEX_ARRAY_BINDING)
      .map_err(|_| StateQueryError::UnknownArrayBufferInitialState)?
      .try_into()
      .map_err(|_| StateQueryError::UnknownArrayBufferInitialState)
  }

  // fn get_ctx_current_program(ctx: &WebGl2RenderingContext) -> Result<GLuint, StateQueryError> {
  //   let used = ctx
  //     .get_parameter(WebGl2RenderingContext::CURRENT_PROGRAM)
  //     .try_into()
  //     .unwrap();
  //   Ok(used)
  // }

  pub(crate) fn create_buffer(&mut self) -> Option<WebGlBuffer> {
    self.ctx.create_buffer()
  }

  pub(crate) fn bind_array_buffer(&mut self, buffer: Option<&WebGlBuffer>, bind: Bind) {
    if bind == Bind::Forced || self.bound_array_buffer.as_ref() != buffer {
      self
        .ctx
        .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, buffer);
      self.bound_array_buffer = buffer.cloned();
    }
  }

  pub(crate) fn unbind_buffer(&mut self, buffer: &WebGlBuffer) {
    if self.bound_array_buffer.as_ref() == Some(buffer) {
      self.bind_array_buffer(None, Bind::Cached);
    }
    // FIXME: enable this as soon as we add either element buffers or vertex array buffers
    // else if self.bound_element_array_buffer == handle {
    //   self.bind_element_array_buffer(0, Bind::Cached);
    // } else if let Some(handle_) = self
    //   .bound_uniform_buffers
    //   .iter_mut()
    //   .find(|h| **h == handle)
    // {
    //   *handle_ = 0;
    // }
  }
}

/// An error that might happen when the context is queried.
#[derive(Debug)]
pub enum StateQueryError {
  /// The [`GLState`] object is unavailable.
  ///
  /// That might occur if the current thread doesn’t support allocating a new graphics state. It
  /// might happen if you try to have more than one state on the same thread, for instance.
  UnavailableGLState,
  /// Unknown array buffer initial state.
  UnknownArrayBufferInitialState,
  // /// Unknown viewport initial state.
  // UnknownViewportInitialState,
  // /// Unknown clear color initial state.
  // UnknownClearColorInitialState,
  // /// Corrupted blending state.
  // UnknownBlendingState(GLboolean),
  // /// Corrupted blending equation.
  // UnknownBlendingEquation(GLenum),
  // /// Corrupted blending source factor.
  // UnknownBlendingSrcFactor(GLenum),
  // /// Corrupted blending destination factor.
  // UnknownBlendingDstFactor(GLenum),
  // /// Corrupted depth test state.
  // UnknownDepthTestState(GLboolean),
  // /// Corrupted face culling state.
  // UnknownFaceCullingState(GLboolean),
  // /// Corrupted face culling order.
  // UnknownFaceCullingOrder(GLenum),
  // /// Corrupted face culling mode.
  // UnknownFaceCullingMode(GLenum),
  // /// Corrupted vertex restart state.
  // UnknownVertexRestartState(GLboolean),
}

impl fmt::Display for StateQueryError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      StateQueryError::UnavailableGLState => write!(f, "unavailable graphics state"),
      StateQueryError::UnknownArrayBufferInitialState => {
        write!(f, "unknown array buffer initial state")
      } // StateQueryError::UnknownViewportInitialState => write!(f, "unknown viewport initial state"),
        // StateQueryError::UnknownClearColorInitialState => {
        //   write!(f, "unknown clear color initial state")
        // }
        // StateQueryError::UnknownBlendingState(ref s) => write!(f, "unknown blending state: {}", s),
        // StateQueryError::UnknownBlendingEquation(ref e) => {
        //   write!(f, "unknown blending equation: {}", e)
        // }
        // StateQueryError::UnknownBlendingSrcFactor(ref k) => {
        //   write!(f, "unknown blending source factor: {}", k)
        // }
        // StateQueryError::UnknownBlendingDstFactor(ref k) => {
        //   write!(f, "unknown blending destination factor: {}", k)
        // }
        // StateQueryError::UnknownDepthTestState(ref s) => write!(f, "unknown depth test state: {}", s),
        // StateQueryError::UnknownFaceCullingState(ref s) => {
        //   write!(f, "unknown face culling state: {}", s)
        // }
        // StateQueryError::UnknownFaceCullingOrder(ref o) => {
        //   write!(f, "unknown face culling order: {}", o)
        // }
        // StateQueryError::UnknownFaceCullingMode(ref m) => {
        //   write!(f, "unknown face culling mode: {}", m)
        // }
        // StateQueryError::UnknownVertexRestartState(ref s) => {
        //   write!(f, "unknown vertex restart state: {}", s)
        // }
    }
  }
}

/// Should the binding be cached or forced to the provided value?
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum Bind {
  Forced,
  Cached,
}

/// Whether or not enable blending.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum BlendingState {
  /// Enable blending.
  On,
  /// Disable blending.
  Off,
}

/// Whether or not depth test should be enabled.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum DepthTest {
  /// The depth test is enabled.
  On,
  /// The depth test is disabled.
  Off,
}

/// Should face culling be enabled?
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum FaceCullingState {
  /// Enable face culling.
  On,
  /// Disable face culling.
  Off,
}

/// Whether or not vertex restart is enabled.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum VertexRestart {
  /// Vertex restart is enabled.
  On,
  /// Vertex restart is disabled.
  Off,
}

//pub(crate) fn depth_comparison_to_glenum(dc: DepthComparison) -> GLenum {
//  match dc {
//    DepthComparison::Never => WebGl2RenderingContext::NEVER,
//    DepthComparison::Always => WebGl2RenderingContext::ALWAYS,
//    DepthComparison::Equal => WebGl2RenderingContext::EQUAL,
//    DepthComparison::NotEqual => WebGl2RenderingContext::NOTEQUAL,
//    DepthComparison::Less => WebGl2RenderingContext::LESS,
//    DepthComparison::LessOrEqual => WebGl2RenderingContext::LEQUAL,
//    DepthComparison::Greater => WebGl2RenderingContext::GREATER,
//    DepthComparison::GreaterOrEqual => WebGl2RenderingContext::GEQUAL,
//  }
//}
