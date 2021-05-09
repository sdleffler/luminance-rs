const rust = import('./pkg');

// TODO: replace with a <select>
let example_select = document.createElement('select');
example_select.add(document.createElement('option'));
console.log(example_select);

example_select.classList.add('example-selector');
example_select.style.position = 'absolute';
example_select.style.top = 0;
example_select.style.left = 0;
document.body.appendChild(example_select);

let canvas = document.createElement('canvas');
canvas.width = window.innerWidth;
canvas.height = window.innerHeight;
canvas.id = 'luminance-canvas';
canvas.hidden = true;
document.body.appendChild(canvas);

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

    example_select.onchange = change => {
      showcase.reset();
      canvas.hidden = change.target.value === '';
    };

    // transform events into input actions
    window.onkeyup = (event) => {
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

        default:
      }
    };

    window.onresize = () => {
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;
      showcase.enqueue_resized_action(window.width, window.height);
    };

    setInterval(() => {
      if (example_select.value !== '') {
        const feedback = showcase.render_example(example_select.value);

        if (!feedback) {
          example_select.value = '';
          showcase.reset();
          canvas.hidden = true;
        }
      }
    }, 10);
  })
  .catch(console.error);
