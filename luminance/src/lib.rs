//! # A simple, type-safe and opinionated graphics crate
//!
//! luminance is an effort to make graphics rendering simple and elegant. It is a _low-level_
//! and opinionated graphics API, highly typed (type-level computations, refined types, etc.)
//! which aims to be simple and performant. Instead of providing users with as many low-level
//! features as possible, luminance provides you with _some ways_ to do rendering. That has
//! both advantages and drawbacks:
//!
//! - On one side, because the API is opinionated, some dynamic branching and decisions are
//!   completely removed / optimized. Some operations breaking state mutations or invariant
//!   violation are not statically constructible, ensuring safety. Because strong typing is
//!   used, lots of runtime checks are also not needed, helping with performance.
//! - On the other side, if you want to do something very specific and very low-level, you
//!   will find luminance not to be friendly as it doesn’t like, most of the time, exposing
//!   its internal design to the outer world — mostly for runtime safety reason.
//!
//! > A note on _safety_: here, _safety_ is not used as with the Rust definiton, but most in
//! > terms of undefined behavior and unwanted behavior. If something can lead to a weird
//! > behavior, a crash, a panic or a black screen, it’s considered `unsafe`. That definition
//! > obviously includes the Rust definiton of safety — memory safety.
//!
//! # Feature flags
//!
//! None so far.
//!
//! # What’s included?
//!
//! luminance is a rendering crate, not a 3D engine nor a video game framework. As so, it doesn’t
//! include specific concepts, such as lights, materials, asset management nor scene description. It
//! only provides a rendering library you can plug in whatever you want to.
//!
//! > There are several so-called 3D-engines out there on [crates.io](https://crates.io). Feel free
//! > to have a look around.
//!
//! However, luminance comes with several interesting features:
//!
//! - **Buffers**: buffers are ways to communicate with the GPU; they represent regions of memory
//!   you can write to and read from. There’re several kinds of buffers you can create, among
//!   *vertex and index buffers*, *uniform buffers*, and so on and so forth…. They look like
//!   regular array but have some differences you might be aware of. Most of the time, you will
//!   use them to customize behaviors in shader stages.
//! - **Framebuffers**: framebuffers are used to hold renders. Each time you want to perform a
//!   render, you need to perform it into a framebuffer. Framebuffers can then be combined with
//!   each other to produce effects and design render layers — this is called compositing.
//! - **Shaders**: luminance supports five kinds of shader stages:
//!     - Vertex shaders.
//!     - Tessellation control shaders.
//!     - Tessellation evaluation shaders.
//!     - Geometry shaders.
//!     - Fragment shaders.
//! - **Vertices, indices, primitives and tessellations**: those are used to define a shape you
//!   can render into a framebuffer with a shader. They are mandatory when it comes to rendering.
//!   Even if you don’t need vertex data, you still need tessellations to issue draw calls.
//! - **Textures**: textures represent information packed into arrays on the GPU, and can be used
//!   to customize a visual aspect or pass information around in shaders. They come in several
//!   flavours — e.g. 1D, 2D, cube maps, etc.
//! - **Control on the render state**: the render state is a set of capabilities you can tweak
//!   to draw frames. It includes:
//!     - The blending equation and factors. Blending is the process of taking two colors from two
//!       framebuffers and mixing them.
//!     - Whether we should have a depth test performed.
//!     - Face culling.
//!     - Etc.
//! - And a lot of other cool things like *GPU commands*, *pipelines*, *uniform interfaces* and so
//!   on…
//!
//! # How to dig in?
//!
//! luminance is written to be fairly simple. There are several ways to learn how to use luminance:
//!
//! - The [online documentation](https://docs.rs/luminance) is a mandatory start for newcomers.
//! - The [“Learn luminance” book](https://rust-tutorials.github.io/learn-luminance). Ideal for
//!   newcomers as well as people already used to luminance, as it’s always updated to the latest
//!   version — you might learn new things!
//! - The [luminance-examples](https://github.com/phaazon/luminance-rs/tree/master/luminance-examples)
//!   project. It contains lots of examples describing how to do specifics things. Not adapted for
//!   newcomers, you will likely want to consult those examples if you’re already familiar with
//!   graphics programing and to look for how to do a specific thing.
//!
//! # Implementation and architecture
//!
//! **luminance** has been originally designed around the OpenGL 3.3 and OpenGL 4.5 APIs. However,
//! it has mutated to adapt to new technologies and modern graphics programming. Even though its API
//! is _not_ meant to converge towards something like Vulkan, it’s changing over time to meet
//! better design decisions and performance implications.
//!
//! The current state of luminance comprises several crates:
//!
//! - A “core” crate, [luminance], which is about all the
//!   abstract, common and interface code.
//! - A set of _backend implementation_ crates, implementing the [luminance] crate.
//! - A set of _windowing_ crates, executing your code written with the core and backend crate.
//! - A special crate, [luminance-agnostic], a special _backend_ crate that allows to combine
//!   several “official” crates to provide a cross-platform experience without having to pick
//!   several backend crates — the crate does it for you. This crate is mainly designed for end-user
//!   crates.
//!
//! ## The core crate
//!
//! The luminance crate gathers all the logic and rendering abstractions necessary to write code
//! over various graphics technologies. It contains parametric types and functions that depend on
//! the actual _implementation type_ — as a convention, the type variable `B` (for backend) is
//! used. For instance, the type `Buffer<B, u8>` is an 8-bit unsigned integer buffer for which the
//! implementation is provided via the `B` type.
//!
//! Backend types — i.e. `B` — are not provided by [luminance] directly. They are typically
//! provided by crates containing the name of the technology as suffix, such as luminance-gl,
//! luminance-webgl, luminance-vk, etc. The interface between those backend crates and
//! luminance is specified in [luminance::backend].
//!
//! On a general note, `Buffer<ConcreteType, u8>` is a monomorphic type that will be usable
//! **only** with code working over the `ConcreteType` backend. If you want to write a function
//! that accepts an 8-bit integer buffer without specifying a concrete type, you will have to
//! write something along the lines of:
//!
//! ```
//! use luminance::backend::buffer::Buffer as BufferBackend;
//! use luminance::buffer::Buffer;
//!
//! fn work<B>(b: &Buffer<B, u8>) where B: BufferBackend<u8> {
//!   todo!();
//! }
//! ```
//!
//! This kind of code is intented for people writing libraries with luminance. For the special case
//! of using the [luminance-agnostic] crate, you will end up writing something like:
//!
//! ```ignore
//! use luminance_agnostic::buffer::Buffer;
//!
//! fn work(b: &Buffer<u8>) {
//!   todo()!;
//! }
//! ```
//!
//! > In [luminance-agnostic], the backend type is selected at compile and link time. This is often
//! > what people want, but keep in mind that [luminance-agnostic] doesn’t allow to have several
//! > backend types at the same time, which might be something you would like to use, too.
//!
//! ## Backend implementations
//!
//! Backends implement the [luminance::backend] traits and provide, mostly, a single type for each
//! implementation. It’s important to understand that a backend crate can provide several backends
//! (for instance, [luminance-gl] can provide one backend — so one type — for each supported OpenGL
//! version). That backend type will be used throughout the rest of the ecosystem to deduce subsequent
//! implementors and associated types.
//!
//! If you want to implement a backend, you don’t have to push any code to any `luminance` crate.
//! `luminance-*` crates are _official_ ones, but you can write your own backend as well. The
//! interface is highly `unsafe`, though, and based mostly on `unsafe impl` on `unsafe trait`. For
//! more information, feel free to read the documentation of the [luminance::backend] module.
//!
//! ## Windowing
//!
//! luminance doesn’t know anything about the context it executes in. That means that it doesn’t
//! know whether it’s used within SDL, GLFW, glutin, Qt, a web canvas or an embedded specific hardware such as
//! the Nintendo Switch. That is actually powerful, because it allows luminance to be
//! completely agnostic of the execution platform it’s running on: one problem less. However, there
//! is an important point to take into account: a single backend type can be used with several windowing
//! crates / implementations. That allows to re-use a backend with several windowing
//! implementations. The backend will typically explain what are the conditions to create it (like,
//! in OpenGL, the windowing crate must set some specific flags when creating the OpenGL context).
//!
//! luminance does not provide a way to create windows because it’s important that it not depend
//! on windowing libraries – so that end-users can use whatever they like. Furthermore, such
//! libraries typically implement windowing and events features, which have nothing to do with our
//! initial purpose.
//!
//! A windowing crate supporting luminance will typically provide native types by re-exporting
//! symbols (types, functions, etc.) from a windowing crate and the necessary code to make it
//! compatible with luminance. That means providing a way to access a backend type, which
//! implements the [luminance::backend] interface.
//!
//! [luminance]: https://crates.io/crates/luminance
//! [luminance-gl]: https://crates.io/crates/luminance-gl
//! [luminance-agnostic]: https://crates.io/crates/luminance-agnostic
//! [luminance::backend]: crate::backend

//#![deny(missing_docs)]

pub mod backend;
pub mod blending;
pub mod buffer;
pub mod context;
pub mod depth_test;
pub mod face_culling;
pub mod framebuffer;
pub mod pipeline;
pub mod pixel;
pub mod render_gate;
pub mod render_state;
pub mod shader;
pub mod shading_gate;
pub mod tess;
pub mod tess_gate;
pub mod texture;
pub mod vertex;
