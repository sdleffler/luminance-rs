use colored::Colorize as _;
use wasm_bindgen::prelude::*;

macro_rules! tests {
  ($($name:expr, $module:ident),*) => {
    // declare the modules for all tests
    $(
      mod $module;
     )*

      // list of all available integration tests
      const TEST_NAMES: &[&str] = &[$( $name ),*];

    #[wasm_bindgen]
    pub fn test_names() -> JsValue {
      JsValue::from_serde(TEST_NAMES).unwrap()
    }

    // run a given test
    #[wasm_bindgen]
    pub fn run_test(canvas_name: &str, name: &str) {
      $(
        if name == $name {
          $module::fixture(canvas_name);
          return;
        }
      )*

      println!("{} is not a valid test. Possible values", name.red());

      for test_name in TEST_NAMES {
        println!("  -> {}", test_name.blue());
      }
    }
  }
}

tests! {
  "flatten-slice", flatten_slice,
  "pixel-array-encoding", pixel_array_encoding
}
