# Midnight Alley - Server

The rat colony, simulated in Clojure.

| Part of the story | Clojure |
| --- | --- |
| Food scraps, warm roofs | `ref`s, claimed inside `dosync` (`world.clj`) |
| A rat's mind | an `agent` per rat (`rat.clj`) |
| The colony diary (rats alive, pups born, food eaten, night) | an `atom` (`world.clj`) |
| The night cycle | a `core.async` loop that ticks the world and publishes snapshots (`engine.clj`) |
| Browser connection | http-kit WebSocket, JSON snapshots (`server.clj`) |

**Broken mode** swaps the refs for plain atoms and a mutable array with an unprotected
check-then-act: ghost food (two rats eat one scrap, the count goes negative) and two rats on one roof.

## Run

Needs a JDK and [Leiningen](https://leiningen.org).

```sh
lein test    # includes the safe-vs-broken race demos
lein run     # http://localhost:8080  (set PORT to change, PUBLIC_DIR for the web folder)
```

The server is also the web server. `public/` holds the page and the vendored JS
(`vendor-js.sh` fetched it once); `../build.sh` compiles the Rust client to
`public/client.wasm` and copies the assets next to it. Open `http://localhost:8080`
and the wasm client connects back to `/ws` on the same address.

Quick check once it runs: `curl http://localhost:8080/snapshot`

## Protocol

Server to browser, every 100 ms: a JSON snapshot
`{safe?, night, morning?, daylight, food-count, food[], roofs[], rats[], diary, events[]}`.
World x positions go from 0 to 1000 (scale to the screen width). `events` holds
`{"type":"death","x":..,"roof":..}` entries so the client can play the boom.

Browser to server:

```json
{"type": "drop", "x": 500}
{"type": "mode", "safe": false}
{"type": "reset"}
```

`reset` starts over: night 1, empty food, free roofs and a fresh colony of 30 (the current mode is kept).
