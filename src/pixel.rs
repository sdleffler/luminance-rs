//! Pixel formats types and function manipulation.
//!
//! The `Pixel` trait is used to reify a pixel type at runtime via `PixelFormat`.

/// Reify a static pixel format to runtime.
pub trait Pixel {
	type Encoding;

  fn pixel_format() -> PixelFormat;
}

/// A `PixelFormat` gathers a `Type` along with a `Format`.
pub struct PixelFormat {
    pub encoding_type: Type
  , pub format: Format
}

/// Pixel type.
pub enum Type {
  Integral,
  Unsigned,
  Floating
}

/// Format of a pixel.
///
/// Whichever the constructor you choose, the carried `u8`s represents how many bytes are used to
/// represent each channel.
pub enum Format {
    /// Holds a red-only channel.
  R(u8),
    /// Holds red and green channels. 
  RG(u8, u8),
  /// Holds red, green and blue channels.
  RGB(u8, u8, u8),
  /// Holds red, green, blue and alpha channels.
  RGBA(u8, u8, u8, u8),
  /// Holds a depth channel.
  Depth(u8)
}

/// Does a `PixelFormat` represent a color?
pub fn is_color_pixel(f: PixelFormat) -> bool {
  match f.format {
      Format::Depth(_) => false
    , _ => true
  }
}

/// Does a `PixelFormat` represent depth information?
pub fn is_depth_pixel(f: PixelFormat) -> bool {
  !is_color_pixel(f)
}

/// A red, green and blue 8-bit unsigned pixel format.
pub struct RGB8UI;

impl Pixel for RGB8UI {
	type Encoding = (u8, u8, u8);

  fn pixel_format() -> PixelFormat {
    PixelFormat {
        encoding_type: Type::Unsigned
      , format: Format::RGB(8, 8, 8)
    }
  }
}

/// A red, green, blue and alpha 8-bit unsigned pixel format.
pub struct RGBA8UI;

impl Pixel for RGBA8UI {
	type Encoding = (u8, u8, u8, u8);

  fn pixel_format() -> PixelFormat {
    PixelFormat {
        encoding_type: Type::Unsigned
      , format: Format::RGBA(8, 8, 8, 8)
    }
  }
}

/// A red, green and blue 32-bit floating pixel format.
pub struct RGB32F;

impl Pixel for RGB32F {
	type Encoding = (f32, f32, f32);

  fn pixel_format() -> PixelFormat {
    PixelFormat {
        encoding_type: Type::Floating
      , format: Format::RGB(32, 32, 32)
    }
  }
}

/// A red, green, blue and alpha 32-bit floating pixel format.
pub struct RGBA32F;

impl Pixel for RGBA32F {
	type Encoding = (f32, f32, f32, f32);

  fn pixel_format() -> PixelFormat {
    PixelFormat {
        encoding_type: Type::Floating
      , format: Format::RGBA(32, 32, 32, 32)
    }
  }
}

/// A depth 32-bit floating pixel format.
pub struct Depth32F;

impl Pixel for Depth32F {
	type Encoding = f32;

  fn pixel_format() -> PixelFormat {
    PixelFormat {
        encoding_type: Type::Floating
      , format: Format::Depth(32)
    }
  }
}
