#!/bin/bash
set -x
cargo build --bin demo --target wasm32-unknown-unknown --release
~/.cargo/bin/wasm-bindgen target/wasm32-unknown-unknown/release/demo.wasm --out-dir web --target web
python -m http.server -d web
