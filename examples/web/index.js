const rust = import('./pkg');

// set to true when an example is ready to be rendered
let example_ready = null;
const set_ready = function(v) {
  example_ready = v;
}

const is_ready = function() {
  return example_ready !== null;
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

// textures (leave empty for no texture)
let texture_input = document.createElement('input');
widgets.appendChild(texture_input);

// run button
let run_button = document.createElement('button')
run_button.innerText = 'Run';
widgets.appendChild(run_button);

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
  texture_input.value = '';
  example_select.value = '';
  set_ready(null);
}

rust
  .then(wasm => {
    // get the showcase
    const showcase = wasm.get_showcase('luminance-canvas');

    // build the <select> shit madness
    const example_names = wasm.examples_names();
    example_names.forEach(name => {
      let option = document.createElement('option');
      option.text = name;
      example_select.add(option);
    });

    // handle run
    run_button.onclick = () => {
      // check that something is selected
      const example_name = example_select.value;

      if (example_name === '') {
        return;
      }

      // if the user has typed any texture path, load it and make it available to the example
      const texture_name = texture_input.value;
      if (texture_name !== '') {
        console.log('there’s a texture OMG, and it’s ' + texture_name);
        fetch(texture_name)
          .then(res => res.blob())
          .then(res => res.arrayBuffer())
          .then(res => {
            console.log('adding the texture');
            showcase.add_texture(new Uint8Array(res));

            set_ready(example_name);
            canvas.hidden = false;
          }).catch(error => {
            console.error(error);
          });
      } else {
        // set it ready right away
        set_ready(example_name);
        canvas.hidden = false;
      }

    }

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
          showcase.enqueue_forward_action();
          break;

        case 'KeyS':
          showcase.enqueue_backward_action();
          break;

        case 'KeyF':
          showcase.enqueue_up_action();
          break;

        case 'KeyR':
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
      if (is_ready()) {
        const feedback = showcase.render_example(example_ready, now * 1e-3);

        if (!feedback) {
          reset_view();
        }
      }

      window.requestAnimationFrame(renderFrame);
    };

    window.requestAnimationFrame(renderFrame);
  })
  .catch(console.error);
