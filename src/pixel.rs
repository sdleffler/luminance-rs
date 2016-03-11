use core::marker::PhantomData;

/// A 8-bit channel.
struct C8;

/// A 16-bit channel.
struct C16;

/// A 32-bit channel.
struct C32;

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
