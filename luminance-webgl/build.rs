use std::env;
use std::fs::File;
use std::path::*;
use webgl_generator::*;

fn main() {
  let dest = env::var("OUT_DIR").unwrap();
  let mut output = File::create(&Path::new(&dest).join("webgl_stdweb.rs")).unwrap();

  Registry::new(Api::WebGl2, Exts::ALL)
    .write_bindings(StdwebGenerator, &mut output)
    .unwrap();
}
