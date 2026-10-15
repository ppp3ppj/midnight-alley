# Midnight Alley - Client

A [macroquad](https://github.com/not-fl3/macroquad) game client, compiled to WebAssembly.

## Build and run

```sh
cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/client.wasm .
```

```sh
cargo install basic-http-server
basic-http-server .
```

Open `http://localhost:4000` and you should see orange cats wandering in the dark.
