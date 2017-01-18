//! This module provides texture features.

use gl;
use gl::types::*;
use std::marker::PhantomData;
use std::mem;
use std::os::raw::c_void;
use std::ops::{Deref, DerefMut};
use std::ptr;

use pixel::{Pixel, PixelFormat, opengl_pixel_format, pixel_components};

/// How to wrap texture coordinates while sampling textures?
#[derive(Clone, Copy, Debug)]
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
#[derive(Clone, Copy, Debug)]
pub enum Filter {
  /// Clamp to nearest pixel.
  Nearest,
  /// Linear interpolation with surrounding pixels.
  Linear
}

/// Depth comparison to perform while depth test. `a` is the incoming fragment’s depth and b is the
/// fragment’s depth that is already stored.
#[derive(Clone, Copy, Debug)]
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
  type Offset;

  /// Dimension.
  fn dim() -> Dim;
  /// Width of the associated `Size`.
  fn width(size: Self::Size) -> u32 where Self::Size: Copy;
  /// Height of the associated `Size`. If it doesn’t have one, set it to 1.
  fn height(_: Self::Size) -> u32 where Self::Size: Copy { 1 }
  /// Depth of the associated `Size`. If it doesn’t have one, set it to 1.
  fn depth(_: Self::Size) -> u32 where Self::Size: Copy { 1 }
  /// X offset.
  fn x_offset(offset: Self::Offset) -> u32 where Self::Offset: Copy;
  /// Y offset. If it doesn’t have one, set it to 0.
  fn y_offset(_: Self::Offset) -> u32 where Self::Offset: Copy { 1 }
  /// Z offset. If it doesn’t have one, set it to 0.
  fn z_offset(_: Self::Offset) -> u32 where Self::Offset: Copy { 1 }
  /// Zero offset.
  fn zero_offset() -> Self::Offset;
}

/// Capacity of the dimension, which is the product of the width, height and depth.
pub fn dim_capacity<D>(size: D::Size) -> u32 where D: Dimensionable, D::Size: Copy {
  D::width(size) * D::height(size) * D::depth(size)
}

/// Dimension of a texture.
#[derive(Clone, Copy, Debug)]
pub enum Dim {
  Dim1,
  Dim2,
  Dim3,
  Cubemap
}

/// 1D dimension.
#[derive(Clone, Copy, Debug)]
pub struct Dim1;

impl Dimensionable for Dim1 {
  type Size = u32;
  type Offset = u32;

  fn dim() -> Dim { Dim::Dim1 }

  fn width(w: Self::Size) -> u32 { w }

  fn x_offset(off: Self::Offset) -> u32 { off }

  fn zero_offset() -> Self::Offset { 0 }
}

/// 2D dimension.
#[derive(Clone, Copy, Debug)]
pub struct Dim2;

impl Dimensionable for Dim2 {
  type Size = (u32, u32);
  type Offset = (u32, u32);

  fn dim() -> Dim { Dim::Dim2 }

  fn width(size: Self::Size) -> u32 { size.0 }

  fn height(size: Self::Size) -> u32 { size.1 }

  fn x_offset(off: Self::Offset) -> u32 { off.0 }

  fn y_offset(off: Self::Offset) -> u32 { off.1 }

  fn zero_offset() -> Self::Offset { (0, 0) }
}

/// 3D dimension.
#[derive(Clone, Copy, Debug)]
pub struct Dim3;

impl Dimensionable for Dim3 {
  type Size = (u32, u32, u32);
  type Offset = (u32, u32, u32);

  fn dim() -> Dim { Dim::Dim3 }

  fn width(size: Self::Size) -> u32 { size.0 }

  fn height(size: Self::Size) -> u32 { size.1 }

  fn depth(size: Self::Size) -> u32 { size.2 }

  fn x_offset(off: Self::Offset) -> u32 { off.0 }

  fn y_offset(off: Self::Offset) -> u32 { off.1 }

  fn z_offset(off: Self::Offset) -> u32 { off.2 }

  fn zero_offset() -> Self::Offset { (0, 0, 0) }
}

/// Cubemap dimension.
#[derive(Clone, Copy, Debug)]
pub struct Cubemap;

impl Dimensionable for Cubemap {
  type Size = u32;
  type Offset = (u32, u32, CubeFace);

  fn dim() -> Dim { Dim::Cubemap }

  fn width(s: Self::Size) -> u32 { s }

  fn height(s: Self::Size) -> u32 { s }

  fn depth(_: Self::Size) -> u32 { 6 }

  fn x_offset(off: Self::Offset) -> u32 { off.0 }

  fn y_offset(off: Self::Offset) -> u32 { off.1 }

  fn z_offset(off: Self::Offset) -> u32 {
    match off.2 {
      CubeFace::PositiveX => 0,
      CubeFace::NegativeX => 1,
      CubeFace::PositiveY => 2,
      CubeFace::NegativeY => 3,
      CubeFace::PositiveZ => 4,
      CubeFace::NegativeZ => 5
    }
  }

  fn zero_offset() -> Self::Offset { (0, 0, CubeFace::PositiveX) }
}

/// Faces of a cubemap.
#[derive(Clone, Copy, Debug)]
pub enum CubeFace {
  PositiveX,
  NegativeX,
  PositiveY,
  NegativeY,
  PositiveZ,
  NegativeZ
}

/// Trait used to reify a type into a `Layering`.
pub trait Layerable {
  /// Reify to `Layering`.
  fn layering() -> Layering;
}

/// Texture layering. If a texture is layered, it has an extra coordinate to access the layer.
#[derive(Clone, Copy, Debug)]
pub enum Layering {
  /// Non-layered.
  Flat,
  /// Layered.
  Layered
}

/// Flat texture hint.
///
/// A flat texture means it doesn’t have the concept of layers.
#[derive(Clone, Copy, Debug)]
pub struct Flat;

impl Layerable for Flat { fn layering() -> Layering { Layering::Flat } }

/// Layered texture hint.
///
/// A layered texture has an extra coordinate to access the layer and can be thought of as an array
/// of textures.
#[derive(Clone, Copy, Debug)]
pub struct Layered;

impl Layerable for Layered { fn layering() -> Layering { Layering::Layered } }

/// Texture.
///
/// `L` refers to the layering type; `D` refers to the dimension; `P` is the pixel format for the
/// texels.
#[derive(Debug)]
pub struct Texture<L, D, P> where L: Layerable, D: Dimensionable, P: Pixel {
  handle: GLuint, // handle to the GPU texture object
  target: GLenum, // “type” of the texture; used for bindings
  size: D::Size,
  mipmaps: usize, // number of mipmaps
  _l: PhantomData<L>,
  _p: PhantomData<P>
}

impl<L, D, P> Drop for Texture<L, D, P> where L: Layerable, D: Dimensionable, P: Pixel {
  fn drop(&mut self) {
    unsafe { gl::DeleteTextures(1, &self.handle) }
  }
}

impl<L, D, P> Texture<L, D, P>
    where L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P: Pixel {
  pub fn new(size: D::Size, mipmaps: usize, sampler: &Sampler) -> Result<Self> {
    let mipmaps = mipmaps + 1; // + 1 prevent having 0 mipmaps
    let mut texture = 0;
    let target = opengl_target(L::layering(), D::dim());

    unsafe {
      gl::GenTextures(1, &mut texture);
      gl::BindTexture(target, texture);
    }
    
    create_texture::<L, D>(target, size, mipmaps, P::pixel_format(), sampler)?;

    // FIXME: maybe we can get rid of this
    unsafe {
      gl::BindTexture(target, 0);
    }

    Ok(Texture {
      handle: texture,
      target: target,
      size: size,
      mipmaps: mipmaps,
      _c: PhantomData,
      _l: PhantomData,
      _p: PhantomData
    })
  }

  /// Create a texture from its backend representation.
  pub unsafe fn from_raw(handle: GLuint, target: GLenum, size: D::Size, mipmaps: usize) -> Self {
    Texture {
      handle: handle,
      target: target,
      size: size,
      mipmaps: mipmaps + 1,
      _c: PhantomData,
      _l: PhantomData,
      _p: PhantomData
    }
  }

  /// Clear a part of a texture.
  ///
  /// The part being cleared is defined by a rectangle in which the `offset` represents the
  /// left-upper corner and the `size` gives the dimension of the rectangle. All the covered texels
  /// by this rectangle will be cleared to the `pixel` value.
  pub fn clear_part(&self, gen_mipmaps: bool, offset: D::Offset, size: D::Size, pixel: P::Encoding)
      where D::Offset: Copy,
            D::Size: Copy,
            P::Encoding: Copy {
    self.upload_part::<L, D, P>(gen_mipmaps, offset, size, &vec![pixel; dim_capacity::<D>(size) as usize])
  }

  /// Clear a whole texture with a `pixel` value.
  pub fn clear(&self, gen_mipmaps: bool, pixel: P::Encoding)
      where D::Offset: Copy,
            D::Size: Copy,
            P::Encoding: Copy {
    self.clear_part(gen_mipmaps, D::zero_offset(), self.size, pixel)
  }

  /// Upload texels to a part of a texture.
  ///
  /// The part being updated is defined by a rectangle in which the `offset` represents the
  /// left-upper corner and the `size` gives the dimension of the rectangle. All the covered texels
  /// by this rectangle will be updated by the `texels` slice.
  pub fn upload_part(&self, gen_mipmaps: bool, offset: D::Offset, size: D::Size, texels: &[P::Encoding])
      where D::Offset: Copy,
            D::Size: Copy {
    unsafe {
      gl::BindTexture(self.target, self.handle);

      upload_texels::<L, D, P, P::Encoding>(self.target, offset, size, texels);

      if gen_mipmaps {
        gl::GenerateMipmap(self.target);
      }

      gl::BindTexture(self.target, 0);
    }
  }

  /// Upload `texels` to the whole texture.
  pub fn upload(&self, gen_mipmaps: bool, texels: &[P::Encoding])
      where D::Offset: Copy,
            D::Size: Copy {
    self.upload_part(gen_mipmaps, D::zero_offset(), self.size, texels)
  }

  /// Upload raw `texels` to a part of a texture.
  ///
  /// This function is similar to `upload_part` but it works on `P::RawEncoding` instead of
  /// `P::Encoding`. This useful when the texels are represented as a contiguous array of raw
  /// components of the texels.
  pub fn upload_part_raw(&self, gen_mipmaps: bool, offset: D::Offset, size: D::Size, texels: &[P::RawEncoding])
      where D::Offset: Copy,
            D::Size: Copy {
    unsafe {
      gl::BindTexture(self.target, self.handle);

      upload_texels::<L, D, P, P::RawEncoding>(self.target, offset, size, texels);

      if gen_mipmaps {
        gl::GenerateMipmap(self.target);
      }

      gl::BindTexture(self.target, 0);
    }
  }

  /// Upload raw `texels` to the whole texture.
  pub fn upload_raw(&self, gen_mipmaps: bool, texels: &[P::RawEncoding])
      where D::Offset: Copy,
            D::Size: Copy {
    self.upload_part_raw(gen_mipmaps, D::zero_offset(), self.size, texels)
  }

  // FIXME: cubemaps?
  /// Get the raw texels associated with this texture.
  pub fn get_raw_texels(&self) -> Vec<P::RawEncoding> where P: Pixel, P::RawEncoding: Copy {
    let mut texels = Vec::new();
    let pf = P::pixel_format();
    let (format, _, ty) = opengl_pixel_format(pf).unwrap();

    unsafe {
      let mut w = 0;
      let mut h = 0;

      gl::BindTexture(self.target, self.handle);

      // retrieve the size of the texture (w and h)
      gl::GetTexLevelParameteriv(self.target, 0, gl::TEXTURE_WIDTH, &mut w);
      gl::GetTexLevelParameteriv(self.target, 0, gl::TEXTURE_HEIGHT, &mut h);

      // resize the vec to allocate enough space to host the returned texels
      texels.resize((w * h) as usize * pixel_components(pf), mem::uninitialized());

      gl::GetTexImage(self.target, 0, format, ty, texels.as_mut_ptr() as *mut c_void);

      gl::BindTexture(self.target, 0);
    }

    texels
  }
}

fn opengl_target(l: Layering, d: Dim) -> GLenum {
  match l {
    Layering::Flat => match d {
      Dim::Dim1 => gl::TEXTURE_1D,
      Dim::Dim2 => gl::TEXTURE_2D,
      Dim::Dim3 => gl::TEXTURE_3D,
      Dim::Cubemap => gl::TEXTURE_CUBE_MAP
    },
    Layering::Layered => match d {
      Dim::Dim1 => gl::TEXTURE_1D_ARRAY,
      Dim::Dim2 => gl::TEXTURE_2D_ARRAY,
      Dim::Dim3 => panic!("3D textures array not supported"),
      Dim::Cubemap => gl::TEXTURE_CUBE_MAP_ARRAY
    }
  }
}

fn create_texture<L, D>(target: GLenum, size: D::Size, mipmaps: usize, pf: PixelFormat, sampler: &Sampler) -> Result<()>
    where L: Layerable,
          D: Dimensionable,
          D::Size: Copy {
  set_texture_levels(target, mipmaps);
  apply_sampler_to_texture(target, sampler);
  create_texture_storage::<L, D>(size, mipmaps, pf)
}

fn create_texture_storage<L, D>(size: D::Size, mipmaps: usize, pf: PixelFormat) -> Result<()>
    where L: Layerable,
          D: Dimensionable,
          D::Size: Copy {
  match opengl_pixel_format(pf) {
    Some(glf) => {
      let (format, iformat, encoding) = glf;

      match (L::layering(), D::dim()) {
        // 1D texture
        (Layering::Flat, Dim::Dim1) => {
          create_texture_1d_storage(format, iformat, encoding, D::width(size), mipmaps);
          Ok(())
        },
        // 2D texture
        (Layering::Flat, Dim::Dim2) => {
          create_texture_2d_storage(format, iformat, encoding, D::width(size), D::height(size), mipmaps);
          Ok(())
        },
        // 3D texture
        (Layering::Flat, Dim::Dim3) => {
          create_texture_3d_storage(format, iformat, encoding, D::width(size), D::height(size), D::depth(size), mipmaps);
          Ok(())
        },
        // cubemap
        (Layering::Flat, Dim::Cubemap) => {
          create_cubemap_storage(format, iformat, encoding, D::width(size), mipmaps);
          Ok(())
        },
        _ => Err(TextureError::TextureStorageCreationFailed(format!("unsupported texture OpenGL pixel format: {:?}", glf)))
      }
    },
    None => Err(TextureError::TextureStorageCreationFailed(format!("unsupported texture pixel format: {:?}", pf)))
  }
}

fn create_texture_1d_storage(format: GLenum, iformat: GLenum, encoding: GLenum, w: u32, mipmaps: usize) {
  for level in 0..mipmaps {
    let w = w / 2u32.pow(level as u32);

    unsafe { gl::TexImage1D(gl::TEXTURE_1D, level as GLint, iformat as GLint, w as GLsizei, 0, format, encoding, ptr::null()) };
  }
}

fn create_texture_2d_storage(format: GLenum, iformat: GLenum, encoding: GLenum, w: u32, h: u32, mipmaps: usize) {
  for level in 0..mipmaps {
    let div = 2u32.pow(level as u32);
    let w = w / div;
    let h = h / div;

    unsafe { gl::TexImage2D(gl::TEXTURE_2D, level as GLint, iformat as GLint, w as GLsizei, h as GLsizei, 0, format, encoding, ptr::null()) };
  }
}

fn create_texture_3d_storage(format: GLenum, iformat: GLenum, encoding: GLenum, w: u32, h: u32, d: u32, mipmaps: usize) {
  for level in 0..mipmaps {
    let div = 2u32.pow(level as u32);
    let w = w / div;
    let h = h / div;
    let d = d / div;

    unsafe { gl::TexImage3D(gl::TEXTURE_3D, level as GLint, iformat as GLint, w as GLsizei, h as GLsizei, d as GLsizei, 0, format, encoding, ptr::null()) };
  }
}

fn create_cubemap_storage(format: GLenum, iformat: GLenum, encoding: GLenum, s: u32, mipmaps: usize) {
  for level in 0..mipmaps {
    let s = s / 2u32.pow(level as u32);

    unsafe { gl::TexImage2D(gl::TEXTURE_CUBE_MAP, level as GLint, iformat as GLint, s as GLsizei, s as GLsizei, 0, format, encoding, ptr::null()) };
  }
}

fn set_texture_levels(target: GLenum, mipmaps: usize) {
  unsafe {
    gl::TexParameteri(target, gl::TEXTURE_BASE_LEVEL, 0);
    gl::TexParameteri(target, gl::TEXTURE_MAX_LEVEL, mipmaps as GLint - 1);
  }
}

fn apply_sampler_to_texture(target: GLenum, sampler: &Sampler) {
  unsafe {
    gl::TexParameteri(target, gl::TEXTURE_WRAP_R, opengl_wrap(sampler.wrap_r) as GLint);
    gl::TexParameteri(target, gl::TEXTURE_WRAP_S, opengl_wrap(sampler.wrap_s) as GLint);
    gl::TexParameteri(target, gl::TEXTURE_WRAP_T, opengl_wrap(sampler.wrap_t) as GLint);
    gl::TexParameteri(target, gl::TEXTURE_MIN_FILTER, opengl_filter(sampler.minification) as GLint);
    gl::TexParameteri(target, gl::TEXTURE_MAG_FILTER, opengl_filter(sampler.minification) as GLint);
    match sampler.depth_comparison {
      Some(fun) => {
        gl::TexParameteri(target, gl::TEXTURE_COMPARE_FUNC, opengl_depth_comparison(fun) as GLint);
        gl::TexParameteri(target, gl::TEXTURE_COMPARE_MODE, gl::COMPARE_REF_TO_TEXTURE as GLint);
      },
      None => {
        gl::TexParameteri(target, gl::TEXTURE_COMPARE_MODE, gl::NONE as GLint);
      }
    }
  }
}

fn opengl_wrap(wrap: Wrap) -> GLenum {
  match wrap {
    Wrap::ClampToEdge => gl::CLAMP_TO_EDGE,
    Wrap::Repeat => gl::REPEAT,
    Wrap::MirroredRepeat => gl::MIRRORED_REPEAT
  }
}

fn opengl_filter(filter: Filter) -> GLenum {
  match filter {
    Filter::Nearest => gl::NEAREST,
    Filter::Linear => gl::LINEAR
  }
}

fn opengl_depth_comparison(fun: DepthComparison) -> GLenum {
  match fun {
    DepthComparison::Never => gl::NEVER,
    DepthComparison::Always => gl::ALWAYS,
    DepthComparison::Equal => gl::EQUAL,
    DepthComparison::NotEqual => gl::NOTEQUAL,
    DepthComparison::Less => gl::LESS,
    DepthComparison::LessOrEqual => gl::LEQUAL,
    DepthComparison::Greater => gl::GREATER,
    DepthComparison::GreaterOrEqual => gl::GEQUAL
  }
}

// Upload texels into the texture’s memory. Becareful of the type of texels you send down.
fn upload_texels<L, D, P, T>(target: GLenum, off: D::Offset, size: D::Size, texels: &[T])
    where L: Layerable,
          D: Dimensionable,
          D::Offset: Copy,
          D::Size: Copy,
          P: Pixel {
  let pf = P::pixel_format();

  match opengl_pixel_format(pf) {
    Some((format, _, encoding)) => {
      match L::layering() {
        Layering::Flat => {
          match D::dim() {
            Dim::Dim1 => unsafe { gl::TexSubImage1D(target, 0, D::x_offset(off) as GLint, D::width(size) as GLsizei, format, encoding, texels.as_ptr() as *const c_void) },
            Dim::Dim2 => unsafe { gl::TexSubImage2D(target, 0, D::x_offset(off) as GLint, D::y_offset(off) as GLint, D::width(size) as GLsizei, D::height(size) as GLsizei, format, encoding, texels.as_ptr() as *const c_void) },
            Dim::Dim3 => unsafe { gl::TexSubImage3D(target, 0, D::x_offset(off) as GLint, D::y_offset(off) as GLint, D::z_offset(off) as GLint, D::width(size) as GLsizei, D::height(size) as GLsizei, D::depth(size) as GLsizei, format, encoding, texels.as_ptr() as *const c_void) },
            Dim::Cubemap => unsafe { gl::TexSubImage3D(target, 0, D::x_offset(off) as GLint, D::y_offset(off) as GLint, (gl::TEXTURE_CUBE_MAP_POSITIVE_X + D::z_offset(off)) as GLint, D::width(size) as GLsizei, D::width(size) as GLsizei, 1, format, encoding, texels.as_ptr() as *const c_void) }
          }
        },
        Layering::Layered => panic!("Layering::Layered not implemented yet")
      }
    },
    None => panic!("unknown pixel format")
  }
}

/// A `Sampler` object gives hint on how a `Texture` should be sampled.
#[derive(Clone, Copy, Debug)]
pub struct Sampler {
  /// How should we wrap around the *r* sampling coordinate?
  pub wrap_r: Wrap,
  /// How should we wrap around the *s* sampling coordinate?
  pub wrap_s: Wrap,
  /// How should we wrap around the *t* sampling coordinate?
  pub wrap_t: Wrap,
  /// Minification filter.
  pub minification: Filter,
  /// Magnification filter.
  pub magnification: Filter,
  /// For depth textures, should we perform depth comparison and if so, how?
  pub depth_comparison: Option<DepthComparison>
}

/// Default value is as following:
///
/// ```
/// Sampler {
///   wrap_r: Wrap::ClampToEdge,
///   wrap_s: Wrap::ClampToEdge,
///   wrap_t: Wrap::ClampToEdge,
///   minification: Filter::Linear,
///   magnification: Filter::Linear,
///   depth_comparison: None
/// }
/// ```
impl Default for Sampler {
  fn default() -> Self {
    Sampler {
      wrap_r: Wrap::ClampToEdge,
      wrap_s: Wrap::ClampToEdge,
      wrap_t: Wrap::ClampToEdge,
      minification: Filter::Linear,
      magnification: Filter::Linear,
      depth_comparison: None
    }
  }
}

/// Texture unit.
pub struct Unit {
  unit: u32
}

impl Unit {
  pub fn new(unit: u32) -> Self {
    Unit {
      unit: unit
    }
  }
}

impl Deref for Unit {
  type Target = u32;

  fn deref(&self) -> &Self::Target {
    &self.unit
  }
}

impl DerefMut for Unit {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.unit
  }
}

/// An opaque type representing any texture.
pub struct TextureProxy<'a> {
  pub repr: &'a Texture
}

impl<'a, L, D, P> From<&'a Texture<L, D, P>> for TextureProxy<'a>
    where L: Layerable,
          D: Dimensionable,
          D::Size: Copy,
          P: Pixel {
  fn from(texture: &'a Texture<L, D, P>) -> Self {
    TextureProxy {
      repr: &texture
    }
  }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TextureError {
  TextureCreationFailed(String),
  TextureStorageCreationFailed(String),
}

pub type Result<T> = ::std::result::Result<T, TextureError>;
