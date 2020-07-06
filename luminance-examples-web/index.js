const rust = import('./pkg');

var canvas = document.createElement('canvas');
canvas.id = 'luminance-canvas';

document.getElementsByTagName('body')[0].appendChild(canvas);

rust
  .then(m => {
    console.log(m);
  })
  .catch(console.error);
