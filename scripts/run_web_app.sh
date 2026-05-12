#!/usr/bin/env bash
set -euo pipefail

# Start the web app dev server for local development.
# Requires: trunk (cargo install trunk)
#           wasm32-unknown-unknown target (rustup target add wasm32-unknown-unknown)

cd "$(dirname "$0")/.."

echo "==> Starting web app dev server..."
trunk serve crates/web-app/index.html --open
