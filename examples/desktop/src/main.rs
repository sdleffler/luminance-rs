mod platform;

use std::{env::args, time::Instant};

use glfw::{Action, Context as _, Key, Modifiers, WindowEvent};
use luminance_examples::{Example, InputAction, LoopFeedback};
use luminance_glfw::GlfwSurface;
use luminance_windowing::{WindowDim, WindowOpt};
use platform::DesktopPlatformServices;

/// Macro to declaratively add examples.
macro_rules! examples {
  ($($name:literal, $test_ident:ident),*) => {
    fn show_available_examples() {
      log::error!("available examples:");
      $( log::error!("  - {}", $name); )*
    }

    // create a function that will run an example based on its name
    fn pick_and_run_example(example_name: &str) {
      match example_name {
        $(
          $name => {
            run_example::<luminance_examples::$test_ident::LocalExample>($name)
          }
        ),*

        _ => {
          log::error!("no example '{}' found", example_name);
          show_available_examples();
        }
      }
    }
  }
}

// Run an example.
fn run_example<E>(name: &str)
where
  E: Example,
{
  // First thing first: we create a new surface to render to and get events from.
  let dim = WindowDim::Windowed {
    width: 960,
    height: 540,
  };
  let surface =
    GlfwSurface::new_gl33(name, WindowOpt::default().set_dim(dim)).expect("GLFW surface creation");
  let mut context = surface.context;
  let events = surface.events_rx;

  let mut services = DesktopPlatformServices::new();

  let mut example = E::bootstrap(&mut services, &mut context);
  let start_t = Instant::now();

  'app: loop {
    // handle events
    context.window.glfw.poll_events();
    let actions = glfw::flush_messages(&events).flat_map(|(_, event)| adapt_events(event));

    let elapsed = start_t.elapsed();
    let t = elapsed.as_secs() as f64 + (elapsed.subsec_millis() as f64 * 1e-3);
    let feedback = example.render_frame(
      t as _,
      context.back_buffer().unwrap(),
      actions,
      &mut context,
    );

    if feedback == LoopFeedback::Continue {
      context.window.swap_buffers();
    } else {
      break 'app;
    }
  }
}

fn adapt_events(event: WindowEvent) -> Option<InputAction> {
  match event {
    WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => {
      Some(InputAction::Quit)
    }

    WindowEvent::Key(Key::Space, _, Action::Release, mods) => {
      if mods.is_empty() {
        Some(InputAction::MainToggle)
      } else if mods == Modifiers::Shift {
        Some(InputAction::AuxiliaryToggle)
      } else {
        None
      }
    }

    WindowEvent::Key(key, _, Action::Press, _) | WindowEvent::Key(key, _, Action::Repeat, _) => {
      match key {
        Key::A => Some(InputAction::Left),
        Key::D => Some(InputAction::Right),
        Key::W => Some(InputAction::Up),
        Key::S => Some(InputAction::Down),
        _ => None,
      }
    }

    WindowEvent::FramebufferSize(width, height) => Some(InputAction::Resized {
      width: width as _,
      height: height as _,
    }),
    _ => None,
  }
}

examples! {
  "hello-world", hello_world,
  "render-state", render_state,
  "sliced-tess", sliced_tess,
  "shader-uniforms", shader_uniforms,
  "attributeless", attributeless,
  "texture", texture
}

fn main() {
  env_logger::init();
  let arg = args().skip(1).next();

  if let Some(example_name) = arg {
    pick_and_run_example(&example_name);
  } else {
    log::error!("no example name provided");
    show_available_examples();
  }
}
