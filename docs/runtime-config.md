# Runtime Configuration

Environment variables used by the gateway and web app.

## Gateway

| Variable | Required | Default | Description |
|---|---|---|---|
| `GATEWAY_BIND_ADDRESS` | No | `127.0.0.1:3000` | Address the gateway HTTP server binds to |
| `GATEWAY_AI_API_KEY` | Yes* | — | API key for the upstream AI service (OpenAI-compatible) |
| `GATEWAY_AI_BASE_URL` | No | `https://api.openai.com/v1` | Base URL for the OpenAI-compatible chat completions endpoint |
| `GATEWAY_AI_MODEL` | No | `gpt-3.5-turbo` | Model name to pass in chat completion requests |
| `GATEWAY_AI_USE_STUB` | No | `false` | Set to `true` to use the stub AI client (no API key needed) |
| `GATEWAY_TOKEN` | No | — | Optional shared token the web client must send |
| `AI_API_KEY` | No | — | Deprecated alias for `GATEWAY_AI_API_KEY` |
| `AI_BASE_URL` | No | — | Deprecated alias for `GATEWAY_AI_BASE_URL` |

*Required when `GATEWAY_AI_USE_STUB` is not `true`.

## Web App

| Variable | Required | Default | Description |
|---|---|---|---|
| `GATEWAY_URL` | No | `http://127.0.0.1:3000` | URL of the gateway for AI requests |
| `GATEWAY_TOKEN` | No | — | Token sent with each gateway request |

## Logging

The gateway writes structured JSON logs to `logs/YYYY-MM-DD_HHMMSS.log`.
Set `RUST_LOG=debug` for verbose output during development.
