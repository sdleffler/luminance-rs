//! Graphics pipelines.
//!
//! Graphics pipelines are the means used to describe — and hence perform — renders. They
//! provide a way to describe how resources should be shared and used to produce a single
//! pixel frame.
//!
//! # Pipelines and AST
//!
//! luminance has a very particular way of doing graphics. It represents a typical _graphics
//! pipeline_ via a typed [AST] that is embedded into your code. As you might already know, when you
//! write code, you’re actually creating an [AST]: expressions, assignments, bindings, conditions,
//! function calls, etc. They all represent a typed tree that represents your program.
//!
//! luminance uses that property to create a dependency between resources your GPU needs to
//! have in order to perform a render. It might be weird at first but you’ll see how simple and easy
//! it is. If you want to perform a simple draw call of a triangle, you need several resources:
//!
//! - A [`Tess`] that represents the triangle. It holds three vertices.
//! - A shader [`Program`], for shading the triangle with a constant color, for short and simple.
//! - A [`Framebuffer`], to accept and hold the actual render.
//! - A [`RenderState`], to state how the render should be performed.
//! - And finally, a [`PipelineState`], which allows even more customization on how the pipeline
//!   behaves
//!
//! There is a dependency _graph_ to represent how the resources must behave regarding each other:
//!
//! ```text
//! (AST1)
//!
//! PipelineState ─── Framebuffer ─── Shader ─── RenderState ─── Tess
//! ```
//!
//! The framebuffer must be _active_, _bound_, _used_ — or whatever verb you want to picture it
//! with — before the shader can start doing things. The shader must also be in use before we can
//! actually render the tessellation.
//!
//! That triple dependency relationship is already a small flat [AST]. Imagine we want to render
//! a second triangle with the same render state and a third triangle with a different render state:
//!
//! ```text
//! (AST2)
//!
//! PipelineState ─── Framebuffer ─── Shader ─┬─ RenderState ─┬─ Tess
//!                                           │               │
//!                                           │               └─ Tess
//!                                           │
//!                                           └─ RenderState ─── Tess
//! ```
//!
//! That [AST] looks more complex. Imagine now that we want to shade one other triangle with
//! another shader!
//!
//! ```text
//! (AST3)
//!
//! PipelineState ─── Framebuffer ─┬─ Shader ─┬─ RenderState ─┬─ Tess
//!                                │          │               │
//!                                │          │               └─ Tess
//!                                │          │
//!                                │          └─ RenderState ─── Tess
//!                                │
//!                                └─ Shader ─── RenderState ─── Tess
//! ```
//!
//! You can now clearly see the [AST]s and the relationships between objects. Those are encoded
//! in luminance within your code directly: lambdas / closures.
//!
//! > If you have followed thoroughly, you might have noticed that you cannot, with such [AST]s,
//! > shade a triangle with another shader but using the same render state as another node. That
//! > was a decision that was needed to be made: how should we allow the [AST] to be shared?
//! > In terms of graphics pipeline, luminance tries to do the best thing to minimize the number
//! > of GPU context switches and CPU <=> GPU bandwidth congestion.
//!
//! # The lambda & closure design
//!
//! A function is a perfect candidate to modelize a dependency: the arguments of the function
//! modelize the dependency — they will be provided _at some point in time_, but it doesn’t matter
//! when while writing the function. We can then write code _depending_ on something without even
//! knowing where it’s from.
//!
//! Using pseudo-code, here’s what the ASTs from above look like: (this is not a real luminance,
//! excerpt, just a simplification).
//!
//! ```ignore
//! // AST1
//! pipeline(framebuffer, pipeline_state, || {
//!   // here, we are passing a closure that will get called whenever the framebuffer is ready to
//!   // receive renders
//!   use_shader(shader, || {
//!     // same thing but for shader
//!     use_render_state(render_state, || {
//!       // ditto for render state
//!       triangle.render(); // render the tessellation
//!     });
//!   );
//! );
//! ```
//!
//! See how simple it is to represent `AST1` with just closures? Rust’s lifetimes and existential
//! quantification allow us to ensure that no resource will leak from the scope of each closures,
//! hence enforcing memory and coherency safety.
//!
//! Now let’s try to tackle `AST2`.
//!
//! ```ignore
//! // AST2
//! pipeline(framebuffer, pipeline_state, || {
//!   use_shader(shader, || {
//!     use_render_state(render_state, || {
//!       first_triangle.render();
//!       second_triangle.render(); // simple and straight-forward
//!     });
//!
//!     // we can just branch a new render state here!
//!     use_render_state(other_render_state, || {
//!       third.render()
//!     });
//!   );
//! );
//! ```
//!
//! And `AST3`:
//!
//! ```ignore
//! // AST3
//! pipeline(framebuffer, pipeline_state, || {
//!   use_shader(shader, || {
//!     use_render_state(render_state, || {
//!       first_triangle.render();
//!       second_triangle.render(); // simple and straight-forward
//!     });
//!
//!     // we can just branch a new render state here!
//!     use_render_state(other_render_state, || {
//!       third.render()
//!     });
//!   );
//!
//!   use_shader(other_shader, || {
//!     use_render_state(yet_another_render_state, || {
//!       other_triangle.render();
//!     });
//!   });
//! );
//! ```
//!
//! The luminance equivalent is a bit more complex because it implies some objects that need
//! to be introduced first.
//!
//! # PipelineGate and Pipeline
//!
//! A [`PipelineGate`] represents a whole [AST] as seen as just above. It is created by a
//! [`GraphicsContext`] when you ask to create a pipeline gate. A [`PipelineGate`] is typically
//! destroyed at the end of the current frame, but that’s not a general rule.
//!
//! Such an object gives you access, via the [`PipelineGate::pipeline`], to two other objects
//! :
//!
//! - A [`ShadingGate`], explained below.
//! - A [`Pipeline`].
//!
//! A [`Pipeline`] is a special object you can use to use some specific scarce resources, such as
//! _textures_ and _buffers_. Those are treated a bit specifically on the GPU, so you have to use
//! the [`Pipeline`] interface to deal with them.
//!
//! Creating a [`PipelineGate`] requires two resources: a [`Framebuffer`] to render to, and a
//! [`PipelineState`], allowing to customize how the pipeline will perform renders at runtime.
//!
//! # ShadingGate
//!
//! When you create a pipeline, you’re also handed a [`ShadingGate`]. A [`ShadingGate`] is an object
//! that allows you to create _shader_ nodes in the [AST] you’re building. You have no other way
//! to go deeper in the [AST].
//!
//! That node will typically borrow a shader [`Program`] and will move you one level lower in the
//! graph ([AST]). A shader [`Program`] is typically an object you create at initialization or at
//! specific moment in time (i.e. you don’t create them each frame) that tells the GPU how vertices
//! should be transformed; how primitives should be moved and generated, how tessellation occurs and
//! how fragment (i.e. pixels) are computed / shaded — hence the name.
//!
//! At that level (i.e. in that closure), you are given two objects:
//!
//!   - A [`RenderGate`], discussed below.
//!   - A [`ProgramInterface`], which has as type parameter the type of uniform your shader
//!     [`Program`] defines.
//!
//! The [`ProgramInterface`] is the only way for you to access your _uniform interface_. More on
//! this in the dedicated section. It also provides you with the [`ProgramInterface::query`]
//! method, that allows you to perform _dynamic uniform lookup_.
//!
//! # RenderGate
//!
//! A [`RenderGate`] is the second to last gate you will be handling. It allows you to create
//! _render state_ nodes in your [AST], creating a new level for you to render tessellations with
//! an obvious, final gate: the [`TessGate`].
//!
//! The kind of object that node manipulates is [`RenderState`]. A [`RenderState`] — a bit like for
//! [`PipelineGate`] with [`PipelineState`] — enables to customize how a render of a specific set
//! of objects (i.e. tessellations) will occur. It’s a bit more specific to renders than pipelines.
//!
//! # TessGate
//!
//! The [`TessGate`] is the final gate you use in an [AST]. It’s used to create _tessellation
//! nodes_. Those are used to render actual [`Tess`]. You cannot go any deeper in the [AST] at that
//! stage.
//!
//! [`TessGate`]s don’t immediately use [`Tess`] as inputs. They use [`TessView`]. That type is
//! a simple GPU view into a GPU tessellation ([`Tess`]). It can be obtained from a [`Tess`] via
//! the [`SubTess`] trait or built explicitly.
//!
//! [AST]: https://en.wikipedia.org/wiki/Abstract_syntax_tree
//! [`Tess`]: crate::tess::Tess
//! [`Program`]: crate::shader::Program
//! [`Framebuffer`]: crate::framebuffer::Framebuffer
//! [`RenderState`]: crate::render_state::RenderState
//! [`PipelineState`]: crate::pipeline::PipelineState
//! [`ShadingGate`]: crate::shading_gate::ShadingGate
//! [`RenderGate`]: crate::render_gate::RenderGate
//! [`ProgramInterface`]: crate::shader::ProgramInterface
//! [`ProgramInterface::query`]: crate::shader::ProgramInterface::query
//! [`TessGate`]: crate::tess_gate::TessGate
//! [`TessView`]: crate::tess::TessView
//! [`SubTess`]: crate::tess::SubTess

use crate::backend::color_slot::ColorSlot;
use crate::backend::depth_slot::DepthSlot;
use crate::backend::framebuffer::Framebuffer as FramebufferBackend;
use crate::backend::pipeline::{
  Pipeline as PipelineBackend, PipelineBase, PipelineBuffer, PipelineTexture,
};
use crate::buffer::Buffer;
use crate::context::GraphicsContext;
use crate::framebuffer::Framebuffer;
use crate::pixel::Pixel;
use crate::shading_gate::ShadingGate;
use crate::texture::Dimensionable;
use crate::texture::Texture;

use std::fmt;
use std::marker::PhantomData;

/// Possible errors that might occur in a graphics [`Pipeline`].
#[non_exhaustive]
#[derive(Debug)]
pub enum PipelineError {}

impl fmt::Display for PipelineError {
  fn fmt(&self, _: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    Ok(())
  }
}

/// The viewport being part of the [`PipelineState`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Viewport {
  /// The whole viewport is used. The position and dimension of the viewport rectangle are
  /// extracted from the framebuffer.
  Whole,
  /// The viewport is specific and the rectangle area is user-defined.
  Specific {
    /// The lower position on the X axis to start the viewport rectangle at.
    x: u32,
    /// The lower position on the Y axis to start the viewport rectangle at.
    y: u32,
    /// The width of the viewport.
    width: u32,
    /// The height of the viewport.
    height: u32,
  },
}

/// Various customization options for pipelines.
#[derive(Clone, Debug)]
pub struct PipelineState {
  pub clear_color: [f32; 4],
  pub clear_color_enabled: bool,
  pub clear_depth_enabled: bool,
  pub viewport: Viewport,
  pub srgb_enabled: bool,
}

impl Default for PipelineState {
  /// Default [`PipelineState`]:
  ///
  /// - Clear color: `[0, 0, 0, 1]`.
  /// - Color is always cleared.
  /// - Depth is always cleared.
  /// - The viewport uses the whole framebuffer’s.
  /// - sRGB encoding is disabled.
  fn default() -> Self {
    PipelineState {
      clear_color: [0., 0., 0., 1.],
      clear_color_enabled: true,
      clear_depth_enabled: true,
      viewport: Viewport::Whole,
      srgb_enabled: false,
    }
  }
}

impl PipelineState {
  /// Create a default [`PipelineState`].
  ///
  /// See the documentation of the [`Default`] for further details.
  pub fn new() -> Self {
    Self::default()
  }

  /// Get the clear color.
  pub fn clear_color(&self) -> [f32; 4] {
    self.clear_color
  }

  /// Set the clear color.
  pub fn set_clear_color(self, clear_color: [f32; 4]) -> Self {
    Self {
      clear_color,
      ..self
    }
  }

  /// Check whether the pipeline’s framebuffer’s color buffers will be cleared.
  pub fn is_clear_color_enabled(&self) -> bool {
    self.clear_color_enabled
  }

  /// Enable clearing color buffers.
  pub fn enable_clear_color(self, clear_color_enabled: bool) -> Self {
    Self {
      clear_color_enabled,
      ..self
    }
  }

  /// Check whether the pipeline’s framebuffer’s depth buffer will be cleared.
  pub fn is_clear_depth_enabled(&self) -> bool {
    self.clear_depth_enabled
  }

  /// Enable clearing depth buffers.
  pub fn enable_clear_depth(self, clear_depth_enabled: bool) -> Self {
    Self {
      clear_depth_enabled,
      ..self
    }
  }

  /// Get the viewport.
  pub fn viewport(&self) -> Viewport {
    self.viewport
  }

  /// Set the viewport.
  pub fn set_viewport(self, viewport: Viewport) -> Self {
    Self { viewport, ..self }
  }

  /// Check whether sRGB linearization is enabled.
  pub fn is_srgb_enabled(&self) -> bool {
    self.srgb_enabled
  }

  /// Enable sRGB linearization.
  pub fn enable_srgb(self, srgb_enabled: bool) -> Self {
    Self {
      srgb_enabled,
      ..self
    }
  }
}

/// A GPU pipeline handle.
///
/// A [`Pipeline`] is a special object that is provided as soon as one enters a [`PipelineGate`].
/// It is used to dynamically modify the behavior of the running graphics pipeline. That includes,
/// for instance, obtaining _bound resources_, like buffers and textures, for subsequent uses in
/// shader stages.
///
/// # Parametricity
///
/// - `B` is the backend type. It must implement [`PipelineBase`].
pub struct Pipeline<'a, B>
where
  B: ?Sized + PipelineBase,
{
  repr: B::PipelineRepr,
  _phantom: PhantomData<&'a mut ()>,
}

impl<'a, B> Pipeline<'a, B>
where
  B: PipelineBase,
{
  /// Bind a buffer.
  ///
  /// Once the buffer is bound, the [`BoundBuffer`] object has to be dropped / die in order to
  /// bind the buffer again.
  pub fn bind_buffer<T>(
    &'a self,
    buffer: &'a mut Buffer<B, T>,
  ) -> Result<BoundBuffer<'a, B, T>, PipelineError>
  where
    B: PipelineBuffer<T>,
  {
    unsafe {
      B::bind_buffer(&self.repr, &buffer.repr).map(|repr| BoundBuffer {
        repr,
        _phantom: PhantomData,
      })
    }
  }

  /// Bind a texture.
  ///
  /// Once the texture is bound, the [`BoundTexture`] object has to be dropped / die in order to
  /// bind the texture again.
  pub fn bind_texture<D, P>(
    &'a self,
    texture: &'a mut Texture<B, D, P>,
  ) -> Result<BoundTexture<'a, B, D, P>, PipelineError>
  where
    B: PipelineTexture<D, P>,
    D: Dimensionable,
    P: Pixel,
  {
    unsafe {
      B::bind_texture(&self.repr, &texture.repr).map(|repr| BoundTexture {
        repr,
        _phantom: PhantomData,
      })
    }
  }
}

/// Top-most node in a graphics pipeline.
///
/// [`PipelineGate`] nodes represent the “entry-points” of graphics pipeline. They are used
/// with a [`Framebuffer`] to render to and a [`PipelineState`] to customize the overall behavior
/// of the pipeline.
///
/// # Parametricity
///
/// - `C`, the graphics context type; must implement [`GraphicsContext`].
pub struct PipelineGate<'a, C>
where
  C: ?Sized + GraphicsContext,
{
  ctx: &'a mut C,
}

impl<'a, C> PipelineGate<'a, C>
where
  C: ?Sized + GraphicsContext,
{
  /// Create a new [`PipelineGate`].
  pub fn new(ctx: &'a mut C) -> Self {
    PipelineGate { ctx }
  }

  /// Enter a pipeline node.
  ///
  /// This method is the entry-point in a graphics pipeline. It takes a [`Framebuffer`] and a
  /// [`PipelineState`] and a closure that allows to go deeper in the pipeline (i.e. resource
  /// graph). The closure is passed a [`Pipeline`] for you to dynamically alter the pipeline and a
  /// [`ShadingGate`] to enter shading nodes.
  pub fn pipeline<D, CS, DS, F>(
    &mut self,
    framebuffer: &Framebuffer<C::Backend, D, CS, DS>,
    pipeline_state: &PipelineState,
    f: F,
  ) -> Result<(), PipelineError>
  where
    C::Backend: FramebufferBackend<D> + PipelineBackend<D>,
    D: Dimensionable,
    CS: ColorSlot<C::Backend, D>,
    DS: DepthSlot<C::Backend, D>,
    F: for<'b> FnOnce(Pipeline<'b, C::Backend>, ShadingGate<'b, C>),
  {
    unsafe {
      self
        .ctx
        .backend()
        .start_pipeline(&framebuffer.repr, pipeline_state);
    }

    let pipeline = unsafe {
      self.ctx.backend().new_pipeline().map(|repr| Pipeline {
        repr,
        _phantom: PhantomData,
      })?
    };
    let shading_gate = ShadingGate { ctx: self.ctx };

    f(pipeline, shading_gate);
    Ok(())
  }
}

/// Opaque buffer binding.
///
/// This type represents a bound [`Buffer`] via [`BoundBuffer`]. It can be used along with a
/// [`Uniform`] to customize a shader’s behavior.
///
/// # Parametricity
///
/// - `T` is the type of the carried item by the [`Buffer`].
///
/// # Notes
///
/// You shouldn’t try to do store / cache or do anything special with that value. Consider it
/// an opaque object.
///
/// [`Uniform`]: crate::shader::Uniform
#[derive(Debug)]
pub struct BufferBinding<T> {
  binding: u32,
  _phantom: PhantomData<*const T>,
}

impl<T> BufferBinding<T> {
  /// Access the underlying binding value.
  ///
  /// # Notes
  ///
  /// That value shouldn’t be read nor store, as it’s only meaningful for backend implementations.
  pub fn binding(self) -> u32 {
    self.binding
  }
}

/// A _bound_ [`Buffer`].
///
/// # Parametricity
///
/// - `B` is the backend type. It must implement [`PipelineBuffer`].
/// - `T` is the type of the carried item by the [`Buffer`].
///
/// # Notes
///
/// Once a [`Buffer`] is bound, it can be used and passed around to shaders. In order to do so,
/// you will need to pass a [`BufferBinding`] to your [`ProgramInterface`]. That value is unique
/// to each [`BoundBuffer`] and should always be asked — you shouldn’t cache them, for instance.
///
/// Getting a [`BufferBinding`] is a cheap operation and is performed via the
/// [`BoundBuffer::binding`] method.
///
/// [`ProgramInterface`]: crate::shader::ProgramInterface
pub struct BoundBuffer<'a, B, T>
where
  B: PipelineBuffer<T>,
{
  pub(crate) repr: B::BoundBufferRepr,
  _phantom: PhantomData<&'a T>,
}

impl<'a, B, T> BoundBuffer<'a, B, T>
where
  B: PipelineBuffer<T>,
{
  /// Obtain a [`BufferBinding`] object that can be used to refer to this bound buffer in shader
  /// stages.
  ///
  /// # Notes
  ///
  /// You shouldn’t try to do store / cache or do anything special with that value. Consider it
  /// an opaque object.
  pub fn binding(&self) -> BufferBinding<T> {
    let binding = unsafe { B::buffer_binding(&self.repr) };
    BufferBinding {
      binding,
      _phantom: PhantomData,
    }
  }
}

/// Opaque texture binding.
///
/// This type represents a bound [`Texture`] via [`BoundTexture`]. It can be used along with a
/// [`Uniform`] to customize a shader’s behavior.
///
/// # Parametricity
///
/// - `D` is the dimension of the original texture. It must implement [`Dimensionable`] in most
///   useful methods.
/// - `S` is the sampler type. It must implement [`SamplerType`] in most useful methods.
///
/// # Notes
///
/// You shouldn’t try to do store / cache or do anything special with that value. Consider it
/// an opaque object.
///
/// [`Uniform`]: crate::shader::Uniform
/// [`SamplerType`]: crate::pixel::SamplerType
pub struct TextureBinding<D, S> {
  binding: u32,
  _phantom: PhantomData<*const (D, S)>,
}

impl<D, S> TextureBinding<D, S> {
  /// Access the underlying binding value.
  ///
  /// # Notes
  ///
  /// That value shouldn’t be read nor store, as it’s only meaningful for backend implementations.
  pub fn binding(self) -> u32 {
    self.binding
  }
}

/// A _bound_ [`Texture`].
///
/// # Parametricity
///
/// - `B` is the backend type. It must implement [`PipelineTexture`].
/// - `D` is the dimension. It must implement [`Dimensionable`].
/// - `P` is the pixel type. It must implement [`Pixel`].
///
/// # Notes
///
/// Once a [`Texture`] is bound, it can be used and passed around to shaders. In order to do so,
/// you will need to pass a [`TextureBinding`] to your [`ProgramInterface`]. That value is unique
/// to each [`BoundTexture`] and should always be asked — you shouldn’t cache them, for instance.
///
/// Getting a [`TextureBinding`] is a cheap operation and is performed via the
/// [`BoundTexture::binding`] method.
///
/// [`ProgramInterface`]: crate::shader::ProgramInterface
pub struct BoundTexture<'a, B, D, P>
where
  B: PipelineTexture<D, P>,
  D: Dimensionable,
  P: Pixel,
{
  pub(crate) repr: B::BoundTextureRepr,
  _phantom: PhantomData<&'a ()>,
}

impl<'a, B, D, P> BoundTexture<'a, B, D, P>
where
  B: PipelineTexture<D, P>,
  D: Dimensionable,
  P: Pixel,
{
  /// Obtain a [`TextureBinding`] object that can be used to refer to this bound texture in shader
  /// stages.
  ///
  /// # Notes
  ///
  /// You shouldn’t try to do store / cache or do anything special with that value. Consider it
  /// an opaque object.
  pub fn binding(&self) -> TextureBinding<D, P::SamplerType> {
    let binding = unsafe { B::texture_binding(&self.repr) };
    TextureBinding {
      binding,
      _phantom: PhantomData,
    }
  }
}
