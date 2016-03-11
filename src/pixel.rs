/// Reify a static pixel format to runtime.
pub trait Pixel {
  fn pixel_format() -> PixelFormat;
}

/// A `PixelFormat` gathers a `Type` along with a `Format`.
pub struct PixelFormat {
    encoding_type: Type
  , format: Format
}

/// Pixel type.
pub enum Type {
    Integral
  , Unsigned
  , Floating
}

/// Format of a pixel.
pub enum Format {
    R(u8)
  , RG(u8, u8)
  , RGB(u8, u8, u8)
  , RGBA(u8, u8, u8, u8)
  , Depth(u8)
}

pub fn is_color_pixel(f: PixelFormat) -> bool {
  match f.format {
      Format::Depth(_) => false
    , _ => true
  }
}

pub fn is_depth_pixel(f: PixelFormat) -> bool {
  !is_color_pixel(f)
}

pub struct RGB8UI;

impl Pixel for RGB8UI {
  fn pixel_format() -> PixelFormat {
    PixelFormat {
        encoding_type: Type::Unsigned
      , format: Format::RGB(8, 8, 8)
    }
  }
}

pub struct RGBA8UI;

impl Pixel for RGBA8UI {
  fn pixel_format() -> PixelFormat {
    PixelFormat {
        encoding_type: Type::Unsigned
      , format: Format::RGBA(8, 8, 8, 8)
    }
  }
}

pub struct RGB8F;

impl Pixel for RGB8F {
  fn pixel_format() -> PixelFormat {
    PixelFormat {
        encoding_type: Type::Floating
      , format: Format::RGB(8, 8, 8)
    }
  }
}

pub struct RGBA8F;

impl Pixel for RGBA8F {
  fn pixel_format() -> PixelFormat {
    PixelFormat {
        encoding_type: Type::Floating
      , format: Format::RGBA(8, 8, 8, 8)
    }
  }
}

pub struct RGB32F;

impl Pixel for RGB32F {
  fn pixel_format() -> PixelFormat {
    PixelFormat {
        encoding_type: Type::Floating
      , format: Format::RGB(32, 32, 32)
    }
  }
}

pub struct RGBA32F;

impl Pixel for RGBA32F {
  fn pixel_format() -> PixelFormat {
    PixelFormat {
        encoding_type: Type::Floating
      , format: Format::RGBA(32, 32, 32, 32)
    }
  }
}

pub struct Depth32F;

impl Pixel for Depth32F {
  fn pixel_format() -> PixelFormat {
    PixelFormat {
        encoding_type: Type::Floating
      , format: Format::Depth(32)
    }
  }
}
