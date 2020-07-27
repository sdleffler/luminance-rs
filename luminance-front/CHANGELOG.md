# Changelog

This document is the changelog of [luminance-front](https://crates.io/crates/luminance-front).
You should consult it when upgrading to a new version, as it contains precious information on
breaking changes, minor additions and patch notes.

**If you’re experiencing weird type errors when upgrading to a new version**, it might be due to
how `cargo` resolve dependencies. `cargo update` is not enough, because all luminance crate use
[SemVer ranges](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html) to stay
compatible with as many crates as possible. In that case, you want `cargo update --aggressive`.

<!-- vim-markdown-toc GFM -->

* [0.2.1](#021)
* [0.2](#02)
* [0.1](#01)

<!-- vim-markdown-toc -->

# 0.2.1

> Jul 27th, 2020

- Fix the default types of `Tess` and `TessBuilder` according to `luminance-0.41`. Those were
  missing while they shouldn’t.

# 0.2

> Jul 24th, 2020

- Re-export missing symbols, such as `RenderGate`.
- Fix the `Tessgate` -> `TessGate` symbol.
- Support of `luminance-0.41`.

# 0.1

> Wed Jul, 15th 2020

- Initial revision.
