# paper-immersive-reader

A Cargo-native, web-first, local-first immersive paper reader.

## Architecture

The project uses a split architecture:

- **`crates/web-app`**: Rust/WASM browser client (Leptos) for library, import, reading, vocabulary, and settings.
- **`crates/gateway`**: Thin Rust HTTP gateway (Axum) for AI translation/explanation requests.
- **`crates/core`**: Shared domain types (Paper, Block, ReadingProgress, Annotation, VocabularyItem, OcrJob).
- **`crates/application`**: Shared orchestration and persistence contracts.
- **`crates/importing`**: Text normalization, segmentation, and PDF extraction helpers.
- **`crates/cli`**: CLI for OCR job execution and local management.

The browser is the source of truth for reading data. The gateway is optional for core reading and required only for AI-backed actions.

## Quick Start

### Prerequisites

- Rust toolchain (stable)
- `wasm32-unknown-unknown` target (`rustup target add wasm32-unknown-unknown`)
- `trunk` (`cargo install trunk`)

### Gateway (AI backend)

```bash
# Start the gateway server
./scripts/run_gateway.sh
```

See [`docs/runtime-config.md`](docs/runtime-config.md) for environment variable configuration.

### Web App (frontend)

```bash
# Start the web app dev server
./scripts/run_web_app.sh
```

## Verification

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```
