# Midnight Alley - Client

A [macroquad](https://github.com/not-fl3/macroquad) game client, compiled to WebAssembly.

## Assets

```
assets/
  tiles/  farm_tilemap.png      ground, hay-bale roofs, greenhouse, pumpkins, sunflowers (18px tiles, 16 columns)
  rat/    rat_sheet.png         3 frames of 64x64: walk A, walk B, rest (art faces left)
  food/   snail_shell.png       the food the player drops
  _source/                      original SVGs the PNGs were rendered from
```

## Credits

- **Tiles:** [Pixel Platformer: Farm Expansion](https://kenney.nl/assets/pixel-platformer-farm-expansion) by [Kenney](https://kenney.nl), licensed CC0 (see `assets/tiles/farm_tilemap_LICENSE.txt`).
- **Rat and snail-shell art:** [New Platformer Pack](https://kenney.nl/assets/new-platformer-pack) (1.1) by [Kenney](https://kenney.nl), licensed CC0. The mouse and snail-shell SVGs are in `assets/_source/`; the PNGs are rendered from them.
- Built with [macroquad](https://github.com/not-fl3/macroquad).

## Build and run

```sh
cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/client.wasm .
```

```sh
cargo install basic-http-server
basic-http-server -a 0.0.0.0:4000 .
```

Open `http://localhost:4000` and you should see rats wandering through the alley at night. Click to drop food.

### Running under WSL

If `localhost:4000` doesn't load from your Windows browser:

1. Make sure the server binds to all interfaces (`-a 0.0.0.0:4000` as above, not just `127.0.0.1`).
2. Check `C:\Users\<you>\.wslconfig` for `localhostForwarding=false` under `[wsl2]`. Remove it or set it to `true`, then run `wsl --shutdown` and restart WSL.
3. As a fallback, get the WSL VM's IP with `hostname -I` and open `http://<that-ip>:4000` instead of `localhost`.
4. Verify the server is reachable from inside WSL first: `curl http://localhost:4000`. If that fails, the server itself isn't running/bound correctly; if it succeeds but the Windows browser still can't connect, check Windows Firewall rules for the `vEthernet (WSL)` adapter.
