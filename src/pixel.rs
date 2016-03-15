//! Pixel formats types and function manipulation.
//!
//! The `Pixel` trait is used to reify a pixel type at runtime via `PixelFormat`.

/// Reify a static pixel format to runtime.
pub trait Pixel {
  fn pixel_format() -> PixelFormat;
}

/// A `PixelFormat` gathers a `Type` along with a `Format`.
pub struct PixelFormat {
    pub encoding_type: Type
  , pub format: Format
}

/// Pixel type.
pub enum Type {
    Integral
  , Unsigned
  , Floating
}

/// Format of a pixel.
///
/// The `R` constructor holds a red-only channel.
///
/// The `RG` constructor holds red and green channels. 
///
/// The `RGB` constructor holds red, green and blue channels.
///
/// The `RGBA` constructor holds red, green, blue and alpha channels.
///
/// The `Depth` constructor holds a depth channel.
///
/// Whichever the constructor you choose, the carried `u8`s represents how many bytes are used to
/// represent each channel.
pub enum Format {
    R(u8)
  , RG(u8, u8)
  , RGB(u8, u8, u8)
  , RGBA(u8, u8, u8, u8)
  , Depth(u8)
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
  fn pixel_format() -> PixelFormat {
    PixelFormat {
        encoding_type: Type::Unsigned
      , format: Format::RGBA(8, 8, 8, 8)
    }
  }
}

/// A red, green and blue 8-bit floating pixel format.
pub struct RGB8F;

impl Pixel for RGB8F {
  fn pixel_format() -> PixelFormat {
    PixelFormat {
        encoding_type: Type::Floating
      , format: Format::RGB(8, 8, 8)
    }
  }
}

/// A red, green, blue and alpha 8-bit floating pixel format.
pub struct RGBA8F;

impl Pixel for RGBA8F {
  fn pixel_format() -> PixelFormat {
    PixelFormat {
        encoding_type: Type::Floating
      , format: Format::RGBA(8, 8, 8, 8)
    }
  }
}

/// A red, green and blue 32-bit floating pixel format.
pub struct RGB32F;

impl Pixel for RGB32F {
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
  fn pixel_format() -> PixelFormat {
    PixelFormat {
        encoding_type: Type::Floating
      , format: Format::Depth(32)
    }
  }
}
