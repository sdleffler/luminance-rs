# What is this?

[![Build Status](https://travis-ci.org/phaazon/luminance-rs.svg?branch=master)](https://travis-ci.org/phaazon/luminance-rs)
[![crates.io](https://img.shields.io/crates/v/luminance.svg)](https://crates.io/crates/luminance)
[![docs.rs](https://docs.rs/luminance/badge.svg)](https://docs.rs/luminance/)
![License](https://img.shields.io/badge/license-BSD3-blue.svg?style=flat)

`luminance` is an effort to make graphics rendering simple and elegant. The aims of `luminance` are:

  - making **unsafe** and **stateful** *APIs* (e.g. **OpenGL**) **safe** and **stateless** ;
  - providing a simple API; that is, exposing core concepts without anything extra – just the bare
    stuff ;
  - abstract over the trending hardware interfaces (i.e. **OpenGL** up to now) and provide several
    backends to pick through different packages ;
  - easy to read with a good documentation and set of tutorials, so that new comers don’t have to
    learn a lot of new concepts to get their feet wet.

# What’s included?

`luminance` is a rendering framework, not a 3D engine. As so, it doesn’t include stuff like
lights, materials, asset management nor scene description. It only provides a rendering framework
you can plug in whatever libraries you want to.

## luminance ecosystem

Because I think it’s important to [KISS](https://en.wikipedia.org/wiki/KISS_principle), `luminance`
is split in very several, very simple packages. The idea is that the `luminance` package is the core
package of the library. It provides all the interface, common algorithms and the overall
architecture and how you should interact with a *GPU*. However, you need a *backend* to interpret
that code and make it run – one could even imagine a backend making it run on a CPU!

Feel free to search for `luminance-*` packages and pick the one you need ;).

## Features set

- **buffers**: **buffers** are way to communicate with the *GPU*; they represent regions of memory
  you can write to and read from. There’re several kinds of buffers you can create, among *vertex
  and index buffers*, *shader buffer*, *compute buffer*, and so on and so forth… ;
- **framebuffers**: **framebuffers** are used to hold *renders*. Each time you want to perform a
  render, you need to perform it into a framebuffer. Framebuffers can then be combined with each
  other to produce nice effects ;
- **shaders**: `luminance` support five kinds of shader stages:
  - tessellation control shaders ;
  - tessellation evaluation shaders ;
  - vertex shaders ;
  - geometry shaders ;
  - fragment shaders ;
- **vertices, indices, primitives and tessellations**: those are used to define a shape you can
  render into a framebuffer
- **textures**: **textures** represent information packed into arrays on the GPU, and can be used
  to customize a visual aspect or pass information around ;
- **blending**: **blending** is the process of taking two colors from two framebuffers and mix them
  between each other ;
- and a lot of other cool things like *GPU commands*.

# Current backends

Here’s a list of backends for `luminance`. If you’ve written one and like to make it appear in that
list, feel free to contact me on github or push a PR ;).

- `luminance-gl`: **OpenGL** backend

# Windowing

`luminance` does not provide point a way to create windows because it’s important that it not depend
on windowing libraries so that end-users can use whatever they like. Furthermore, such libraries
typically implement windowing and events features, which have nothing to do with our initial
purpose.

# How to dig in?

`luminance` is written to be fairly simple. [The documentation](https://docs.rs/luminance/) is very
transparent about what the library does and several articles will appear as the development goes on.
Keep tuned! Feel free to generate the documentation on local and browse it with the projects you’re
linking `luminance` against! (`cargo doc`).
