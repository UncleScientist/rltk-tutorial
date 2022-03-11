# rltk-tutorial

My implementation of the [Rust Roguelike
Tutorial](https://bfnightly.bracketproductions.com/chapter_67.html)

## To build for the web:

$ `rustup target add wasm32-unknown-unknown`

$ `cargo install wasm-bindgen-cli`

$ `./build.sh`

Then start a webserver to serve the files in `$PROJECT_ROOT/wasm` and point your browser at it.

## To build natively:

$ `cargo run`
