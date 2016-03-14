//! # Vertices
//!
//! A vertex is a type representing a point. It’s common to find vertex position, normals, colors or
//! even texture coordinates. However, you’re free to use whichever type you want.
//! Nevertheless, you’re limited to a range of types and dimensions. See `VertexComponentType` and
//! `VertexComponentDim` for further details.
//!
//! # `Vertex`
//!
//! ## Rules
//!
//! To be able to use a type as a vertex, you have to implement the `Vertex` trait. That trait
//! represents a mapping between your type and `VertexFormat`. A `VertexFormat` gives runtime hints
//! about your type and restricts the supported type. If you cannot map your type to `VertexFormat`,
//! that means you cannot use it as a `Vertex`.
//!
//! The rule is that your type should have a static size greater than 0 and less than or equal to 4.
//! It should also be either integral, unsigned, floating or boolean. If your type is a complex one
//! – for instance a `struct` – you have to recursively apply that rule to all its fields.
//! For instance, the tuple `(i32,bool)` implements `Vertex` by providing an implementation using
//! the ones of `i32` and `bool`.
//!
//! ## Components list
//!
//! As mentionned above, you can use tuples and structs as `Vertex`. If you look at the definition
//! of `VertexFormat`, you’ll notice that it’s a `Vec<VertexComponentFormat>`. That means simple
//! and primary types map to unit vectors – i.e. their size is 1 – but tuples and structs need
//! several `VertexComponentFormat`s to be represented, hence vectors with sizes greater than 1.
//!
//! # Generic implementation
//!
//! You have `Vertex` implementations for all the primary types that can be mapped to
//! `VertexFormat`. However, as it’s not possible to automatically implement `Vertex` for your
//! structure (yet?), a type is provided to help you design your vertex type so that you’re
//! automatically provided with a `Vertex` implementation: `VertexComponent`.
//!
//! `VertexComponent` is a special type used to represent list of types that all should be `Vertex`.
//! With that in hand, you can easily create `Vertex` types and start using them without even
//! implementing `Vertex`, as long as you use `Vertex` types. Feel free to dig in the
//! `VertexComponent` documentation for further details and examples.
use std::vec::Vec;

/// A `VertexFormat` is a list of `VertexComponentFormat`s.
pub type VertexFormat = Vec<VertexComponentFormat>;

/// A `VertexComponentFormat` gives hints about:
///
/// - the type of the component (`VertexComponentType`);
/// - the dimension of the component (`u8`).
pub struct VertexComponentFormat {
    component_type: VertexComponentType
  , dim: VertexComponentDim
}

/// Possible type of vertex components.
pub enum VertexComponentType {
    Integral
  , Unsigned
  , Floating
	, Boolean
}

/// Possible dimension of vertex components.
pub enum VertexComponentDim {
	  DIM1
	, DIM2
	, DIM3
	, DIM4
}

/// Generic type to represent list of vertex components. You should use that type or tuples to
/// design your vertex types. You can also implement `Vertex` by mapping your internal structs’ to
/// that type or tuples.
///
/// `T` refers to the type of the vertex component and `N` represents the next component.
///
/// The special construct `VertexComponent<T>` can be used to either indicate a single vertex
/// component or the latest vertex component of a list.
///
/// # Examples
///
/// ```
/// type V0 = VertexComponent<f32>; // a single floating value
/// type V1 = VertexComponent<i32, VertexComponent<[f32; 3]>>; // a i32 and three f32
/// ```
pub struct VertexComponent<T, N=()> {
		component: T
	, next: N
}

/// A type that can be used as a `Vertex` has to implement that trait – it must provide a mapping
/// to `VertexFormat`.
///
/// If you’re not sure on how to implement that or if you want to use automatic types, feel free
/// to use the primary supported types and `VertexComponent` or tuples.
pub trait Vertex {
	fn vertex_format() -> VertexFormat;
}

impl Vertex for i32 {
	fn vertex_format() -> VertexFormat {
		vec![ VertexComponentFormat { component_type: VertexComponentType::Integral, dim: VertexComponentDim::DIM1 } ]
	}
}

impl Vertex for [i32; 1] {
	fn vertex_format() -> VertexFormat {
		vec![ VertexComponentFormat { component_type: VertexComponentType::Integral, dim: VertexComponentDim::DIM1 } ]
	}
}

impl Vertex for [i32; 2] {
	fn vertex_format() -> VertexFormat {
		vec![ VertexComponentFormat { component_type: VertexComponentType::Integral, dim: VertexComponentDim::DIM2 } ]
	}
}

impl Vertex for [i32; 3] {
	fn vertex_format() -> VertexFormat {
		vec![ VertexComponentFormat { component_type: VertexComponentType::Integral, dim: VertexComponentDim::DIM3 } ]
	}
}

impl Vertex for [i32; 4] {
	fn vertex_format() -> VertexFormat {
		vec![ VertexComponentFormat { component_type: VertexComponentType::Integral, dim: VertexComponentDim::DIM4 } ]
	}
}

impl Vertex for u32 {
	fn vertex_format() -> VertexFormat {
		vec![ VertexComponentFormat { component_type: VertexComponentType::Unsigned, dim: VertexComponentDim::DIM1 } ]
	}
}

impl Vertex for [u32; 1] {
	fn vertex_format() -> VertexFormat {
		vec![ VertexComponentFormat { component_type: VertexComponentType::Unsigned, dim: VertexComponentDim::DIM1 } ]
	}
}

impl Vertex for [u32; 2] {
	fn vertex_format() -> VertexFormat {
		vec![ VertexComponentFormat { component_type: VertexComponentType::Unsigned, dim: VertexComponentDim::DIM2 } ]
	}
}

impl Vertex for [u32; 3] {
	fn vertex_format() -> VertexFormat {
		vec![ VertexComponentFormat { component_type: VertexComponentType::Unsigned, dim: VertexComponentDim::DIM3 } ]
	}
}

impl Vertex for [u32; 4] {
	fn vertex_format() -> VertexFormat {
		vec![ VertexComponentFormat { component_type: VertexComponentType::Unsigned, dim: VertexComponentDim::DIM4 } ]
	}
}

impl Vertex for f32 {
	fn vertex_format() -> VertexFormat {
		vec![ VertexComponentFormat { component_type: VertexComponentType::Floating, dim: VertexComponentDim::DIM1 } ]
	}
}

impl Vertex for [f32; 1] {
	fn vertex_format() -> VertexFormat {
		vec![ VertexComponentFormat { component_type: VertexComponentType::Floating, dim: VertexComponentDim::DIM1 } ]
	}
}

impl Vertex for [f32; 2] {
	fn vertex_format() -> VertexFormat {
		vec![ VertexComponentFormat { component_type: VertexComponentType::Floating, dim: VertexComponentDim::DIM2 } ]
	}
}

impl Vertex for [f32; 3] {
	fn vertex_format() -> VertexFormat {
		vec![ VertexComponentFormat { component_type: VertexComponentType::Floating, dim: VertexComponentDim::DIM3 } ]
	}
}

impl Vertex for [f32; 4] {
	fn vertex_format() -> VertexFormat {
		vec![ VertexComponentFormat { component_type: VertexComponentType::Floating, dim: VertexComponentDim::DIM4 } ]
	}
}

impl Vertex for bool {
	fn vertex_format() -> VertexFormat {
		vec![ VertexComponentFormat { component_type: VertexComponentType::Boolean, dim: VertexComponentDim::DIM1 } ]
	}
}

impl Vertex for [bool; 1] {
	fn vertex_format() -> VertexFormat {
		vec![ VertexComponentFormat { component_type: VertexComponentType::Boolean, dim: VertexComponentDim::DIM1 } ]
	}
}

impl Vertex for [bool; 2] {
	fn vertex_format() -> VertexFormat {
		vec![ VertexComponentFormat { component_type: VertexComponentType::Boolean, dim: VertexComponentDim::DIM2 } ]
	}
}

impl Vertex for [bool; 3] {
	fn vertex_format() -> VertexFormat {
		vec![ VertexComponentFormat { component_type: VertexComponentType::Boolean, dim: VertexComponentDim::DIM3 } ]
	}
}

impl Vertex for [bool; 4] {
	fn vertex_format() -> VertexFormat {
		vec![ VertexComponentFormat { component_type: VertexComponentType::Boolean, dim: VertexComponentDim::DIM4 } ]
	}
}

impl<T, N> Vertex for VertexComponent<T, N> where T: Vertex, N: Vertex {
	fn vertex_format() -> VertexFormat {
		let mut t = T::vertex_format();
		t.extend(N::vertex_format());
		t
	}
}

impl<A, B> Vertex for (A, B) where A: Vertex, B: Vertex {
	fn vertex_format() -> VertexFormat {
		VertexComponent::<A, B>::vertex_format()
	}
}

impl<A, B, C> Vertex for (A, B, C) where A: Vertex, B: Vertex, C: Vertex {
	fn vertex_format() -> VertexFormat {
		VertexComponent::<A, VertexComponent<B, C>>::vertex_format()
	}
}

impl<A, B, C, D> Vertex for (A, B, C, D) where A: Vertex, B: Vertex, C: Vertex, D: Vertex {
	fn vertex_format() -> VertexFormat {
		VertexComponent::<A, VertexComponent<B, VertexComponent<C, D>>>::vertex_format()
	}
}

impl<A, B, C, D, E> Vertex for (A, B, C, D, E) where A: Vertex, B: Vertex, C: Vertex, D: Vertex, E: Vertex {
	fn vertex_format() -> VertexFormat {
		VertexComponent::<A, VertexComponent<B, VertexComponent<C, VertexComponent<D, E>>>>::vertex_format()
	}
}

impl<A, B, C, D, E, F> Vertex for (A, B, C, D, E, F) where A: Vertex, B: Vertex, C: Vertex, D: Vertex, E: Vertex, F: Vertex {
	fn vertex_format() -> VertexFormat {
		VertexComponent::<A, VertexComponent<B, VertexComponent<C, VertexComponent<D, VertexComponent<E, F>>>>>::vertex_format()
	}
}
