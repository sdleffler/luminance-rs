//! Dynamic rendering pipelines.
//!
//! This module gives you materials to build *dynamic* rendering **pipelines**. A `Pipeline`
//! represents a functional stream that consumes geometric data and rasterizes them.

use gl;
use gl::types::*;

use buffer::RawBuffer;
use blending::{Equation, Factor};
use framebuffer::{ColorSlot, DepthSlot, Framebuffer};
use shader::program::{Program, UniformInterface};
use std::marker::PhantomData;
use tess::TessRender;
use texture::{Dimensionable, Layerable, RawTexture};
use vertex::Vertex;

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
      where F: FnOnce(&RenderGate<Out>, &Uni),
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

impl<V> TessGate<V> {
  pub fn render(&self, tess: TessRender, texture_set: &[&RawTexture], buffer_set: &[&RawBuffer]) {
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
