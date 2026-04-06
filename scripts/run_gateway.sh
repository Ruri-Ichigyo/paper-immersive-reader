#!/usr/bin/env bash

set -euo pipefail

cargo run -p paper-immersive-reader-gateway --bin paper-immersive-reader-gateway -- "$@"
