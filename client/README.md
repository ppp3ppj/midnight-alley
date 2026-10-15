# Midnight Alley - Client

A [macroquad](https://github.com/not-fl3/macroquad) game client, compiled to WebAssembly.

## Build and run

```sh
cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/client.wasm .
```

```sh
cargo install basic-http-server
basic-http-server -a 0.0.0.0:4000 .
```

Open `http://localhost:4000` and you should see orange cats wandering in the dark.

### Running under WSL

If `localhost:4000` doesn't load from your Windows browser:

1. Make sure the server binds to all interfaces (`-a 0.0.0.0:4000` as above, not just `127.0.0.1`).
2. Check `C:\Users\<you>\.wslconfig` for `localhostForwarding=false` under `[wsl2]`. Remove it or set it to `true`, then run `wsl --shutdown` and restart WSL.
3. As a fallback, get the WSL VM's IP with `hostname -I` and open `http://<that-ip>:4000` instead of `localhost`.
4. Verify the server is reachable from inside WSL first: `curl http://localhost:4000`. If that fails, the server itself isn't running/bound correctly; if it succeeds but the Windows browser still can't connect, check Windows Firewall rules for the `vEthernet (WSL)` adapter.
