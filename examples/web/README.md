# Luminance examples Web implementation

This directory holds the Web implementation of the examples.

## Quick start

You must install all the `npm` dependencies before trying to run the examples. Run this in the `examples/web` directory
(same as this README):

```sh
yarn install
```

This will install all the dependencies needed to bundle a Web app gathering the example. You can then start a
development server:

```sh
yarn serve
```

If you want to give a try to examples that require fetching and loading textures, it is highly recommended to run in
production mode. The reason for that is that `yarn serve` (or `yarn build`) will automatically ask
`wasm-pack-plugin` to (re)compile the Rust code using release targets, greatly speeding up loading times (this is mostly
due to the [image](https://crates.io/crates/image) crate):

```sh
yarn serve --mode production
```

## Run an example

Once you have started the local server, head over to http://localhost:8080. You should be facing a white page with a
drop-down list at the top-left of the page. That list contains all the available example. Selecting one will
automatically bootstrap the example and run it.

### Featured examples: textures

Some examples have special features, like being able to take _textures_ as input. The Web implementation uses the
[Fetch API](https://developer.mozilla.org/fr/docs/Web/API/Fetch_API) to get them. When such an example requires you to
pass a texture, you can currently pass the name of a texture that must live in the `examples/web/static` directory. So
far, this is pretty limited and you will most of the time be required to give a static name to the texture (like
`source.jpg`). This is subject to change later.

> There is currently no way to fetch textures from the Internet because of CORS and because of _j’ai la flemme_.

Once you have selected and submitted the name of the texture to use, the example should bootstrap, load the texture and
run the actual code.
