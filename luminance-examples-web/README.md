# luminance-examples, the Web edition

This project gathers examples from [luminance-examples] but re-written to work within one’s
browser.

<!-- vim-markdown-toc GFM -->

* [How to configure and run](#how-to-configure-and-run)
* [Entry point](#entry-point)

<!-- vim-markdown-toc -->

## How to configure and run

The way you run and test them is not the same as with [luminance-examples] and will most
of the time requires several tools installed:

- `npm`: required to download all the required package and bootstrap the rest of the installation.
- `yarn`: required to ease the whole building system. If you don’t have it installed, simply
  install it locally (local development install path) by running the `npm i yarn` command.

You will then have to install all the development tools to be able to transpile / build web apps.
All of this is handled by [webpack] in this current project but others can be used too. Quickie:

```
yarn install
```

Done. This will install all the required wasm tools and required dependencies.

```
yarn build
```

Will build the web app by first compiling the Rust project to the wasm target and will bundle
all the files together.

You can then run the examples with:

```
yarn serve
```

Which will open a local server that you can access at http://localhost:8080.

> Note: you don’t have to run `yarn build` if you plan to serve the files right after: `yarn serve`
> will effectively recompile your wasm project if a change has occurred. That also works with
> hot-reloading.

## Entry point

The entry point for the Web is somewhat confusing with a Rust project, because depending on what
you want to focus, it might be different. The idea is the following:

- The core logic is in the `lib.rs` file. It contains everything that relates to graphics code.
- The event, DOM and system logics are located in the `index.js` file. It is easier to hack in JS
  from within this file and call Rust functions to perform the various actions (key presses for
  instance, etc.)

[luminance-examples]: ../luminance-examples
[webpack]: https://webpack.js.org
