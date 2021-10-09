# Next changes to be available

> This document lists all the last changes that occurred lately and that are about to be publicly available soon. These
> are items that must be formatted accordingly and ready to be moved to the [CHANGELOG](./CHANGELOG.md).

# `luminance`

- Remove `Buffer` from the public API. Buffers are not really used by people, besides for _uniform buffers_, which are
  known to be problematic regarding memory alignment / safety. A complete redesign of this feature is planned and should
  land soon, called `ShaderData`.
- Annotate `Tess::vertices`, `Tess::vertices_mut`, `Tess::indices`, `Tess::indices_mut`, `Tess::instances`,
  `Tess::instances_mut` and all associated types with lifetimes to prevent dropping the `Tess` while memory is sliced.
  This change shouldn’t create any issue if your code is sound but if you were doing something like dropping a `Tess`
  while still maintaining a slice, or randomly dropping `Tess` and slices, it is likely that you will have `rustc` ask
  you to fix your code now.
- Fix `Tess::instances` and `Tess::instances_mut` returned slices, which were using the wrong type variables and made it
  impossible to even compile that code. Because that situation couldn’t compile, we release this as a patch bump.
- Introduce `ShaderData`, a new abstraction to allow to share data between shaders and pass large amount of properties to
  implement techniques such as geometry instancing.
- Add `shader::types::*`, thin type wrappers allowing to pass aligned data in a fast backend agnostic way. If you were
  using encodings such as `[f32; 2]` for the GLSL counterpart `vec2`, you should now use `Vec2<f32>`.
- Update the documentation of the `luminance::backend` module.

# `luminance-derive`

# `luminance-front`

- Update `Vertices`, `VerticesMut`, `Indices`, `IndicesMut`, `Instances` and `InstancesMut` to reflect the lifetime
  change that happened in `luminance`.
- Export the `tess::View` trait, which was missing from the public interface.
- Export the new `luminance::shader::types::*`.

# `luminance-gl`

- Fix lifetime issue with slicing tessellation.
- Add support for `ShaderData` via `Std140` (`luminance-std140`).

# `luminance-glfw`

- Add support for `glfw-0.42`.

# `luminance-glutin`

# `luminance-sdl2`

# `luminance-std140`

- New crate, `luminance-std140` provides the `Std140` trait, allowing implementations to easily provide types for OpenGL-based backends.

# `luminance-web-sys`

# `luminance-webgl`

- Fix buffer kind not correctly being used (i.e. mixing vertex and index buffers is not possible, for instance). This
  fix was the premise of the full fix, as a redesign of luminance’s buffer interface was needed to fully fix the problem.
- Fix lifetime issue with slicing tessellation.
- Add support for `ShaderData` via `Std140` (`luminance-std140`).

# `luminance-windowing`
