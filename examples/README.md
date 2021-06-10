# Luminance examples

This directory gathers all the currently available official examples for [luminance]. It is designed to use the
_backend-agnostic_ feature of [luminance]: you write the code once, and executes it on several platform by providing the
platform code to run it. [luminance] code can be imagined as library code that requires an executor to be fully
runnable.

Currently, two executors exist:

- [desktop](./desktop): compile the examples as a binary usable from CLI. The examples are selected by passing arguments
  to the binary on the command line.
- [web](./web): compile the examples as a WASM module, loaded in a webpack environment, run on a local server. The
  examples are selected via the DOM in the browser of your choice.

Feel free to visit each executor and try them out!

The actual code for the examples is located in the [common](./common) directory.

[luminance]: https://crates.io/crates/luminance
