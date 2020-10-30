const rust = import('./pkg');

var canvas = null;

function createCanvas() {
  canvas = document.createElement('canvas');
  canvas.width = 800;
  canvas.height = 600;
  canvas.id = 'luminance-canvas';
  document.body.appendChild(canvas);
}

function runExample(name, wasm) {
  // remove the canvas if already there and recreate it
  if (document.body.contains(canvas)) {
    console.log("-> Resetting canvas…");
    document.body.removeChild(canvas);
  }

  // do not run anything if we selected nothing
  if (name === "") {
    console.log("-> No test.");
    return;
  }

  createCanvas();

  console.log("-> running " + name);

  wasm.run_test(canvas.id, name);
}

rust
  .then(wasm => {
    // create a simple input to select the test to run
    var select = document.createElement('select');
    select.append(document.createElement('option'));
    select.onchange = () => {
      // set the location search so that we remember that we want to run this test at next page refresh
      window.history.replaceState({}, "", "?test_name=" + select.value);
      runExample(select.value, wasm);
    };

    const testNames = wasm.test_names();
    for (name in testNames) {
      var option = document.createElement('option');
      option.setAttribute('value', testNames[name]);
      option.append(testNames[name]);
      select.append(option);
    }

    document.body.appendChild(select);

    const testParam = new URLSearchParams(window.location.search).get('test_name');
    console.log(testParam);
    if (testParam != null) {
      select.value = testParam;
      runExample(testParam, wasm);
    }
  }).catch(console.error);
