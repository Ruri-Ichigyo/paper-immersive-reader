#!/usr/bin/env bash

set -euo pipefail

cargo run -p paper-immersive-reader-web-app --bin paper-immersive-reader-web-app -- "$@"
