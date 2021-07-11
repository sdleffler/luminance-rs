# Luminance examples desktop implementation

This directory holds the desktop implementation of the examples.

## Quick start

Nothing specific to do, just run `cargo run -- -l` in `examples/desktop` to build the examples and list all the
available examples you can run. An example is run by giving its name to the built binary, like so:

```sh
cargo run hello-world
```

Some examples have _features_, such as loading textures. Those require a special argument to be passed when run: the
`-t` argument for textures, for instance:

```sh
cargo run -- -t /tmp/texture-test displacement-map
```

For examples using textures, it is highly recommended to compile in `--release` mode:

```sh
cargo run --release -- -t /tmp/texture-test displacement-map
```
