//! Pipeline backend interface.
//!
//! This interface defines the low-level API pipelines must implement to be usable.
//!
//! A (graphics) pipeline is a strongly typed object that represents a set of actions to perform to render a scene into
//! a framebuffer. This module exposes the required traits to implement to perform various actions in the pipeline.
//!
//! The most important one is [`PipelineBase`], that provides pipeline creation. This doesn’t feel like much but it’s
//! actually an important part of the backend interface. Besides telling the backend to prepare the pipeline creation,
//! this also serves for the backend to allocate and cache some values in their state so that they don’t get created
//! every time a user starts a pipeline, making dynamic pipeline creation cheap and possible.
//!
//! The [`Pipeline`] trait is the « entry-point » of a render. It takes a [`Framebuffer`] and a [`PipelineState`] and
//! put both objects to the backend to start a render.
//!
//! [`PipelineTexture`], [`PipelineShaderData`] etc. are used to scope-bind specific resources, such as textures and
//! shader data.
//!
//! [`Framebuffer`]: crate::framebuffer::Framebuffer

use crate::{
  backend::{
    framebuffer::Framebuffer as FramebufferBackend,
    shader::ShaderData,
    shading_gate::ShadingGate as ShadingGateBackend,
    texture::{Texture, TextureBase},
  },
  pipeline::{PipelineError, PipelineState},
  pixel::Pixel,
  texture::Dimensionable,
};

/// The base trait of pipelines.
///
/// This trait exposes the [`PipelineBase::new_pipeline`] method, which is called when a new pipeline is created. The
/// backend must create and allocate the data required to execute graphics pipelines, and is strongly advised to cache
/// that data so that this method is cheap after the first frame.
///
/// This trait has [`ShadingGate`] and [`TextureBase`] as super traits, as those are required to perform more operations
/// on pipelines.
///
/// [`ShadingGate`]: crate::backend::shading_gate::ShadingGate
pub unsafe trait PipelineBase: ShadingGateBackend + TextureBase {
  type PipelineRepr;

  /// Create a new (cached) pipeline.
  unsafe fn new_pipeline(&mut self) -> Result<Self::PipelineRepr, PipelineError>;
}

/// Start a pipeline.
///
/// This trait requires [`PipelineBase`] and [`Framebuffer`], as it starts rendering into one.
///
/// [`Framebuffer`]: crate::backend::framebuffer::Framebuffer
pub unsafe trait Pipeline<D>: PipelineBase + FramebufferBackend<D>
where
  D: Dimensionable,
{
  /// Start a pipeline that will output in the input [`Framebuffer`] and the given [`PipelineState`].
  ///
  /// This method should perform the required backend action to take into account the framebuffer and the state to start
  /// the pipeline.
  ///
  /// [`Framebuffer`]: crate::backend::framebuffer::Framebuffer
  unsafe fn start_pipeline(
    &mut self,
    framebuffer: &Self::FramebufferRepr,
    pipeline_state: &PipelineState,
  );
}

/// Operations that can be run on pipelines and textures.
///
/// This trait requires [`PipelineBase`] and [`Texture`].
pub unsafe trait PipelineTexture<D, P>: PipelineBase + Texture<D, P>
where
  D: Dimensionable,
  P: Pixel,
{
  /// Representation of a _bound_ [`Texture`] on the backend.
  type BoundTextureRepr;

  /// Bind a [`Texture`] to the current [`Pipeline`].
  ///
  /// This method must bind the texture on the backend and return an object representing the bound texture. Must of the
  /// time, this bound representation will also implement [`Drop`] so that backend resources are freed and recycled on
  /// the next bind.
  unsafe fn bind_texture(
    pipeline: &Self::PipelineRepr,
    texture: &Self::TextureRepr,
  ) -> Result<Self::BoundTextureRepr, PipelineError>
  where
    D: Dimensionable,
    P: Pixel;

  /// Get the `u32` representation of the bound texture, also known as binding.
  unsafe fn texture_binding(bound: &Self::BoundTextureRepr) -> u32;
}

/// Operations that can be run on pipelines and shader data.
///
/// This trait requires [`PipelineBase`] and [`ShaderData`].
pub unsafe trait PipelineShaderData<T>: PipelineBase + ShaderData<T> {
  /// Representation of a _bound_ [`ShaderData`] on the backend.
  type BoundShaderDataRepr;

  /// Bind a [`ShaderData`] to the current [`Pipeline`].
  ///
  /// This method must bind the shader data on the backend and return an object representing the bound shader data. Must
  /// of the time, this bound representation will also implement [`Drop`] so that backend resources are freed and
  /// recycled on the next bind.
  unsafe fn bind_shader_data(
    pipeline: &Self::PipelineRepr,
    shader_data: &Self::ShaderDataRepr,
  ) -> Result<Self::BoundShaderDataRepr, PipelineError>;

  /// Get the `u32` representation of the bound shader data, also known as binding.
  unsafe fn shader_data_binding(bound: &Self::BoundShaderDataRepr) -> u32;
}
