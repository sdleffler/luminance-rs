//! Graphics state.

use std::cell::RefCell;
use std::fmt;
use std::marker::PhantomData;
use web_sys::{
  WebGl2RenderingContext, WebGlBuffer, WebGlFramebuffer, WebGlTexture, WebGlVertexArrayObject,
};

// TLS synchronization barrier for `GLState`.
thread_local!(static TLS_ACQUIRE_GFX_STATE: RefCell<Option<()>> = RefCell::new(Some(())));

#[derive(Debug)]
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
#[derive(Debug)]
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

  // // patch primitive vertex number
  // patch_vertex_nb: usize,

  // texture
  current_texture_unit: usize,
  bound_textures: Vec<(u32, Option<WebGlTexture>)>,

  // texture buffer used to optimize texture creation; regular textures typically will never ask
  // for fetching from this set but framebuffers, who often generate several textures, might use
  // this opportunity to get N textures (color, depth and stencil) at once, in a single CPU / GPU
  // roundtrip
  //
  // fishy fishy
  texture_swimming_pool: Vec<Option<WebGlTexture>>,

  // // uniform buffer
  // bound_uniform_buffers: Vec<GLuint>,

  // array buffer
  bound_array_buffer: Option<WebGlBuffer>,
  // element buffer
  bound_element_array_buffer: Option<WebGlBuffer>,

  // framebuffer
  bound_draw_framebuffer: Option<WebGlFramebuffer>,
  bound_read_framebuffer: Option<WebGlFramebuffer>,

  // A special framebuffer used to read textures (workaround the fact WebGL2 doesn’t have
  // support of glGetTexImage). That object will never be created until trying to read a
  // texture’s image.
  readback_framebuffer: Option<WebGlFramebuffer>,

  // vertex array
  bound_vertex_array: Option<WebGlVertexArrayObject>,
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
    //let patch_vertex_nb = 0;
    let current_texture_unit = 0;
    let bound_textures = vec![(WebGl2RenderingContext::TEXTURE0, None); 48]; // 48 is the platform minimal requirement
    let texture_swimming_pool = Vec::new();
    //let bound_uniform_buffers = vec![0; 36]; // 36 is the platform minimal requirement
    let bound_array_buffer = None;
    let bound_element_array_buffer = None;
    let bound_draw_framebuffer = None;
    let bound_read_framebuffer = None;
    let readback_framebuffer = None;
    let bound_vertex_array = None;
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
      // patch_vertex_nb,
      current_texture_unit,
      bound_textures,
      texture_swimming_pool,
      // bound_uniform_buffers,
      bound_array_buffer,
      bound_element_array_buffer,
      bound_draw_framebuffer,
      bound_read_framebuffer,
      readback_framebuffer,
      bound_vertex_array,
      // current_program,
    })
  }

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

  pub(crate) fn bind_element_array_buffer(&mut self, buffer: Option<&WebGlBuffer>, bind: Bind) {
    if bind == Bind::Forced || self.bound_element_array_buffer.as_ref() != buffer {
      self
        .ctx
        .bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, buffer);
      self.bound_element_array_buffer = buffer.cloned();
    }
  }

  pub(crate) fn unbind_buffer(&mut self, buffer: &WebGlBuffer) {
    if self.bound_array_buffer.as_ref() == Some(buffer) {
      self.bind_array_buffer(None, Bind::Cached);
    } else if self.bound_element_array_buffer.as_ref() == Some(buffer) {
      self.bind_element_array_buffer(None, Bind::Cached);
    }
    // FIXME: enable this as soon as we add uniform buffers
    // } else if let Some(handle_) = self
    //   .bound_uniform_buffers
    //   .iter_mut()
    //   .find(|h| **h == handle)
    // {
    //   *handle_ = 0;
    // }
  }

  pub(crate) fn create_vertex_array(&mut self) -> Option<WebGlVertexArrayObject> {
    self.ctx.create_vertex_array()
  }

  pub(crate) fn bind_vertex_array(&mut self, vao: Option<&WebGlVertexArrayObject>, bind: Bind) {
    if bind == Bind::Forced || self.bound_vertex_array.as_ref() != vao {
      self.ctx.bind_vertex_array(vao);
      self.bound_vertex_array = vao.cloned();
    }
  }

  pub(crate) fn create_texture(&mut self) -> Option<WebGlTexture> {
    self
      .texture_swimming_pool
      .pop()
      .flatten()
      .or_else(|| self.ctx.create_texture())
  }

  /// Reserve at least a given number of textures.
  pub(crate) fn reserve_textures(&mut self, nb: usize) {
    let available = self.texture_swimming_pool.len();
    let needed = nb.max(available) - available;

    if needed > 0 {
      // resize the internal buffer to hold all the new textures and create a slice starting from
      // the previous end to the new end
      self.texture_swimming_pool.resize(available + needed, None);

      for _ in 0..needed {
        match self.ctx.create_texture() {
          Some(texture) => self.texture_swimming_pool.push(Some(texture)),
          None => break,
        }
      }
    }
  }

  pub(crate) fn bind_texture(&mut self, target: u32, handle: Option<&WebGlTexture>) {
    let unit = self.current_texture_unit;

    match self.bound_textures.get(unit) {
      Some((t, ref h)) if target != *t || handle != h.as_ref() => {
        self.ctx.bind_texture(target, handle);
        self.bound_textures[unit] = (target, handle.cloned());
      }

      None => {
        self.ctx.bind_texture(target, handle);

        // not enough available texture units; let’s grow a bit more
        self
          .bound_textures
          .resize(unit + 1, (WebGl2RenderingContext::TEXTURE_2D, None));
        self.bound_textures[unit] = (target, handle.cloned());
      }

      _ => (), // cached
    }
  }

  pub(crate) fn create_framebuffer(&mut self) -> Option<WebGlFramebuffer> {
    self.ctx.create_framebuffer()
  }

  pub(crate) fn create_or_get_readback_framebuffer(&mut self) -> Option<WebGlFramebuffer> {
    self.readback_framebuffer.clone().or_else(|| {
      // create the readback framebuffer if not already created
      self.readback_framebuffer = self.create_framebuffer();
      self.readback_framebuffer.clone()
    })
  }

  pub(crate) fn bind_draw_framebuffer(&mut self, handle: Option<&WebGlFramebuffer>) {
    if self.bound_draw_framebuffer.as_ref() != handle {
      self
        .ctx
        .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, handle);
      self.bound_draw_framebuffer = handle.cloned();
    }
  }

  pub(crate) fn bind_read_framebuffer(&mut self, handle: Option<&WebGlFramebuffer>) {
    if self.bound_read_framebuffer.as_ref() != handle {
      self
        .ctx
        .bind_framebuffer(WebGl2RenderingContext::READ_FRAMEBUFFER, handle);
      self.bound_read_framebuffer = handle.cloned();
    }
  }
}

impl Drop for WebGL2State {
  fn drop(&mut self) {
    // drop the readback framebuffer if it was allocated
    self
      .ctx
      .delete_framebuffer(self.readback_framebuffer.as_ref());
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
