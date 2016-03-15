use core::marker::PhantomData;
use pixel::Pixel;

/// How to wrap texture coordinates while sampling textures?
pub enum Wrap {
  /// If textures coordinates lay outside of *[0;1]*, they will be clamped to either *0* or *1* for
  /// every components.
  ClampToEdge,
  /// Textures coordinates are repeated if they lay outside of *[0;1]*. Picture this as:
  ///
  /// ```
  /// // given the frac function returning the fractional part of a floating number:
  /// coord_ith = frac(coord_ith); // always between [0;1]
  /// ```
  Repeat,
  /// Same as `Repeat` but it will alternatively repeat between *[0;1]* and *[1;0]*.
  MirroredRepeat
}

/// Minification and magnification filter.
pub enum Filter {
  /// Clamp to nearest pixel.
  Nearest,
  /// Linear interpolation with surrounding pixels.
  Linear
}

/// Depth comparison to perform while depth test. `a` is the incoming fragment’s depth and b is the
/// fragment’s depth that is already stored.
pub enum DepthComparison {
  /// Depth test never succeeds.
  Never,
  /// Depth test always succeeds.
  Always,
  /// Depth test succeeds if `a == b`.
  Equal,
  /// Depth test succeeds if `a != b`.
  NotEqual,
  /// Depth test succeeds if `a < b`.
  Less,
  /// Depth test succeeds if `a <= b`.
  LessOrEqual,
  /// Depth test succeeds if `a > b`.
  Greater,
  /// Depth test succeeds if `a >= b`.
  GreaterOrEqual
}

/// Reify a type into a `Dim`.
pub trait Dimensionable {
  type Size;

  fn dim() -> Dim;
}

/// Dimension of a texture.
pub enum Dim {
  DIM1,
  DIM2,
  DIM3,
  Cubemap
}

pub struct DIM1;

impl Dimensionable for DIM1 {
  type Size = u32;

  fn dim() -> Dim { Dim::DIM1 }
}

pub struct DIM2;

impl Dimensionable for DIM2 {
  type Size = (u32, u32);

  fn dim() -> Dim { Dim::DIM2 }
}

pub struct DIM3;

impl Dimensionable for DIM3 {
  type Size = (u32, u32, u32);

  fn dim() -> Dim { Dim::DIM3 }
}

pub struct Cubemap;

impl Dimensionable for Cubemap {
  type Size = (u32, u32, u32);

  fn dim() -> Dim { Dim::Cubemap }
}

/// Reify a type into a `Layering`.
pub trait Layerable {
  fn layering() -> Layering;
}

/// Texture layering. If a texture is layered, it has an extra coordinates to access the layer.
pub enum Layering {
  /// Non-layered.
  Flat,
  /// Layered.
  Layered
}

pub struct Flat;

impl Layerable for Flat { fn layering() -> Layering { Layering::Flat } }

pub struct Layered;

impl Layerable for Layered { fn layering() -> Layering { Layering::Layered } }

/// Trait to implement to provide texture features.
pub trait HasTexture {
  type ATex;

  fn new<L, D, P>(size: D::Size, mipmaps: u32, sampling: ()) -> Self::ATex
    where L: Layerable,
          D: Dimensionable,
          P: Pixel;
}

pub struct Tex<C, L, D, P> where C: HasTexture, L: Layerable, D: Dimensionable, P: Pixel {
  pub repr: C::ATex,
  pub size: D::Size,
  pub mipmaps: u32,
  _l: PhantomData<L>,
  _c: PhantomData<C>,
  _p: PhantomData<P>
}

impl<C, L, D, P> Tex<C, L, D, P>
    where C: HasTexture,
          L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P: Pixel {
  pub fn new(size: D::Size, mipmaps: u32, sampling: ()) -> Self {
    let tex = C::new::<L, D, P>(size, mipmaps, sampling);
    Tex { repr: tex, size: size, mipmaps: mipmaps, _c: PhantomData, _l: PhantomData, _p: PhantomData }
  }
}
