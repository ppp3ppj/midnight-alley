# Midnight Alley - Client

A [macroquad](https://github.com/not-fl3/macroquad) game client, compiled to WebAssembly.

## The story

When the sun goes down in an old city, the stray rats come out. The market has closed, but scraps of food are left on the ground. A few warm roofs still hold the heat of the day, the best places to sleep. Every night, dozens of rats compete for food and warm spots, pups are born, and the big street dog patrols the alley. And sometimes a kind old lady comes out with a bowl of food (that's the player, clicking to drop food).

The goal is to keep the rat colony alive through as many nights as possible.

### How the story maps to Clojure

- **Food scraps and food bowls are refs.** When two hungry rats jump on the same scrap at the same moment, a `dosync` transaction makes sure only one rat gets it. The other rat stays hungry and looks somewhere else.
- **Warm roofs are refs too.** Each roof fits only one sleeping rat. Claiming a spot is a transaction, so two rats can never sleep in the same place.
- **Each rat's mind is an agent.** Rats decide on their own in the background: look for food, nap, or run from the dog. Each rat has hunger, energy, and mood.
- **The colony diary is an atom.** It counts rats alive, pups born, food eaten, and which night it is.
- **The night cycle runs on core.async.** A loop ticks the world forward, changes night to morning (rats go hide and sleep), and sends snapshots to the browser.

### The "broken mode" demo

Turn off STM and use plain mutable arrays. Now you get ghost food: two rats eat the same scrap, both get full, and the food count goes negative. Two rats sleep on one roof inside each other. It's funny to watch, and it shows the problem clearly: without safe shared state, the world stops making sense. Turn STM back on and everything is correct again.

### Rendering in macroquad

The client draws a side-view alley from Kenney tiles: grass and dirt ground, hay-bale roofs, a greenhouse and pumpkins. Rats are pixel-art sprites that walk, hop and curl up to rest, tinted red when very hungry. Food scraps are snail shells. The background is dark blue with the greenhouse windows lit, and it fades to light gray at morning.

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

## Run with the server (desktop)

The client only draws; the Clojure server in `../server` runs the world.

```sh
cd ../server && lein run     # terminal 1: ws://localhost:8080/ws
cd ../client && cargo run    # terminal 2
```

Click to drop food. Press **B** to switch the server between STM on and broken mode (ghost food, two rats on one roof).
Set `MIDNIGHT_SERVER=host:port` to use another server address. If the server isn't up yet the client waits and keeps retrying.

## Build and run (browser, served by the Clojure server)

```sh
./build.sh                   # from the repo root: builds client.wasm into server/public/
cd server && lein run        # then open http://localhost:8080
```

The server serves `index.html`, `client.wasm`, the JS plugins and `assets/` from `server/public/`,
and the page talks back to the same server over `/ws`. Re-run `./build.sh` after changing Rust code.
The old standalone instructions below build the wasm by hand and are only needed without the server.

## Build by hand (WebAssembly)

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
