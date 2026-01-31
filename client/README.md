# AlphaChess client (Rust / Yew)

Web client for AlphaChess, built with Yew and compiled to WASM. Uses the local engine crate for game state and [stockfish.js](https://github.com/lichess-org/stockfish.js) (Web Worker) for AI move generation—no server required.

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Trunk](https://trunkrs.dev/): `cargo install trunk`
- wasm32 target: `rustup target add wasm32-unknown-unknown`

## Build and run

```bash
# Development (with reload)
trunk serve

# Release build
trunk build --release
```

Output is in `dist/`.

## Stockfish (AI)

The AI uses [lichess-org/stockfish.js](https://github.com/lichess-org/stockfish.js). The bridge tries to load the worker from the release; if that fails (e.g. CORS), it falls back to `/stockfish.js` or `/stockfish.wasm.js` in your dist. To use the local fallback, download the matching file from [releases](https://github.com/lichess-org/stockfish.js/releases) (e.g. `ddugovic-250718`) and place it in `dist/`.

## Assets

SVGs and sounds are in `assets/` and are copied into `dist/` by Trunk.
