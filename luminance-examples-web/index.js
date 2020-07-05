const rust = import('./pkg');

var canvas = document.createElement('canvas')
canvas.id = 'luminance-canvas'

document.getElementByTagName('body')[0].appendChild(canvas)

rust
  .then(m => m.hello_world('luminance-canvas'))
  .catch(console.error);
