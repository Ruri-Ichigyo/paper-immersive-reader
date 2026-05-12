#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

# Load .env if present (for dev convenience)
if [ -f .env ]; then
  set -a
  source .env
  set +a
fi

echo "==> Starting gateway server..."
cargo run -p paper-immersive-reader-gateway
