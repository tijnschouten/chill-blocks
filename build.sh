#!/usr/bin/env bash
# Builds the wasm module and generates the browser bindings.
set -euo pipefail

cd "$(dirname "$0")"

WASM_BINDGEN="tools/wasm-bindgen-0.2.129-aarch64-apple-darwin/wasm-bindgen"

cargo build --target wasm32-unknown-unknown -p chillblocks-app --release

if [[ ! -x "$WASM_BINDGEN" ]]; then
  echo "wasm-bindgen CLI not found at $WASM_BINDGEN" >&2
  echo "Install it, or run: cargo install wasm-bindgen-cli" >&2
  exit 1
fi

"$WASM_BINDGEN" --target web --out-dir web/pkg \
  target/wasm32-unknown-unknown/release/chillblocks_app.wasm

echo "Built web/pkg. Serve the web/ directory with any static server, e.g.:"
echo "  uv run --no-project python -m http.server -d web 8080"
