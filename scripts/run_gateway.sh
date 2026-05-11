#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "==> Starting gateway server..."
cargo run -p paper-immersive-reader-gateway
