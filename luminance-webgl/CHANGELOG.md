# Changelog

This document is the changelog of [luminance-webgl]. You should consult it when upgrading to a new
version, as it contains precious information on breaking changes, minor additions and patch
notes.

**If you’re experiencing weird type errors when upgrading to a new version**, it might be due to
how `cargo` resolves dependencies. `cargo update` is not enough, because all luminance crates use
[SemVer ranges](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html) to stay
compatible with as many crates as possible. In that case, you want `cargo update --aggressive`.

<!-- vim-markdown-toc GFM -->

* [0.1](#01)

<!-- vim-markdown-toc -->

# 0.1

> ?

- Initial revision.

[luminance-webgl]: https://crates.io/crates/luminance-webgl
