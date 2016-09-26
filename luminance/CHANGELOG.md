### 0.11.0

- Added the first, constraint-only, `UniformBlock` trait. Used to restrict types that can be passed
  around in uniform buffers.
- Added *proxied* types – i.e. `TextureProxy` and `UniformBufferProxy`. Those are used to build sets
  that can passed to shaders.
- Uniform buffers can now be sent to shaders via the `buffer::Binding` type. Put your buffer in a
  uniform buffer set and there you go.
- Textures are not `Uniformable` anymore. You have to put the texture in a texture set and use a
  `texture::Unit`, which is `Uniformable`.
- Added `buffer::Binding`.
- Added `texture::Unit`.
- `map_uniform` now takes `&str` instead of `String`.
- Cleaned up documentation.
- Fixed internal bugs.

## 0.10.0

- Fixed the type of the uniform errors in the uniform interface builder function.

### 0.9.1

- Documentation updates.

## 0.9.0

- Several textures can now be passed as uniforms to shaders. The interface is pretty instable as it
  might change in the future, because for now, the user has to pass the texture units each textures
  should be bound to.

## 0.8.0

- Documentation is now available on docs.rs.
- Replaced references to `Vec` by slices.

## 0.7.0

- Uniforms have new semantics; mapping them cannot fail anymore but it can be emitted warnings.

### 0.6.4

- Backends now have more information to work with about uniforms. That enables using reification
  techniques to validate types, dimensions, sizes, etc…

### 0.6.3

- Added `get_raw_texels` to `Texture`.

### 0.6.2

- Added `upload_part_raw` and `upload_raw` to `Texture`, enabling to upload raw texels instead of
  texels directly.
- Added `RawEncoding` to `Pixel`.

### 0.6.1

- Added documentation field in Cargo.toml.

## 0.6.0

- Removed `Default` implementation for `Framebuffer` and added a new `default()` method, taking the
  size of the `Framebuffer`.
- Added `RenderablePixel` trait bound on `Slot` implementations for `ColorPixel`.
- Added `RenderablePixel`.
- Removed the need of the **core** crate.
- Removed `UniformName`.
- We can now have textures as uniforms.
- New uniform system to accept value depending on the backend.
- Using `AsRef` instead of ATexture in `update_textures`.
- Changed the meaning of mipmaps (now it’s the number of extra mipmaps).
- Using `usize` instead of `u32` for mipmaps.
- Added `Dimensionable` and `Layerable` in the interface.

### 0.5.3

- Added `update_textures` into `HasUniform`.
- Fixed signature of `UniformUpdate::update`.
- Fixed trait bound on `UniformUpdate::{contramap, update}`.

### 0.5.2

- Added `UniformUpdate`.
- Added `Uniformable` in the public interfarce shortcut.

### 0.5.1

- Removed `run_pipeline` and added `Pipeline::run`.

## 0.5.0

- Fixed uniform interfaces in `ShadingCommand` and `RenderCommand` with existential quantification.
- Renamed `FrameCommand` into `Pipeline`.
- Several patch fixes.
- Added travis CI support.
- Added documentation for `Program`.

## 0.4.0

- Changed the whole `Program` API to make it safer. Now, it closely looks like the Haskell version
  of `luminance`. The idea is that the user cannot have `Uniform`s around anymore as they’re held by
  `Program`s. Furthermore, the *uniform interface* is introduced. Because Rust doesn’t have a
  **“rank-2 types”** mechanism as Haskell, `ProgramProxy` is introduced to emulate such a behavior.
- Added a bit of documentation for shader programs.

### 0.3.1

- Removed `rw`.

## 0.3.0

- Removed A type parameter form `Buffer`. It was unnecessary safety that was never actually used.
- Added documentation around.

### 0.2.1

- Exposed `Vertex` directly in `luminance`.

## 0.2.0

- Changed `Negative*` blending factors to `*Complement`.

## 0.1.0

- Initial revision.
