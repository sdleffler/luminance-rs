//! Backend interfacing.
//!
//! Almost everything declared in this module and its submodules is `unsafe`. An end-user **is not
//! supposed to implement any of this.** Library authors might use some traits from here, required
//! by generic code, but no one but backend authors should implement any symbols from here.
//!
//! # Conventions
//!
//! ## Constrained types
//!
//! When a public symbol, like [`Buffer`], has type variables and the first one is `B`, it’s likely
//! to be the _backend type_. Very often, that type will be constrained by a trait. For instance,
//! for [`Buffer`], it is constrained by [`BufferBackend`]. That trait will provide the interface
//! that backends need to implement to support the API type — in our case, [`Buffer`]. Implementing
//! such traits is `unsafe` and using their methods is `unsafe` too.
//!
//! ## Associated types and representation objects
//!
//! You will notice, if you have a look at backend traits, that most of them have associated types.
//! Most of them end with the `Repr` suffix. Those types are the concrete _representation_ of the
//! general concept. For [`Buffer`], [`BufferBackend::BufferRepr`] must be provided by a backend
//! and used with the rest of the methods of the [`BufferBackend`] trait. As with `unsafe` methods,
//! accessing a `Repr` object is not possible from the public interface — if you have found a way
//! to do without `unsafe`, this is a bug: please consider filing an issue.
//!
//! ## Relationship to GraphicsContext
//!
//! The [`GraphicsContext`] is an important trait as it is the parent trait of _platform backends_ —
//! i.e. windowing. It makes it possible to use luminance with a wide variety of technologies,
//! systems and platforms.
//!
//! On the other side, traits defined in this module — and its submodules — are the traits
//! representing all concrete implementations of the code people will write with luminance. They
//! contain the concrete implementation of what it means to create a new [`Buffer`] or set
//! a value at a given index in it, for instance.
//!
//! [`GraphicsContext`] has an associated type, [`GraphicsContext::Backend`], that maps the
//! type implementing [`GraphicsContext`] to a backend type. The graphics context doesn’t have to
//! ship its backend, as they’re loosely coupled: it’s possible to write a backend implementation
//! and use / share it as [`GraphicsContext::Backend`] in several system backends.
//!
//! [`GraphicsContext::Backend`] is surjective — i.e. all backends have a [`GraphicsContext`]
//! mapped, which means that some backends are available via different graphics contexts. The
//! implication is that:
//!
//! - Given a [`GraphicsContext`], you immediately know its associated backend.
//! - Give a backend, there is no unique [`GraphicsContext`] you can map backwards, because
//!   several [`GraphicsContext`] might use that backend.
//!
//! That property allows to write a backend type and use it in several graphics contexts.
//!
//! ## What should a backend crate expose?
//!
//! If you would like to implement your own backend, you must implement all the traits defined in
//! this module — and its submodules. Your crate should then only expose a type — the backend
//! type — and make it available to pick by end-users. The [`GraphicsContext::Backend`] associated
//! type makes a strong contract to find all the other types you will be using in your crate, so you
//! don’t have to worry too much about them.
//!
//! > Note: however, when implementing a public trait, all associated types must be `pub` too. So
//! > it’s likely you will have to declare them `pub`, too.
//!
//! [`Buffer`]: crate::buffer::Buffer
//! [`BufferBackend`]: crate::backend::buffer::Buffer
//! [`BufferBackend::BufferRepr`]: crate::backend::buffer::Buffer::BufferRepr
//! [`GraphicsContext`]: crate::context::GraphicsContext
//! [`GraphicsContext::Backend`]: crate::context::GraphicsContext::Backend

#![allow(missing_docs)]

pub mod color_slot;
pub mod depth_slot;
pub mod framebuffer;
pub mod pipeline;
pub mod query;
pub mod render_gate;
pub mod shader;
pub mod shading_gate;
pub mod tess;
pub mod tess_gate;
pub mod texture;
