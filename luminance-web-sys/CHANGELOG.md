# Changelog

This document is the changelog of [luminance-web-sys](https://crates.io/crates/luminance-web-sys).
You should consult it when upgrading to a new version, as it contains precious information on
breaking changes, minor additions and patch notes.

**If you’re experiencing weird type errors when upgrading to a new version**, it might be due to
how `cargo` resolve dependencies. `cargo update` is not enough, because all luminance crate use
[SemVer ranges](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html) to stay
compatible with as many crates as possible. In that case, you want `cargo update --aggressive`.

<!-- vim-markdown-toc GFM -->

* [0.2.1](#021)
* [0.2](#02)
* [0.1.1](#011)
* [0.1](#01)

<!-- vim-markdown-toc -->

# 0.2.1

> Oct 26th, 2020

- Support of `luminance-0.43`.
- Support of `luminance-webgl-0.3`.

# 0.2

> Aug 30th, 2020

- Support of `luminance-0.42`.

# 0.1.1

> Jul 24th, 2020

- Support of `luminance-0.41`.

# 0.1

> Wed Jul, 15th 2020

- Initial revision.
