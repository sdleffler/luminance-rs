//! Dynamic rendering pipelines.
//!
//! This module gives you materials to build *dynamic* rendering **pipelines**. A `Pipeline`
//! represents a functional stream that consumes geometric data and rasterizes them.

use gl;
use gl::types::*;

use buffer::RawBuffer;
use blending;
use framebuffer::{ColorSlot, DepthSlot, Framebuffer};
use shader::program::{AlterUniform, Program};
use tess::TessRender;
use texture::{Dimensionable, Layerable, RawTexture};

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
  buffer_set: &'a[&'a RawBuffer],
  /// Shading commands to render into the embedded framebuffer.
  shading_commands: &'a [Pipe<'a, ShadingCommand<'a>>]
}

impl<'a, L, D, CS, DS> Pipeline<'a, L, D, CS, DS>
    where L: 'a + Layerable,
          D: 'a + Dimensionable,
          D::Size: Copy,
          CS: 'a + ColorSlot<L, D>,
          DS: 'a + DepthSlot<L, D> {
  /// Create a new pipeline.
  pub fn new(framebuffer: &'a Framebuffer<L, D, CS, DS>, clear_color: [f32; 4],
             texture_set: &'a[&'a RawTexture], buffer_set: &'a[&'a RawBuffer],
             shading_commands: &'a [Pipe<'a, ShadingCommand<'a>>]) -> Self {
    Pipeline {
      framebuffer: framebuffer,
      clear_color: clear_color,
      texture_set: texture_set,
      buffer_set: buffer_set,
      shading_commands: shading_commands
    }
  }

  /// Run a `Pipeline`.
  pub fn run(self) {
    let clear_color = self.clear_color;

    unsafe {
      gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer.handle());
      gl::Viewport(0, 0, self.framebuffer.width() as GLint, self.framebuffer.height() as GLint);
      gl::ClearColor(clear_color[0], clear_color[1], clear_color[2], clear_color[3]);
      gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

      bind_uniform_buffers(self.buffer_set);
      bind_textures(self.texture_set);
    }

    for pipe_shading_cmd in self.shading_commands {
      Self::run_shading_command(pipe_shading_cmd);
    }
  }

  fn run_shading_command(pipe: &Pipe<'a, ShadingCommand>) {
    let shading_cmd = &pipe.next;
    let program = &shading_cmd.program;

    unsafe { gl::UseProgram(program.handle()) };

    alter_uniforms(program, pipe.uniforms);
    bind_uniform_buffers(pipe.uniform_buffers);
    bind_textures(pipe.textures);

    for pipe_render_cmd in shading_cmd.render_commands {
      Self::run_render_command(program, pipe_render_cmd);
    }
  }

  fn run_render_command(program: &Program, pipe: &Pipe<'a, RenderCommand<'a>>) {
    let render_cmd = &pipe.next;

    alter_uniforms(program, pipe.uniforms);
    bind_uniform_buffers(pipe.uniform_buffers);
    bind_textures(pipe.textures);

    set_blending(render_cmd.blending);
    set_depth_test(render_cmd.depth_test);

    for pipe_tess in render_cmd.tess {
      let tess = &pipe_tess.next;

      alter_uniforms(program, pipe_tess.uniforms);
      bind_textures(pipe.textures);

      tess.render();
    }
  }
}

fn set_blending(blending: Option<(blending::Equation, blending::Factor, blending::Factor)>) {
  match blending {
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

fn set_depth_test(test: bool) {
  unsafe {
    if test {
      gl::Enable(gl::DEPTH_TEST);
    } else {
      gl::Disable(gl::DEPTH_TEST);
    }
  }
}

fn opengl_blending_equation(equation: blending::Equation) -> GLenum {
  match equation {
    blending::Equation::Additive => gl::FUNC_ADD,
    blending::Equation::Subtract => gl::FUNC_SUBTRACT,
    blending::Equation::ReverseSubtract => gl::FUNC_REVERSE_SUBTRACT,
    blending::Equation::Min => gl::MIN,
    blending::Equation::Max => gl::MAX
  }
}

fn opengl_blending_factor(factor: blending::Factor) -> GLenum {
  match factor {
    blending::Factor::One => gl::ONE,
    blending::Factor::Zero => gl::ZERO,
    blending::Factor::SrcColor => gl::SRC_COLOR,
    blending::Factor::SrcColorComplement => gl::ONE_MINUS_SRC_COLOR,
    blending::Factor::DestColor => gl::DST_COLOR,
    blending::Factor::DestColorComplement => gl::ONE_MINUS_DST_COLOR,
    blending::Factor::SrcAlpha => gl::SRC_ALPHA,
    blending::Factor::SrcAlphaComplement => gl::ONE_MINUS_SRC_ALPHA,
    blending::Factor::DstAlpha => gl::DST_ALPHA,
    blending::Factor::DstAlphaComplement => gl::ONE_MINUS_DST_ALPHA,
    blending::Factor::SrcAlphaSaturate => gl::SRC_ALPHA_SATURATE
  }
}

/// A dynamic *shading command*. A shading command gathers *render commands* under a shader
/// `Program`.
#[derive(Clone)]
pub struct ShadingCommand<'a> {
  /// Embedded program.
  program: &'a Program,
  /// Render commands to execute for this shading command.
  render_commands: &'a [Pipe<'a, RenderCommand<'a>>]
}

impl<'a> ShadingCommand<'a> {
  /// Create a new shading command.
  pub fn new(program: &'a Program, render_commands: &'a [Pipe<'a, RenderCommand<'a>>]) -> Self {
    ShadingCommand {
      program: program,
      render_commands: render_commands
    }
  }
}

/// A render command, which holds information on how to rasterize tessellations and render-related
/// hints (like blending equations and factors and whether the depth test should be enabled).
#[derive(Clone)]
pub struct RenderCommand<'a> {
  /// Color blending configuration. Set to `None` if you don’t want any color blending. Set it to
  /// `Some(equation, source, destination)` if you want to perform a color blending with the
  /// `equation` formula and with the `source` and `destination` blending factors.
  blending: Option<(blending::Equation, blending::Factor, blending::Factor)>,
  /// Should a depth test be performed?
  depth_test: bool,
  /// The embedded tessellations.
  tess: &'a [Pipe<'a, TessRender<'a>>],
}

impl<'a> RenderCommand<'a> {
  /// Create a new render command.
  pub fn new(blending: Option<(blending::Equation, blending::Factor, blending::Factor)>,
             depth_test: bool, tess: &'a [Pipe<'a, TessRender<'a>>]) -> Self {
    RenderCommand {
      blending: blending,
      depth_test: depth_test,
      tess: tess,
    }
  }
}

/// A pipe used to build up a `Pipeline` by connecting its inner layers.
#[derive(Clone)]
pub struct Pipe<'a, T> {
  uniforms: &'a [AlterUniform<'a>],
  uniform_buffers: &'a [&'a RawBuffer],
  textures: &'a [&'a RawTexture],
  next: T
}

impl<'a, T> Pipe<'a, T> {
  /// Create a new pipe that just contains the next layer.
  pub fn new(next: T) -> Self {
    Pipe {
      uniforms: &[],
      uniform_buffers: &[],
      textures: &[],
      next: next
    }
  }
}

impl<'a> Pipe<'a, ()> {
  /// Create an empty pipe; it holds nothing.
  pub fn empty() -> Pipe<'a, ()> {
    Self::new(())
  }

  /// Add the next layer to make it hold something.
  pub fn unwrap<T>(self, next: T) -> Pipe<'a, T> {
    Pipe {
      uniforms: self.uniforms,
      uniform_buffers: self.uniform_buffers,
      textures: self.textures,
      next: next
    }
  }

  /// Add uniforms to be altered to this pipe.
  pub fn uniforms(self, uniforms: &'a [AlterUniform<'a>]) -> Self {
    Pipe {
      uniforms: uniforms,
      ..self
    }
  }

  /// Add uniform buffers as available to this pipe.
  pub fn uniform_buffers(self, uniform_buffers: &'a [&'a RawBuffer]) -> Self {
    Pipe {
      uniform_buffers: uniform_buffers,
      ..self
    }
  }

  /// Add textures as available to this pipe.
  pub fn textures(self, textures: &'a [&'a RawTexture]) -> Self {
    Pipe {
      textures: textures,
      ..self
    }
  }
}

#[inline]
fn alter_uniforms(program: &Program, uniforms: &[AlterUniform]) {
  for uniform in uniforms {
    unsafe { uniform.alter(program) };
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
