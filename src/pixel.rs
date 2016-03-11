use core::marker::PhantomData;

/// Leverage a static channel size to runtime.
trait ChanSize {
	fn chan_size() -> u8;
}

/// Leverage a static channel type to runtime.
trait ChanType<T> {
	fn chan_type() -> Type;
}

/// Channel type.
enum Type {
	  Integral
	, Unsigned
	, Floating
}

/// A 8-bit channel.
struct C8;

impl ChanSize for C8 {
	fn chan_size() -> u8 { 8 }
}

/// A 16-bit channel.
struct C16;

impl ChanSize for C16 {
	fn chan_size() -> u8 { 16 }
}

/// A 32-bit channel.
struct C32;

impl ChanSize for C32 {
	fn chan_size() -> u8 { 32 }
}

/// Integral channel.
struct Integral;

/// Unsigned integral channel.
struct Unsigned;

/// Floating channel.
struct Floating;

/// Depth channel.
struct Depth<D> {
    _d: PhantomData<D>
}

type RGB8UI = (Unsigned, C8, C8, C8);
type RGBA8UI = (Unsigned, C8, C8, C8, C8);
type RGB8F = (Floating, C8, C8, C8);
type RGBA8F = (Floating, C8, C8, C8, C8);
type RGB32F = (Floating, C32, C32, C32);
type RGBA32F = (Floating, C32, C32, C32, C32);
type Depth32F = (Floating, Depth<C32>);
