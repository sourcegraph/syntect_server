# Syntect Server

This is an HTTP server that exposes the Rust [Syntect](https://github.com/trishume/syntect) syntax highlighting library for use by other services. Send it some code, and it'll send you syntax-highlighted code in response.

## Development

1. [Install Rust](https://www.rust-lang.org/en-US/install.html)
2. `git clone` this repository anywhere on your filesystem.
3. Use `cargo run` to download dependencies + compile + run the server.

## Building

Invoke `cargo build --release` and an optimized binary will be built (e.g. to `./target/debug/syntect_server`).

## Code hygiene

- Use `cargo fmt` or an editor extension to format code.
