# Next changes to be available

> This document lists all the last changes that occurred lately and that are about to be publicly available soon. These
> are items that must be formatted accordingly and ready to be moved to the [CHANGELOG](./CHANGELOG.md).

# `luminance`

- Remove `Buffer` from the public API. Buffers are not really used by people, besides for _uniform buffers_, which are
  known to be problematic regarding memory alignment / safety. A complete redesign of this feature is planned and should
  land soon.

# `luminance-derive`

# `luminance-front`

# `luminance-gl`

# `luminance-glfw`

# `luminance-glutin`

# `luminance-sdl2`

# `luminance-web-sys`

# `luminance-webgl`

- Fix buffer kind not correctly being used (i.e. mixing vertex and index buffers is not possible, for instance). This
  fix was the premise of the full fix, as a redesign of luminance’s buffer interface was needed to fully fix the problem.

# `luminance-windowing`
