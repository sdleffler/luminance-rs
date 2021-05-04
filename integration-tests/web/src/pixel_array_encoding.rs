use luminance_front::pixel::RGB8UI;
use luminance_front::texture::{Dim2, Sampler};
use luminance_front::{context::GraphicsContext as _, texture::Texture};
use luminance_web_sys::WebSysWebGL2Surface;

pub fn fixture(canvas_name: &str) {
  let mut surface = WebSysWebGL2Surface::new(canvas_name).expect("web-sys surface");
  web_sys::console::log_1(&"got surface".into());

  let _texture: Texture<Dim2, RGB8UI> = surface
    .new_texture_no_texels([100, 100], 0, Sampler::default())
    .unwrap();
}
