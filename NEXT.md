# Next changes to be available

> This document lists all the last changes that occurred lately and that are about to be publicly available soon. These
> are items that must be formatted accordingly and ready to be moved to the [CHANGELOG](./CHANGELOG.md).

# `luminance`

- Remove `Buffer` from the public API. Buffers are not really used by people, besides for _uniform buffers_, which are
  known to be problematic regarding memory alignment / safety. A complete redesign of this feature is planned and should
  land soon.
- Annotate `Tess::vertices`, `Tess::vertices_mut`, `Tess::indices`, `Tess::indices_mut`, `Tess::instances`,
  `Tess::instances_mut` and all associated types with lifetimes to prevent dropping the `Tess` while memory is sliced.
  This change shouldn’t create any issue if your code is sound but if you were doing something like dropping a `Tess`
  while still maintaining a slice, or randomly dropping `Tess` and slices, it is likely that you will have `rustc` ask
  you to fix your code now.

# `luminance-derive`

# `luminance-front`

- Update `Vertices`, `VerticesMut`, `Indices`, `IndicesMut`, `Instances` and `InstancesMut` to reflect the lifetime
  change that happened in `luminance`.

# `luminance-gl`

- Fix lifetime issue with slicing tessellation.

# `luminance-glfw`

# `luminance-glutin`

# `luminance-sdl2`

# `luminance-web-sys`

# `luminance-webgl`

- Fix buffer kind not correctly being used (i.e. mixing vertex and index buffers is not possible, for instance). This
  fix was the premise of the full fix, as a redesign of luminance’s buffer interface was needed to fully fix the problem.
- Fix lifetime issue with slicing tessellation.

# `luminance-windowing`
