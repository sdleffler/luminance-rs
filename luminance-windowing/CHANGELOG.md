# Changelog

This document is the changelog of [luminance-windowing](https://crates.io/crates/luminance-windowing).
You should consult it when upgrading to a new version, as it contains precious information on
breaking changes, minor additions and patch notes.

**If you’re experiencing weird type errors when upgrading to a new version**, it might be due to
how `cargo` resolves dependencies. `cargo update` is not enough, because all luminance crate use
[SemVer ranges](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html) to stay
compatible with as many crates as possible. In that case, you want `cargo update --aggressive`.

<!-- vim-markdown-toc GFM -->

* [0.9.3](#093)
* [0.9.2](#092)
* [0.9.1](#091)
* [0.9](#09)
  * [Migration guide](#migration-guide)
* [0.8.1](#081)
* [0.8](#08)
* [0.7](#07)
* [0.6](#06)
* [0.5.1](#051)
* [0.5](#05)
  * [Major changes](#major-changes)
* [0.4](#04)
* [0.3.1](#031)
* [0.3](#03)
  * [Major changes](#major-changes-1)
  * [Minor changes](#minor-changes)
  * [Patch & misc changes](#patch--misc-changes)
* [0.2.4](#024)
* [0.2.3](#023)
* [0.2.2](#022)
* [0.2.1](#021)
* [0.2](#02)
* [0.1.1](#011)
* [0.1.0](#010)

<!-- vim-markdown-toc -->

# 0.9.3

> Oct 28, 2020

- Support of `luminance-0.43`.

# 0.9.2

> Aug 30th, 2020

- Support of `luminance-0.42`.

# 0.9.1

> Jul 24th, 2020

- Support of `luminance-0.41`.

# 0.9

> Wed Jul, 15th 2020

- Most features got removed and the crate is now a simple convience crate to provide some very
  common / naive types, such as `WindowDim` and `WindowOpt`. It is important that people realize
  that windowing (a.k.a. platform) code is specific to each crates and that there is no easy way
  (and it’s not even interesting) to abstract over them. The `Surface` trait for instance, is
  too much opinionated for the extern boundary of luminance.
- Support for `glfw-0.39`.
- Implement `std::error::Error` for various types of the crate.

## Migration guide

- If you were using a type that was removed, odds are that you will find the same feature in the
  `luminance-<platform-name>` crate. For instance, the `back_buffer` method is still available in
  [luminance-glfw] for instance.

# 0.8.1

> Sat Jan, 4th 2020

- Support of `luminance-0.38`.

# 0.8

> Sun Sep, 29th 2019

- Support of `luminance-0.37`.

# 0.7

> Fri Sep, 20th 2019

- `luminance-0.36` support.

# 0.6

> Thur Sep, 12th 2019

- Fix SemVer issues with ranges and duplicated dependencies.

# 0.5.1

> Thur Sep, 12th 2019

- Support of `luminance-0.35`.

# 0.5

> Wed Sep, 11th 2019

## Major changes

- The `Surface` trait has a new method to implement: `Surface::back_buffer`. That method provides
  the `Framebuffer::back_buffer` in a much more convenient way and is automatically implemented
  by default.

# 0.4

> Fri Sep, 6th 2019

- Support of `luminance-0.33`.

# 0.3.1

> Tue Sep, 3rd 2019

- Support of `luminance-0.32`.

# 0.3

> Fri Aug, 23th 2019

## Major changes

- Move `swap_buffers` from `GraphicsContext` to `Surface` in [luminance-windowing].

## Minor changes

- The `WindowOpt` now has support for multisampling. See the `WindowOpt::set_num_samples` for
  further details.
- Migrate to Rust Edition 2018.
- Implement dynamic edition of windowing types properties. That allows to change data on-the-fly,
  such as the cursor mode.

## Patch & misc changes

- Add more CI testing.
- Massive documentation rewrite (among the use of `#![deny(missing_docs)]`. The situation is still
  not perfect and patch versions will be released to fix and update the documentation. Step by
  step.
- Massive dependencies update. Special thanks to @eijebong for his help!

# 0.2.4

> Thursday, 20th of July, 2018

- Add support for `luminance-0.30`.

# 0.2.3

> Tuesday, 13th of July, 2018

- Add support for `luminance-0.29`.

# 0.2.2

> Tuesday, 3rd of July, 2018

- Add support for `luminance-0.28`.

# 0.2.1

> Friday, 29th of June, 2018

- Add support for `luminance-0.27`.

# 0.2

> Sunday, 17th of June, 2018

- Re-export the new `luminance` backend interface.
- Remove the concept of `Device` and the concept of `Surface`.

# 0.1.1

> Sunday, 1st of October, 2017

- Fix a small nit in the documentation.

# 0.1.0

> Saturday, 30th of September, 2017

- Initial revision.

[luminance-glfw]: https://crates.io/crates/luminance-glfw
