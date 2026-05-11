# Runtime Configuration

Environment variables used by the gateway and web app.

## Gateway

| Variable | Required | Default | Description |
|---|---|---|---|
| `GATEWAY_BIND_ADDRESS` | No | `127.0.0.1:3000` | Address the gateway HTTP server binds to |
| `AI_API_KEY` | Yes* | — | API key for the upstream AI service |
| `AI_BASE_URL` | No | — | Base URL for the AI API (provider-specific) |
| `GATEWAY_TOKEN` | No | — | Optional shared token the web client must send |

*Required when using a real AI client; not needed with the stub.

## Web App

| Variable | Required | Default | Description |
|---|---|---|---|
| `GATEWAY_URL` | No | `http://127.0.0.1:3000` | URL of the gateway for AI requests |
| `GATEWAY_TOKEN` | No | — | Token sent with each gateway request |

## Logging

The gateway writes structured JSON logs to `logs/YYYY-MM-DD_HHMMSS.log`.
Set `RUST_LOG=debug` for verbose output during development.
