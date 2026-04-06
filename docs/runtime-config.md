# Runtime Configuration

The gateway and web MVP startup scripts remain Cargo-native. No second package manager is required.

## Gateway

The gateway reads these environment variables:

- `PAPER_IR_GATEWAY_BIND_ADDR`: socket address for the Axum server. Default: `127.0.0.1:4000`
- `PAPER_IR_GATEWAY_TOKEN`: optional gateway token reserved for later authenticated web-client calls
- `PAPER_IR_UPSTREAM_AI_BASE_URL`: upstream AI endpoint base URL
- `PAPER_IR_UPSTREAM_AI_API_KEY`: upstream AI API key

The gateway writes logs to `logs/YYYY-MM-DD_HHMMSS.log`.

## Web App

The current web app startup helper runs the Rust workspace binary shell:

- `scripts/run_web_app.sh`

Future browser bundling variables should be documented here when the WASM/PWA runtime is introduced.
