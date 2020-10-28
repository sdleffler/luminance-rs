use colored::Colorize as _;

macro_rules! tests {
  ($($name:expr, $module:ident),*) => {
    // declare the modules for all tests
    $(
      mod $module;
    )*

    // list of all available integration tests
    const TEST_NAMES: &[&str] = &[$( $name ),*];

    // run a given test
    fn run_test(name: &str) {
      $(
        if name == $name {
          $module::fixture();
          return;
        }
      )*

      else {
        println!("{} is not a valid test. Possible values", name.red());

        for test_name in TEST_NAMES {
          println!("  -> {}", test_name.blue());
        }
      }
    }
  }
}

tests! {
  "gl33-f64-uniform", gl33_f64_uniform,
  "tess-no-data", tess_no_data,
  "scissor-test", scissor,
  "360-manually-drop-framebuffer", manually_drop_framebuffer
}

fn main() {
  let test_name = std::env::args().skip(1).next();

  if let Some(test_name) = test_name {
    println!("test name: {}", test_name.green());

    run_test(&test_name);
  } else {
    println!("Please provide a test name. Possible values");

    for test_name in TEST_NAMES {
      println!("  -> {}", test_name.blue());
    }
  }
}
