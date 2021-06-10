const rust = import('./pkg');

// set to true when an example is ready to be rendered
let example_ready = false;
const such_a_shame = function(v) {
  if (v !== undefined) {
    example_ready = v;
  }

  return example_ready;
}

// things we can be waiting for in the user input arg field
const user_input_types = {
  TEXTURE: "texture",
};

// widgets
let widgets = document.createElement('div');
widgets.style.position = 'absolute';
widgets.style.top = 0;
widgets.style.left = 0;
document.body.appendChild(widgets);

// list of examples to run
let example_select = document.createElement('select');
example_select.add(document.createElement('option'));
widgets.appendChild(example_select);

// user input, if needed
let user_input_args = document.createElement('input');
user_input_args.hidden = true;
widgets.appendChild(user_input_args);

let canvas = document.createElement('canvas');
canvas.tabIndex = 0;
canvas.width = window.innerWidth;
canvas.height = window.innerHeight;
canvas.id = 'luminance-canvas';
canvas.hidden = true;
document.body.appendChild(canvas);

// Reset the view to its default.
const reset_view = function() {
  canvas.hidden = true;
  user_input_args.hidden = true;
  example_ready = false;
}

rust
  .then(wasm => {
    // get the showcase
    const showcase = wasm.get_showcase('luminance-canvas');

    // FIXME
    let user_input_type = null;

    // build the <select> shit madness
    const example_names = wasm.examples_names();
    example_names.forEach(name => {
      let option = document.createElement('option');
      option.text = name;
      example_select.add(option);
    });

    // handle example change
    example_select.onchange = change => {
      reset_view();
      showcase.reset();

      const value = change.target.value;
      canvas.hidden = value === '';
      if (value !== '') {
        // check the features
        const features = showcase.get_features(change.target.value);
        const textures = features.textures();

        // if we require some textures
        if (textures.length > 0) {
          if (textures.length == 1) {
            user_input_args.placeholder = 'URL to texture';
            user_input_args.hidden = false;
            user_input_type = user_input_types.TEXTURE;
          } else {
            error('not implemented yet');
          }
        } else {
          such_a_shame(true);
        }
      }
    };

    // handle user input
    user_input_args.onchange = change => {
      const value = change.target.value;

      switch (user_input_type) {
        case user_input_types.TEXTURE:
          fetch(value)
            .then(res => res.blob())
            .then(res => res.arrayBuffer())
            .then(res => {
              showcase.add_texture(value, new Uint8Array(res));
              such_a_shame(true);
            });
          break;

        default: ;
      }
    };

    // transform events into input actions
    canvas.addEventListener('keyup', (event) => {
      switch (event.code) {
        case 'Space':
          if (event.shiftKey) {
            showcase.enqueue_auxiliary_toggle_action();
          } else {
            showcase.enqueue_main_toggle_action();
          }
          break;

        case 'Escape':
          showcase.enqueue_quit_action();
          break;

        case 'KeyA':
          showcase.enqueue_left_action();
          break;

        case 'KeyD':
          showcase.enqueue_right_action();
          break;

        case 'KeyW':
          showcase.enqueue_up_action();
          break;

        case 'KeyS':
          showcase.enqueue_down_action();
          break;

        default:
      }
    });

    window.onresize = () => {
      if (window.innerWidth !== undefined && window.innerHeight !== undefined && window.innerWidth > 0 && window.innerHeight > 0) {
        canvas.width = window.innerWidth;
        canvas.height = window.innerHeight;
        showcase.enqueue_resized_action(window.innerWidth, window.innerHeight);
      }
    };

    window.onmousemove = (value) => {
      showcase.enqueue_cursor_moved_action(value.x, value.y);
    };

    window.onmouseup = (value) => {
      if (value.button == 0) {
        showcase.enqueue_primary_released_action();
      }
    };

    window.onmousedown = (value) => {
      if (value.button == 0) {
        showcase.enqueue_primary_pressed_action();
      }
    };

    const renderFrame = (now) => {
      if (such_a_shame()) {
        const feedback = showcase.render_example(example_select.value, now * 1e-3);

        if (!feedback) {
          example_select.value = '';
          showcase.reset();
          canvas.hidden = true;
        }
      }

      window.requestAnimationFrame(renderFrame);
    };

    window.requestAnimationFrame(renderFrame);
  })
  .catch(console.error);
