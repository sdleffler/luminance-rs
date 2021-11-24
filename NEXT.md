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
- Change the backend interface for `Uniformable`. Backends must now implement `Uniformable<T>`, instead of having to provide the
  implementor `T: Uniformable<Backend>`. This is a change allowing for better polymorphic code, where people can create
  « trait aliases » by simply adding `Uniformable<TypeHere>`.
- Enhance the documentation of various types.
- Support for uniform array and runtime-check them.
- Change the encoding of clear color. Now, `PipelineState` expects an `Option<[f32; 4]>`. If it’s `None`, then
  color clearing will be disabled. Otherwise, it will be enabled with the provided color.
- Add support for stencil buffers.
- Change the way texture uploads / texture creation works. Methods such as `GraphicsContext::new_texture_no_texels` are
  removed. Instead, all the texture API functions must go through the new `TexelUpload` type, which encodes various
  different situations:
  - Providing the base level, the number of mipmaps and let luminance generate the mipmaps automatically.
  - Providing the base level with no mipmaps.
  - Providing the base level as well as all the mipmap levels.

# `luminance-derive`

# `luminance-front`

- Update `Vertices`, `VerticesMut`, `Indices`, `IndicesMut`, `Instances` and `InstancesMut` to reflect the lifetime
  change that happened in `luminance`.
- Export the `tess::View` trait, which was missing from the public interface.
- Export the new `luminance::shader::types::*`.
- Fix architecture-based detection. The current process is that if the target architecture is not
  `wasm32-unknown-unknown`, we use `luminance-gl`. So we don’t depend on the CPU architecture anymore.

# `luminance-gl`

- Fix lifetime issue with slicing tessellation.
- Add support for `ShaderData` via `Std140` (`luminance-std140`).
- Implement the new `Uniformable` interface.
- Support for uniform array and runtime-check them.
- Document public symbols.
- Support the new color clearing.
- Implement the new `TexelUpload` interface.

# `luminance-glfw`

- Add support for `glfw-0.42`.
- Remove support for `luminance-windowing`. The interface now requires you to pass a function to build the `Window` and
  the `Receiver` for the events. This might seem like a regression but it actually allows for a more flexible way to use
  `luminance`. Instead of hiding the window construction to the user and trying to do things for them, `luminance-glfw`
  now just passes the strict minimum to the `Glfw` object (basically, the OpenGL context), and the user can create the
  window the way they want.

# `luminance-glutin`

# `luminance-sdl2`

- Bump support of `sdl2-0.35.1`.

# `luminance-std140`

- New crate, `luminance-std140` provides the `Std140` trait, allowing implementations to easily provide types for OpenGL-based backends.

# `luminance-web-sys`

- Remove useless dependency (`luminance-windowing`).

# `luminance-webgl`

- Fix buffer kind not correctly being used (i.e. mixing vertex and index buffers is not possible, for instance). This
  fix was the premise of the full fix, as a redesign of luminance’s buffer interface was needed to fully fix the problem.
- Fix lifetime issue with slicing tessellation.
- Add support for `ShaderData` via `Std140` (`luminance-std140`).
- Implement the new `Uniformable` interface.
- Support for uniform array and runtime-check them.
- Support the new color clearing.
- Implement the new `TexelUpload` interface.

# `luminance-windowing`

- Final version of the crate as it is now deprecated and will not be maintained.
