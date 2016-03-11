use core::marker::PhantomData;


/// Reify a static pixel format to runtime.
trait Pixel {
	fn pixel_format() -> PixelFormat;
}

/// A `PixelFormat` gathers a `Type` along with a `Format`.
struct PixelFormat {
		encoding_type: Type
	, format: Format
}

/// Pixel type.
enum Type {
	  Integral
	, Unsigned
	, Floating
}

/// Format of a pixel.
enum Format {
		R(u8)
	, RG(u8, u8)
	, RGB(u8, u8, u8)
	, RGBA(u8, u8, u8, u8)
	, Depth(u8)
}

struct RGB8UI;

impl Pixel for RGB8UI {
	fn pixel_format() -> PixelFormat { 
		PixelFormat {
				encoding_type: Type::Unsigned
			, format: Format::RGB(8, 8, 8)
		}
	}
}

struct RGBA8UI;

impl Pixel for RGBA8UI {
	fn pixel_format() -> PixelFormat { 
		PixelFormat {
				encoding_type: Type::Unsigned
			, format: Format::RGBA(8, 8, 8, 8)
		}
	}
}

struct RGB8F;

impl Pixel for RGB8F {
	fn pixel_format() -> PixelFormat { 
		PixelFormat {
				encoding_type: Type::Floating
			, format: Format::RGB(8, 8, 8)
		}
	}
}

struct RGBA8F;

impl Pixel for RGBA8F {
	fn pixel_format() -> PixelFormat { 
		PixelFormat {
				encoding_type: Type::Floating
			, format: Format::RGBA(8, 8, 8, 8)
		}
	}
}
