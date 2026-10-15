#!/usr/bin/env bash
# One-off: vendor the JS the wasm client needs into public/js so the Clojure
# server is self-contained (no CDN). Run it again only to upgrade these files.
set -euo pipefail
cd "$(dirname "$0")"
mkdir -p public/js

curl -fsSL https://not-fl3.github.io/miniquad-samples/mq_js_bundle.js -o public/js/mq_js_bundle.js

CARGO_SRC="${CARGO_HOME:-$HOME/.cargo}/registry/src"
cp "$CARGO_SRC"/*/sapp-jsutils-*/js/sapp_jsutils.js public/js/sapp_jsutils.js
cp "$CARGO_SRC"/*/quad-net-*/js/quad-net.js public/js/quad-net.js

# The page and the websocket live on the same server, so let the client say
# "ws://same-origin/ws" and resolve it from the page's own address.
sed -i 's|new WebSocket(consume_js_object(addr))|new WebSocket(midnight_ws_url(consume_js_object(addr)))|' public/js/quad-net.js
cat >> public/js/quad-net.js <<'EOF'

// Midnight Alley patch: "ws://same-origin/..." -> the address this page was served from.
function midnight_ws_url(url) {
    var scheme = location.protocol === "https:" ? "wss://" : "ws://";
    return url.replace(/^wss?:\/\/same-origin/, scheme + location.host);
}
EOF
grep -c midnight_ws_url public/js/quad-net.js
ls -la public/js
