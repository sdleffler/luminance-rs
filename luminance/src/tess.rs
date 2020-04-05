//! Vertex sets.
//!
//! [`Tess`] is a type that represents the gathering of vertices and the way to connect / link
//! them. A [`Tess`] has several intrinsic properties:
//!
//! - Its _primitive mode_ — [`Mode`]. That object tells the GPU how to connect the vertices.
//! - A default number of vertex to render. When passing the [`Tess`] to the GPU for rendering,
//!   it’s possible to specify the number of vertices to render or just let the [`Tess`] render
//!   a default number of vertices (typically, the whole [`Tess`]).
//! - A default number of _instances_, which allows for geometry instancing. Geometry instancing
//!   is the fact of drawing with the same [`Tess`] (GPU buffers) several times, only changing the
//!   instance index every time a new render is performed. This is done entirely on the GPU to
//!   prevent bandwidth exhaustion. The index of the instance, in the shader stages, is often used
//!   to pick material properties, matrices, etc. to customize each instances.
//! - An indexed configuration, allowing to tell the GPU how to render the vertices by referring to
//!   them via indices.
//! - For indexed configuration, an optional _primitive restart index_ can be specified. That
//!   index, when present in the indexed set, will make some primitive modes _“restart”_ and create
//!   new primitives. More on this on the documentation of [`Mode`].
//!
//! # Tessellation creation
//!
//! [`Tess`] is not created directly. Instead, you need to use a [`TessBuilder`]. Tessellation
//! builders make it easy to customize what a [`Tess`] will be made of before actually requesting
//! the GPU to create them. They support a large number of possible situations:
//!
//! - _Attributeless_: when you only specify the [`Mode`] and number of vertices to render (and
//!   optionally the number of instances). That will create a vertex set with no vertex data. Your
//!   vertex shader will be responsible for creating the vertex attributes on the fly.
//! - _Direct geometry_: when you pass vertices directly.
//! - _Indexed geometry_: when you pass vertices and reference from with indices.
//! - _Instanced geometry_: when you ask to use instances, making the graphics pipeline create
//!   several instances of your vertex set on the GPU.
//!
//! # Tessellation views
//!
//! Once you have a [`Tess`] — created from [`TessBuilder::build`], you can now render it in a
//! [`TessGate`]. In order to do so, you need a [`TessView`].
//!
//! A [`TessView`] is a temporary _view_ into a [`Tess`], describing what part of it should be
//! drawn. Creating [`TessView`]s is a cheap operation, and can be done in two different ways:
//!
//! - By directly using the methods from [`TessView`].
//! - By using the [`SubTess`] trait.
//!
//! The [`SubTess`] trait is a convenient way to create [`TessView`]. It provides the
//! [`SubTess::slice`] and [`SubTess::inst_slice`] methods, which accept Rust’s range operators
//! to create the [`TessView`]s in a more comfortable way.
//!
//! # Tessellation mapping
//!
//! Sometimes, you will want to edit tessellations in a dynamic way instead of re-creating new
//! ones. That can be useful for streaming data of for using a small part of a big [`Tess`]. The
//! [`Tess`] type has several methods to obtain [`TessSlice`] and [`TessSliceMut`] objects, which
//! allow you to map values and iterate over them via standard Rust slices. See
//! [`TessSlice::as_slice`] and [`TessSliceMut::as_slice_mut`].
//!
//! > Note: because of their slice nature, both [`TessSlice`] and [`TessSliceMut`] won’t help you
//! > if you want to resize [`Tess`]. This is not currently supported.
//!
//! [`TessGate`]: crate::tess_gate::TessGate

use std::fmt;
use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

use crate::backend::tess::{
  Tess as TessBackend, TessBuilder as TessBuilderBackend, TessSlice as TessSliceBackend,
};
use crate::buffer::BufferError;
use crate::context::GraphicsContext;
use crate::vertex::Vertex;
use crate::vertex::VertexDesc;

/// Vertices can be connected via several modes.
///
/// Some modes allow for _primitive restart_. Primitive restart is a cool feature that allows to
/// _break_ the building of a primitive to _start over again_. For instance, when making a curve,
/// you can imagine gluing segments next to each other. If at some point, you want to start a new
/// curve, you have two choices:
///
///   - Either you stop your draw call and make another one.
///   - Or you just use the _primitive restart_ feature to ask to create another line from scratch.
///
/// _Primitive restart_ should be used as much as possible as it will decrease the number of GPU
/// commands you have to issue.
///
/// That feature is encoded with a special _vertex index_. You can setup the value of the _primitive
/// restart index_ with [`TessBuilder::set_primitive_restart_index`]. Whenever a vertex index is set
/// to the same value as the _primitive restart index_, the value is not interpreted as a vertex
/// index but just a marker / hint to start a new primitive.
#[derive(Copy, Clone, Debug)]
pub enum Mode {
  /// A single point.
  ///
  /// Points are left unconnected from each other and represent a _point cloud_. This is the typical
  /// primitive mode you want to do, for instance, particles rendering.
  Point,
  /// A line, defined by two points.
  ///
  /// Every pair of vertices are connected together to form a straight line.
  Line,
  /// A strip line, defined by at least two points and zero or many other ones.
  ///
  /// The first two vertices create a line, and every new vertex flowing in the graphics pipeline
  /// (starting from the third, then) well extend the initial line, making a curve composed of
  /// several segments.
  ///
  /// > This kind of primitive mode allows the usage of _primitive restart_.
  LineStrip,
  /// A triangle, defined by three points.
  Triangle,
  /// A triangle fan, defined by at least three points and zero or many other ones.
  ///
  /// Such a mode is easy to picture: a cooling fan is a circular shape, with blades.
  /// [`Mode::TriangleFan`] is kind of the same. The first vertex is at the center of the fan, then
  /// the second vertex creates the first edge of the first triangle. Every time you add a new
  /// vertex, a triangle is created by taking the first (center) vertex, the very previous vertex
  /// and the current vertex. By specifying vertices around the center, you actually create a
  /// fan-like shape.
  ///
  /// > This kind of primitive mode allows the usage of _primitive restart_.
  TriangleFan,
  /// A triangle strip, defined by at least three points and zero or many other ones.
  ///
  /// This mode is a bit different from [`Mode::TriangleFan`]. The first two vertices define the
  /// first edge of the first triangle. Then, for each new vertex, a new triangle is created by
  /// taking the very previous vertex and the last to very previous vertex. What it means is that
  /// every time a triangle is created, the next vertex will share the edge that was created to
  /// spawn the previous triangle.
  ///
  /// This mode is useful to create long ribbons / strips of triangles.
  ///
  /// > This kind of primitive mode allows the usage of _primitive restart_.
  TriangleStrip,
  /// A general purpose primitive with _n_ vertices, for use in tessellation shaders.
  /// For example, `Mode::Patch(3)` represents triangle patches, so every three vertices in the
  /// buffer form a patch.
  /// If you want to employ tessellation shaders, this is the only primitive mode you can use.
  Patch(usize),
}

/// Error that can occur while trying to map GPU tessellations to host code.
#[derive(Debug, Eq, PartialEq)]
pub enum TessMapError {
  /// The CPU mapping failed due to buffer errors.
  BufferMapError(BufferError),
  /// Vertex target type is not the same as the one stored in the buffer.
  VertexTypeMismatch(VertexDesc, VertexDesc),
  /// Index target type is not the same as the one stored in the buffer.
  IndexTypeMismatch(TessIndexType, TessIndexType),
  /// The CPU mapping failed because you cannot map an attributeless tessellation since it doesn’t
  /// have any vertex attribute.
  ForbiddenAttributelessMapping,
  /// The CPU mapping failed because currently, mapping deinterleaved buffers is not supported via
  /// a single slice.
  ForbiddenDeinterleavedMapping,
}

impl fmt::Display for TessMapError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      TessMapError::BufferMapError(ref e) => write!(f, "cannot map tessellation buffer: {}", e),
      TessMapError::VertexTypeMismatch(ref a, ref b) => write!(
        f,
        "cannot map tessellation: vertex type mismatch between {:?} and {:?}",
        a, b
      ),
      TessMapError::IndexTypeMismatch(ref a, ref b) => write!(
        f,
        "cannot map tessellation: index type mismatch between {:?} and {:?}",
        a, b
      ),
      TessMapError::ForbiddenAttributelessMapping => {
        f.write_str("cannot map an attributeless buffer")
      }
      TessMapError::ForbiddenDeinterleavedMapping => {
        f.write_str("cannot map a deinterleaved buffer as interleaved")
      }
    }
  }
}

/// Possible errors that might occur when dealing with [`Tess`].
#[derive(Debug)]
pub enum TessError {
  /// Error related to attributeless tessellation and/or render.
  AttributelessError(String),
  /// Length incoherency in vertex, index or instance buffers.
  LengthIncoherency(usize),
  /// Overflow when accessing underlying buffers.
  Overflow(usize, usize),
  /// Internal error ocurring with a buffer.
  InternalBufferError(BufferError),
}

impl From<BufferError> for TessError {
  fn from(e: BufferError) -> Self {
    TessError::InternalBufferError(e)
  }
}

/// Possible tessellation index types.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TessIndexType {
  /// 8-bit unsigned integer.
  U8,
  /// 16-bit unsigned integer.
  U16,
  /// 32-bit unsigned integer.
  U32,
}

impl TessIndexType {
  /// Get the number of bytes that are needed to represent a type described by the variant.
  pub fn bytes(self) -> usize {
    match self {
      TessIndexType::U8 => 1,
      TessIndexType::U16 => 2,
      TessIndexType::U32 => 4,
    }
  }
}

/// Class of tessellation indices.
///
/// Values which types implement this trait are allowed to be used to index tessellation in *indexed
/// draw commands*.
///
/// You shouldn’t have to worry too much about that trait. Have a look at the current implementors
/// for an exhaustive list of types you can use.
///
/// > Implementing this trait is `unsafe`.
pub unsafe trait TessIndex {
  /// Type of the underlying index.
  ///
  /// You are limited in which types you can use as indexes. Feel free to have a look at the
  /// documentation of the [`TessIndexType`] trait for further information.
  const INDEX_TYPE: TessIndexType;
}

/// Boop.
unsafe impl TessIndex for u8 {
  const INDEX_TYPE: TessIndexType = TessIndexType::U8;
}

/// Boop.
unsafe impl TessIndex for u16 {
  const INDEX_TYPE: TessIndexType = TessIndexType::U16;
}

/// Wuuuuuuha.
unsafe impl TessIndex for u32 {
  const INDEX_TYPE: TessIndexType = TessIndexType::U32;
}

/// [`Tess`] builder object.
///
/// This type allows to create [`Tess`] via a _builder pattern_.
///
/// # Parametricity
///
/// - `B` is the backend type.
pub struct TessBuilder<'a, B>
where
  B: TessBuilderBackend,
{
  backend: &'a mut B,
  repr: B::TessBuilderRepr,
}

impl<'a, B> Default for TessBuilder<'a, B>
where
  B: TessBuilderBackend,
{
  /// See the documentation of [`TessBuilder::new`] for further information.
  fn default() -> Self {
    Self::new()
  }
}

impl<'a, B> TessBuilder<'a, B>
where
  B: TessBuilderBackend,
{
  /// Create a new default [`TessBuilder`].
  ///
  /// The default value for this type is backend-dependent. Refer to the implementation of your
  /// backend type. Especially, the implementation of [`TessBuilder::new_tess_builder`] will give
  /// you that information
  ///
  /// [`TessBuilder::new_tess_builder`]: crate::backend::tess::TessBuilder::new_tess_builder
  pub fn new<C>(ctx: &'a mut C) -> Result<Self, TessError>
  where
    C: GraphicsContext<Backend = B>,
  {
    unsafe {
      ctx
        .backend()
        .new_tess_builder()
        .map(move |repr| TessBuilder {
          backend: ctx.backend(),
          repr,
        })
    }
  }

  /// Add vertices to be bundled in the [`Tess`].
  ///
  /// Every time you call that function, the set of vertices is added to a new internal set. That
  /// means that you can separate all the attributes from your vertex type into several arrays of
  /// each attributes and call that function several times. This will yield a _deinterleaved_
  /// [`Tess`], opposed to an _interleaved_ [`Tess`] if you just submit a single array of structure
  /// once.
  pub fn add_vertices<V, W>(mut self, vertices: W) -> Result<Self, TessError>
  where
    W: AsRef<[V]>,
    V: Vertex,
  {
    unsafe {
      self
        .backend
        .add_vertices(&mut self.repr, vertices)
        .map(move |_| self)
    }
  }

  /// Add instance data to be bundled in the [`Tess`].
  ///
  /// Every time you call that function, the set of data is added to a new internal set. That
  /// means that you can separate all the attributes from your vertex type into several arrays of
  /// each attributes and call that function several times. This will yield a _deinterleaved_
  /// [`Tess`], opposed to an _interleaved_ [`Tess`] if you just submit a single array of structure
  /// once.
  ///
  /// Instance data is a per-vertex special attribute that gets extracted for each instance, while
  /// regular vertex data (added via [`TessBuilder::add_vertices`] is shared for all instances.
  pub fn add_instances<V, W>(mut self, instances: W) -> Result<Self, TessError>
  where
    W: AsRef<[V]>,
    V: Vertex,
  {
    unsafe {
      self
        .backend
        .add_instances(&mut self.repr, instances)
        .map(move |_| self)
    }
  }

  /// Set indices to index into the vertex data.
  ///
  /// That function should be called only once. Calling it twice ends up replacing the already
  /// present indices.
  pub fn set_indices<T, I>(mut self, indices: T) -> Result<Self, TessError>
  where
    T: AsRef<[I]>,
    I: TessIndex,
  {
    unsafe {
      self
        .backend
        .set_indices(&mut self.repr, indices)
        .map(move |_| self)
    }
  }

  /// Set the [`Mode`] to connect vertices.
  ///
  /// Calling that function twice replace the previously set value.
  pub fn set_mode(mut self, mode: Mode) -> Result<Self, TessError> {
    unsafe {
      self
        .backend
        .set_mode(&mut self.repr, mode)
        .map(move |_| self)
    }
  }

  /// Set the default number of vertices to render.
  ///
  /// Calling that function twice replace the previously set value.
  pub fn set_vertex_nb(mut self, nb: usize) -> Result<Self, TessError> {
    unsafe {
      self
        .backend
        .set_vertex_nb(&mut self.repr, nb)
        .map(move |_| self)
    }
  }

  /// Set the default number of instances to render.
  ///
  /// Calling that function twice replace the previously set value.
  pub fn set_instance_nb(mut self, nb: usize) -> Result<Self, TessError> {
    unsafe {
      self
        .backend
        .set_instance_nb(&mut self.repr, nb)
        .map(move |_| self)
    }
  }

  /// Set the primitive restart index.
  ///
  /// Calling that function twice replace the previously set value.
  pub fn set_primitive_restart_index<T>(mut self, index: T) -> Result<Self, TessError>
  where
    T: Into<Option<u32>>,
  {
    unsafe {
      self
        .backend
        .set_primitive_restart_index(&mut self.repr, index.into())
        .map(move |_| self)
    }
  }
}

impl<'a, B> TessBuilder<'a, B>
where
  B: TessBuilderBackend + TessBackend,
{
  /// Build a [`Tess`] if the [`TessBuilder`] has enough data and is in a valid state. What is
  /// needed is backend-dependent but most of the time, you will want to:
  ///
  /// - Set a [`Mode`].
  /// - Give vertex data and optionally indices, or give none of them (attributeless objects).
  /// - If you provide vertex data by submitting several sets with [`TessBuilder::add_vertices`]
  ///   and/or [`TessBuilder::add_instances`], do not forget that you must submit sets with the
  ///   same size. Otherwise, the GPU will not know what values use for missing attributes in
  ///   vertices.
  pub fn build(self) -> Result<Tess<B>, TessError> {
    unsafe { self.backend.build(self.repr).map(|repr| Tess { repr }) }
  }
}

/// A GPU vertex set.
///
/// Vertex set are the only way to represent space data. The dimension you choose is up to you, but
/// people will typically want to represent objects in 2D or 3D. A _vertex_ is a point in such
/// space and it carries _properties_ — called _“vertex attributes_”. Those attributes are
/// completely free to use. They must, however, be compatible with the [`Semantics`] and [`Vertex`]
/// traits.
///
/// [`Tess`] are built out of [`TessBuilder`] and can be _sliced_ to edit their content in-line —
/// by mapping the GPU memory region onto your virtual address memory space that your CPU knows —
/// or be part of a render via a [`TessGate`].
///
/// [`Semantics`]: crate::vertex::Semantics
/// [`TessGate`]: crate::tess_gate::TessGate
#[derive(Debug)]
pub struct Tess<B>
where
  B: ?Sized + TessBackend,
{
  pub(crate) repr: B::TessRepr,
}

impl<B> Drop for Tess<B>
where
  B: ?Sized + TessBackend,
{
  fn drop(&mut self) {
    unsafe { B::destroy_tess(&mut self.repr) };
  }
}

impl<B> Tess<B>
where
  B: ?Sized + TessBackend,
{
  /// Get the number of vertices.
  pub fn vert_nb(&self) -> usize {
    unsafe { B::tess_vertices_nb(&self.repr) }
  }

  /// Get the number of indices.
  pub fn inst_nb(&self) -> usize {
    unsafe { B::tess_instances_nb(&self.repr) }
  }

  /// Immutably slice the [`Tess`] in order to read its content via usual slices.
  ///
  /// This method gives access to the underlying _vertex storage_.
  pub fn slice_vertices<T>(&self) -> Result<TessSlice<B, T>, TessMapError>
  where
    B: TessSliceBackend<T>,
    T: Vertex,
  {
    unsafe { B::slice_vertices(&self.repr).map(|repr| TessSlice { repr }) }
  }

  /// Mutably slice the [`Tess`] in order to read and/or edit its content via usual slices.
  ///
  /// This method gives access to the underlying _vertex storage_.
  pub fn slice_vertices_mut<T>(&mut self) -> Result<TessSliceMut<B, T>, TessMapError>
  where
    B: TessSliceBackend<T>,
    T: Vertex,
  {
    unsafe { B::slice_vertices_mut(&mut self.repr).map(|repr| TessSliceMut { repr }) }
  }

  /// Immutably slice the [`Tess`] in order to read its content via usual slices.
  ///
  /// This method gives access to the underlying _index storage_.
  pub fn slice_indices<T>(&self) -> Result<TessSlice<B, T>, TessMapError>
  where
    B: TessSliceBackend<T>,
    T: TessIndex,
  {
    unsafe { B::slice_indices(&self.repr).map(|repr| TessSlice { repr }) }
  }

  /// Mutably slice the [`Tess`] in order to read and/or edit its content via usual slices.
  ///
  /// This method gives access to the underlying _index storage_.
  pub fn slice_indices_mut<T>(&mut self) -> Result<TessSliceMut<B, T>, TessMapError>
  where
    B: TessSliceBackend<T>,
    T: TessIndex,
  {
    unsafe { B::slice_indices_mut(&mut self.repr).map(|repr| TessSliceMut { repr }) }
  }

  /// Immutably slice the [`Tess`] in order to read its content via usual slices.
  ///
  /// This method gives access to the underlying _instance storage_.
  pub fn slice_instances<T>(&self) -> Result<TessSlice<B, T>, TessMapError>
  where
    B: TessSliceBackend<T>,
    T: Vertex,
  {
    unsafe { B::slice_instances(&self.repr).map(|repr| TessSlice { repr }) }
  }

  /// Mutably slice the [`Tess`] in order to read and/or edit its content via usual slices.
  ///
  /// This method gives access to the underlying _instance storage_.
  pub fn slice_instances_mut<T>(&mut self) -> Result<TessSliceMut<B, T>, TessMapError>
  where
    B: TessSliceBackend<T>,
    T: Vertex,
  {
    unsafe { B::slice_instances_mut(&mut self.repr).map(|repr| TessSliceMut { repr }) }
  }
}

/// A [`Tess`] immutable slice.
#[derive(Debug)]
pub struct TessSlice<B, T>
where
  B: ?Sized + TessSliceBackend<T>,
{
  repr: B::SliceRepr,
}

impl<B, T> Drop for TessSlice<B, T>
where
  B: ?Sized + TessSliceBackend<T>,
{
  fn drop(&mut self) {
    unsafe { B::destroy_tess_slice(&mut self.repr) };
  }
}

impl<B, T> TessSlice<B, T>
where
  B: ?Sized + TessSliceBackend<T>,
{
  /// Get access to an immutable slice.
  pub fn as_slice(&self) -> Result<&[T], TessMapError> {
    unsafe { B::obtain_slice(&self.repr) }
  }
}

/// Mutable [`Tess`] slice.
#[derive(Debug)]
pub struct TessSliceMut<B, T>
where
  B: ?Sized + TessSliceBackend<T>,
{
  repr: B::SliceMutRepr,
}

impl<B, T> Drop for TessSliceMut<B, T>
where
  B: ?Sized + TessSliceBackend<T>,
{
  fn drop(&mut self) {
    unsafe { B::destroy_tess_slice_mut(&mut self.repr) };
  }
}

impl<B, T> TessSliceMut<B, T>
where
  B: ?Sized + TessSliceBackend<T>,
{
  /// Get access to a mutable slice.
  pub fn as_slice_mut(&mut self) -> Result<&mut [T], TessMapError> {
    unsafe { B::obtain_slice_mut(&mut self.repr) }
  }
}

/// Possible error that might occur while dealing with [`TessView`] objects.
#[non_exhaustive]
#[derive(Debug)]
pub enum TessViewError {
  /// The view has incorrect size.
  ///
  /// `capacity` refers to the number of current data and `start` and `nb` the number of requested
  /// data.
  IncorrectViewWindow {
    capacity: usize,
    start: usize,
    nb: usize,
  },
}

/// A _view_ into a GPU tessellation.
#[derive(Clone)]
pub struct TessView<'a, S>
where
  S: ?Sized + TessBackend,
{
  /// Tessellation to render.
  pub(crate) tess: &'a Tess<S>,
  /// Start index (vertex) in the tessellation.
  pub(crate) start_index: usize,
  /// Number of vertices to pick from the tessellation.
  pub(crate) vert_nb: usize,
  /// Number of instances to render.
  pub(crate) inst_nb: usize,
}

impl<'a, S> TessView<'a, S>
where
  S: ?Sized + TessBackend,
{
  /// Create a view that is using the whole input [`Tess`].
  pub fn whole(tess: &'a Tess<S>) -> Self {
    TessView {
      tess,
      start_index: 0,
      vert_nb: tess.vert_nb(),
      inst_nb: tess.inst_nb(),
    }
  }

  /// Create a view that is using the whole input [`Tess`] with `inst_nb` instances.
  pub fn inst_whole(tess: &'a Tess<S>, inst_nb: usize) -> Self {
    TessView {
      tess,
      start_index: 0,
      vert_nb: tess.vert_nb(),
      inst_nb,
    }
  }

  /// Create a view that is using only a subpart of the input [`Tess`], starting from the beginning
  /// of the vertices.
  pub fn sub(tess: &'a Tess<S>, vert_nb: usize) -> Result<Self, TessViewError> {
    let capacity = tess.vert_nb();

    if vert_nb > capacity {
      return Err(TessViewError::IncorrectViewWindow {
        capacity,
        start: 0,
        nb: vert_nb,
      });
    }

    Ok(TessView {
      tess,
      start_index: 0,
      vert_nb,
      inst_nb: tess.inst_nb(),
    })
  }

  /// Create a view that is using only a subpart of the input [`Tess`], starting from the beginning
  /// of the vertices, with `inst_nb` instances.
  pub fn inst_sub(
    tess: &'a Tess<S>,
    vert_nb: usize,
    inst_nb: usize,
  ) -> Result<Self, TessViewError> {
    let capacity = tess.vert_nb();

    if vert_nb > capacity {
      return Err(TessViewError::IncorrectViewWindow {
        capacity,
        start: 0,
        nb: vert_nb,
      });
    }

    Ok(TessView {
      tess,
      start_index: 0,
      vert_nb,
      inst_nb,
    })
  }

  /// Create a view that is using only a subpart of the input [`Tess`], starting from `start`, with
  /// `nb` vertices.
  pub fn one_slice(tess: &'a Tess<S>, start: usize, nb: usize) -> Result<Self, TessViewError> {
    let capacity = tess.vert_nb();

    if start > capacity || nb + start > capacity {
      return Err(TessViewError::IncorrectViewWindow {
        capacity,
        start,
        nb,
      });
    }

    Ok(TessView {
      tess,
      start_index: start,
      vert_nb: nb,
      inst_nb: 1,
    })
  }

  /// Create a view that is using only a subpart of the input [`Tess`], starting from `start`, with
  /// `nb` vertices and `inst_nb` instances.
  pub fn inst_slice(
    tess: &'a Tess<S>,
    start: usize,
    nb: usize,
    inst_nb: usize,
  ) -> Result<Self, TessViewError> {
    let capacity = tess.vert_nb();

    if start > capacity || nb + start > capacity {
      return Err(TessViewError::IncorrectViewWindow {
        capacity,
        start,
        nb,
      });
    }

    Ok(TessView {
      tess,
      start_index: start,
      vert_nb: nb,
      inst_nb,
    })
  }
}

impl<'a, S> From<&'a Tess<S>> for TessView<'a, S>
where
  S: ?Sized + TessBackend,
{
  fn from(tess: &'a Tess<S>) -> Self {
    TessView::one_whole(tess)
  }
}

/// [`TessView`] helper trait.
///
/// This trait helps to create [`TessView`] by allowing using the Rust range operators, such as
///
/// - [`..`](https://doc.rust-lang.org/std/ops/struct.RangeFull.html); the full range operator.
/// - [`a .. b`](https://doc.rust-lang.org/std/ops/struct.Range.html); the range operator.
/// - [`a ..`](https://doc.rust-lang.org/std/ops/struct.RangeFrom.html); the range-from operator.
/// - [`.. b`](https://doc.rust-lang.org/std/ops/struct.RangeTo.html); the range-to operator.
/// - [`..= b`](https://doc.rust-lang.org/std/ops/struct.RangeToInclusive.html); the inclusive range-to operator.
pub trait SubTess<S, Idx>
where
  S: ?Sized + TessBackend,
{
  /// Slice a tessellation object and yields a [`TessView`] according to the index range.
  fn slice(&self, idx: Idx) -> Result<TessView<S>, TessViewError>;

  /// Slice a tesselation object and yields a [`TessView`] according to the index range with as
  /// many instances as specified.
  fn inst_slice(&self, idx: Idx, inst_nb: usize) -> Result<TessView<S>, TessViewError>;
}

impl<S> SubTess<S, RangeFull> for Tess<S>
where
  S: ?Sized + TessBackend,
{
  fn slice(&self, _: RangeFull) -> Result<TessView<S>, TessViewError> {
    Ok(TessView::one_whole(self))
  }

  fn inst_slice(&self, _: RangeFull, inst_nb: usize) -> Result<TessView<S>, TessViewError> {
    Ok(TessView::inst_whole(self, inst_nb))
  }
}

impl<S> SubTess<S, RangeTo<usize>> for Tess<S>
where
  S: ?Sized + TessBackend,
{
  fn slice(&self, to: RangeTo<usize>) -> Result<TessView<S>, TessViewError> {
    TessView::one_sub(self, to.end)
  }

  fn inst_slice(&self, to: RangeTo<usize>, inst_nb: usize) -> Result<TessView<S>, TessViewError> {
    TessView::inst_sub(self, to.end, inst_nb)
  }
}

impl<S> SubTess<S, RangeFrom<usize>> for Tess<S>
where
  S: ?Sized + TessBackend,
{
  fn slice(&self, from: RangeFrom<usize>) -> Result<TessView<S>, TessViewError> {
    TessView::one_slice(self, from.start, self.vert_nb() - from.start)
  }

  fn inst_slice(
    &self,
    from: RangeFrom<usize>,
    inst_nb: usize,
  ) -> Result<TessView<S>, TessViewError> {
    TessView::inst_slice(self, from.start, self.vert_nb() - from.start, inst_nb)
  }
}

impl<S> SubTess<S, Range<usize>> for Tess<S>
where
  S: ?Sized + TessBackend,
{
  fn slice(&self, range: Range<usize>) -> Result<TessView<S>, TessViewError> {
    TessView::one_slice(self, range.start, range.end - range.start)
  }

  fn inst_slice(&self, range: Range<usize>, inst_nb: usize) -> Result<TessView<S>, TessViewError> {
    TessView::inst_slice(self, range.start, range.end - range.start, inst_nb)
  }
}

impl<S> SubTess<S, RangeInclusive<usize>> for Tess<S>
where
  S: ?Sized + TessBackend,
{
  fn slice(&self, range: RangeInclusive<usize>) -> Result<TessView<S>, TessViewError> {
    let start = *range.start();
    let end = *range.end();
    TessView::one_slice(self, start, end - start + 1)
  }

  fn inst_slice(
    &self,
    range: RangeInclusive<usize>,
    inst_nb: usize,
  ) -> Result<TessView<S>, TessViewError> {
    let start = *range.start();
    let end = *range.end();
    TessView::inst_slice(self, start, end - start + 1, inst_nb)
  }
}

impl<S> SubTess<S, RangeToInclusive<usize>> for Tess<S>
where
  S: ?Sized + TessBackend,
{
  fn slice(&self, to: RangeToInclusive<usize>) -> Result<TessView<S>, TessViewError> {
    TessView::one_sub(self, to.end + 1)
  }

  fn inst_slice(
    &self,
    to: RangeToInclusive<usize>,
    inst_nb: usize,
  ) -> Result<TessView<S>, TessViewError> {
    TessView::inst_sub(self, to.end + 1, inst_nb)
  }
}
