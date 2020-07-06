const rust = import('./pkg');

var canvas = document.createElement('canvas');
canvas.id = 'luminance-canvas';

document.getElementsByTagName('body')[0].appendChild(canvas);

rust
  .then(wasm => {
    // get the scene and set it up (events)
    const scene = wasm.get_scene('luminance-canvas');

    window.onkeypress = (event) => {
      switch (event.key) {
        case ' ':
          wasm.toggle_tess_method(scene);
          break;

        default:
      }
    };

    setInterval(() => wasm.render_scene(scene), 100);
  })
  .catch(console.error);
