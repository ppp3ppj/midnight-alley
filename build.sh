#!/usr/bin/env bash
# Build the Rust client to WebAssembly and put it where the Clojure server serves it.
#   ./build.sh && cd server && lein run     ->  http://localhost:8080
set -euo pipefail
cd "$(dirname "$0")"

( cd client && cargo build --release --target wasm32-unknown-unknown )

PUB=server/public
mkdir -p "$PUB"
cp client/target/wasm32-unknown-unknown/release/client.wasm "$PUB/client.wasm"

# textures the wasm client loads at runtime (the SVG sources stay out)
rm -rf "$PUB/assets"
cp -r client/assets "$PUB/assets"
rm -rf "$PUB/assets/_source"

echo "built: $(ls -la "$PUB/client.wasm" | awk '{print $5}') bytes -> $PUB/"
