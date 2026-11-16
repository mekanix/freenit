#!/bin/sh
set -eu

cd "$(dirname "$0")/.."
cargo build --manifest-path ../dziber/Cargo.toml \
  --no-default-features --features web \
  --target wasm32-unknown-unknown --release
wasm-bindgen --target web --no-typescript \
  --out-dir assets --out-name dziber \
  ../dziber/target/wasm32-unknown-unknown/release/dziber.wasm
