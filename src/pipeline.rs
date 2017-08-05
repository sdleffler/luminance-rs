//! Dynamic rendering pipelines.
//!
//! This module gives you types and functions to build *dynamic* rendering **pipelines**. A
//! `Pipeline` represents a functional stream that consumes geometric data and rasterizes them.
//!
//! # Key concepts
//!
//! luminance exposes several concepts you have to be familiar with:
//!
//! - render buffers;
//! - blending;
//! - shaders;
//! - tessellations;
//! - gates;
//! - render commands;
//! - shading commands;
//! - pipelines;
//!
//! # Render buffers
//!
//! The render buffers are GPU-allocated memory regions used while rendering images into
//! framebuffers. Typically, a framebuffer has at least three buffers:
//!
//! - a *color buffer*, that will receive texels (akin to pixels, but for textures/buffers);
//! - a *depth buffer*, a special buffer mostly used to determine whether a fragment (pixel) is
//!   behind something that was previously rendered – it’s a simple solution to discard render that
//!   won’t be visible anyway;
//! - a *stencil buffer*, which often acts as a mask to create interesting effects to your renders.
//!
//! luminance gives you access to the first two – the stencil buffer will be added in a future
//! release.
//!
//! Alternatively, you can also tell your GPU that you won’t be using a depth buffer, or that you
//! need several color buffers – this is called [MRT](https://en.wikipedia.org/wiki/Multiple_Render_Targets).
//! In most frameworks, you create some textures to hold the color information, then you “bind” them
//! to your pipeline, and you’re good to go. In luminance, everything happens at the type level: you
//! say to luminance which type of framebuffer you want (for instance, a framebuffer with two color
//! outputs and a depth buffer). luminance will handle the textures for you – you can retrieve access
//! to them whenever you want to.
//!
//! If you have a depth buffer, you can ask luminance to perform a depth test that will discard any
//! fragment being “behind” the fragment already in place. You can also give luminance the *clear
//! color* it must use when you issue a new pipeline to fill the buffers.
//!
//! # Blending
//!
//! When you render a fragment A at a position P in a framebuffer, there are several configurations:
//!
//! - you have a depth buffer and the depth test is enabled: in that case, no blending will happen
//!   as either the already in-place fragment will be chosen or the new one you try to write,
//!   depending on the result of the depth test;
//! - the depth test is disabled: in that case, each time a fragment is to be written to a place in
//!   a buffer, its output will be blended with the color already present according to a *blending
//!   equation* and two *blending factors*.
//!
//! # Shaders
//!
//! Shaders in luminance are pretty simple: you have `Stage` that represents each type of shader
//! stages you can use, and `Program`, that links them into a GPU executable.
//!
//! In luminance, you’re supposed to create a `Program` with `Stage`. Some of them are mandatory and
//! others are optional. In order to customize your build, you can use a *uniform interface*. The
//! uniform interface is defined by our own type. When you create a program, you pass code to tell
//! luminance how to get such a type. Then that type will be handed back to you when needed in the
//! form of an immutable object.
//!
//! # Tessellations
//!
//! The `Tess` type represents a tessellation associated with a possible set of GPU vertices and
//! indices. Note that it’s also possible to create *attributeless* tesselations – i.e.
//! tessellations that don’t own any vertices nor indices and are used with specific shaders only.
//!
//! # Gates
//!
//! The gate concept is quite easy to understand: picture a pipeline as tree. Each node is typed,
//! hence, the structure is quite limited to what your GPU can understand. Gates are a way to
//! spread information of root nodes down. For instance, if you have a shader node, every nested
//! child of that shader node will have the possibility to use its features via a shading gate.
//!
//! # Render commands
//!
//! A set of values that tag a collection of tessellations. Typical information is whether we should
//! use a depth test, the blending factors and equation, etc.
//!
//! # Shading commands
//!
//! A set of values that tag a collection of render commands. Typical information is a shader
//! program along with its uniform interface.
//!
//! # Pipelines
//!
//! A pipeline is just an aggregation of shadings commands with a few extra information.

use gl;
use gl::types::*;

use buffer::RawBuffer;
use blending::{Equation, Factor};
use framebuffer::{ColorSlot, DepthSlot, Framebuffer};
use shader::program::{Program, UniformInterface};
use std::marker::PhantomData;
use tess::TessRender;
use texture::{Dimensionable, Layerable, RawTexture};
use vertex::{CompatibleVertex, Vertex};

/// A dynamic rendering pipeline. A *pipeline* is responsible of rendering into a `Framebuffer`.
///
/// `L` refers to the `Layering` of the underlying `Framebuffer`.
///
/// `D` refers to the `Dim` of the underlying `Framebuffer`.
///
/// `CS` and `DS` are – respectively – the *color* and *depth* `Slot`(s) of the underlying
/// `Framebuffer`.
///
/// Pipelines also have several transient objects:
///
/// - a *clear color*, used to clear the framebuffer
/// - a *texture set*, used to make textures available in subsequent structures
/// - a *buffer set*, used to make uniform buffers available in subsequent structures
#[derive(Clone)]
pub struct Pipeline<'a, L, D, CS, DS>
    where L: 'a + Layerable,
          D: 'a + Dimensionable,
          D::Size: Copy,
          CS: 'a + ColorSlot<L, D>,
          DS: 'a + DepthSlot<L, D> {
  /// The embedded framebuffer.
  framebuffer: &'a Framebuffer<L, D, CS, DS>,
  /// The color used to clean the framebuffer when executing the pipeline.
  clear_color: [f32; 4],
  /// Texture set.
  texture_set: &'a[&'a RawTexture],
  /// Buffer set.
  buffer_set: &'a[&'a RawBuffer]
}

impl<'a, L, D, CS, DS> Pipeline<'a, L, D, CS, DS>
    where L: 'a + Layerable,
          D: 'a + Dimensionable,
          D::Size: Copy,
          CS: 'a + ColorSlot<L, D>,
          DS: 'a + DepthSlot<L, D> {
  /// Create a new pipeline.
  pub fn new(framebuffer: &'a Framebuffer<L, D, CS, DS>, clear_color: [f32; 4],
             texture_set: &'a[&'a RawTexture], buffer_set: &'a[&'a RawBuffer]) -> Self {
    Pipeline {
      framebuffer: framebuffer,
      clear_color: clear_color,
      texture_set: texture_set,
      buffer_set: buffer_set
    }
  }

  /// Enter a `Pipeline`.
  pub fn enter<F>(&self, f: F) where F: FnOnce(&ShadingGate) {
    let clear_color = self.clear_color;

    unsafe {
      gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer.handle());
      gl::Viewport(0, 0, self.framebuffer.width() as GLint, self.framebuffer.height() as GLint);
      gl::ClearColor(clear_color[0], clear_color[1], clear_color[2], clear_color[3]);
      gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

      bind_uniform_buffers(self.buffer_set);
      bind_textures(self.texture_set);
    }

    f(&ShadingGate);
  }
}

pub struct ShadingGate;

impl ShadingGate {
  pub fn new<'a, In, Out, Uni>(&self, program: &'a Program<In, Out, Uni>, texture_set: &'a [&'a RawTexture], buffer_set: &'a [&'a RawBuffer]) -> ShadingCommand<'a, In, Out, Uni> {
    ShadingCommand::new(program, texture_set, buffer_set)
  }
}

/// A dynamic *shading command*. A shading command gathers *render commands* under a shader
/// `Program`.
#[derive(Clone)]
pub struct ShadingCommand<'a, In, Out, Uni> where In: 'a, Out: 'a, Uni: 'a {
  /// Embedded program.
  program: &'a Program<In, Out, Uni>,
  /// Texture set.
  texture_set: &'a [&'a RawTexture],
  /// Buffer set.
  buffer_set: &'a [&'a RawBuffer]
}

impl<'a, In, Out, Uni> ShadingCommand<'a, In, Out, Uni> {
  /// Create a new shading command.
  fn new(program: &'a Program<In, Out, Uni>, texture_set: &'a [&'a RawTexture], buffer_set: &'a [&'a RawBuffer]) -> Self {
    ShadingCommand {
      program: program,
      texture_set: texture_set,
      buffer_set
    }
  }

  /// Enter a `ShadingCommand`.
  pub fn enter<F>(&self, f: F)
      where F: FnOnce(&RenderGate<In>, &Uni),
            In: Vertex,
            Uni: UniformInterface {
    unsafe { gl::UseProgram(self.program.handle()) };

    bind_uniform_buffers(self.buffer_set);
    bind_textures(self.texture_set);

    let render_gate = RenderGate {
      _v: PhantomData,
    };

    let uni_iface = unsafe { self.program.uniform_interface() };
    f(&render_gate, uni_iface);
  }
}

pub struct RenderGate<V> {
  _v: PhantomData<*const V>,
}

impl<V> RenderGate<V> {
  pub fn new<'a, B>(&self, blending: B, depth_test: bool, texture_set: &'a [&'a RawTexture], buffer_set: &'a [&'a RawBuffer]) -> RenderCommand<'a, V> where B: Into<Option<(Equation, Factor, Factor)>> {
    RenderCommand::new(blending, depth_test, texture_set, buffer_set)
  }
}

/// A render command, which holds information on how to rasterize tessellations and render-related
/// hints (like blending equations and factors and whether the depth test should be enabled).
#[derive(Clone)]
pub struct RenderCommand<'a, V> {
  /// Color blending configuration. Set to `None` if you don’t want any color blending. Set it to
  /// `Some(equation, source, destination)` if you want to perform a color blending with the
  /// `equation` formula and with the `source` and `destination` blending factors.
  blending: Option<(Equation, Factor, Factor)>,
  /// Should a depth test be performed?
  depth_test: bool,
  /// Texture set.
  texture_set: &'a [&'a RawTexture],
  /// Buffer set.
  buffer_set: &'a [&'a RawBuffer],
  _v: PhantomData<*const V>,
}

impl<'a, V> RenderCommand<'a, V> {
  /// Create a new render command.
  fn new<B>(blending: B,
            depth_test: bool,
            texture_set: &'a [&'a RawTexture],
            buffer_set: &'a [&'a RawBuffer])
            -> Self where B: Into<Option<(Equation, Factor, Factor)>>{
    RenderCommand {
      blending: blending.into(),
      depth_test: depth_test,
      texture_set: texture_set,
      buffer_set: buffer_set,
      _v: PhantomData,
    }
  }

  /// Enter the render command.
  pub fn enter<F>(&self, f: F) where F: FnOnce(&TessGate<V>) {
    bind_uniform_buffers(self.buffer_set);
    bind_textures(self.texture_set);

    set_blending(self.blending);
    set_depth_test(self.depth_test);

    let tess_gate = TessGate {
      _v: PhantomData,
    };

    f(&tess_gate);
  }
}

pub struct TessGate<V> {
  _v: PhantomData<*const V>
}

impl<V> TessGate<V> where V: Vertex {
  pub fn render<W>(&self,
                   tess: TessRender<W>,
                   texture_set: &[&RawTexture],
                   buffer_set: &[&RawBuffer])
      where W: CompatibleVertex<V> {
    bind_uniform_buffers(buffer_set);
    bind_textures(texture_set);

    tess.render();
  }
}

#[inline]
fn bind_uniform_buffers(uniform_buffers: &[&RawBuffer]) {
  for (index, buf) in uniform_buffers.iter().enumerate() {
    unsafe { gl::BindBufferBase(gl::UNIFORM_BUFFER, index as GLuint, buf.handle()); }
  }
}

#[inline]
fn bind_textures(textures: &[&RawTexture]) {
  for (unit, tex) in textures.iter().enumerate() {
    unsafe {
      gl::ActiveTexture(gl::TEXTURE0 + unit as GLenum);
      gl::BindTexture(tex.target(), tex.handle());
    }
  }
}

#[inline]
fn set_blending<B>(blending: B) where B: Into<Option<(Equation, Factor, Factor)>> {
  match blending.into() {
    Some((equation, src_factor, dest_factor)) => {
      unsafe {
        gl::Enable(gl::BLEND);
        gl::BlendEquation(opengl_blending_equation(equation));
        gl::BlendFunc(opengl_blending_factor(src_factor), opengl_blending_factor(dest_factor));
      }
    },
    None => {
      unsafe { gl::Disable(gl::BLEND) };
    }
  }
}

#[inline]
fn set_depth_test(test: bool) {
  unsafe {
    if test {
      gl::Enable(gl::DEPTH_TEST);
    } else {
      gl::Disable(gl::DEPTH_TEST);
    }
  }
}

#[inline]
fn opengl_blending_equation(equation: Equation) -> GLenum {
  match equation {
    Equation::Additive => gl::FUNC_ADD,
    Equation::Subtract => gl::FUNC_SUBTRACT,
    Equation::ReverseSubtract => gl::FUNC_REVERSE_SUBTRACT,
    Equation::Min => gl::MIN,
    Equation::Max => gl::MAX
  }
}

#[inline]
fn opengl_blending_factor(factor: Factor) -> GLenum {
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
