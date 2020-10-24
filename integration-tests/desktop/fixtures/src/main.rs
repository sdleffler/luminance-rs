mod gl33_f64_uniform;
mod tess_no_data;

use colored::Colorize as _;

macro_rules! fixture {
  ($name:expr, $module:ident) => {
    println!("Running: {}", $name.blue().italic());
    let r = $module::fixture();
    println!(
      "  -> {}",
      if r {
        "Success!".green()
      } else {
        "Nope :(".red()
      }
    );
  };
}
fn main() {
  fixture!("Tess with no data should generate an error", tess_no_data);
  fixture!(
    "Double-precision floating point uniforms support",
    gl33_f64_uniform
  );
}
