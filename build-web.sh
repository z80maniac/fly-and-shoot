#!/usr/bin/env bash
set -e
cd "$(dirname -- "${BASH_SOURCE[0]}")"

cargo build --release --target wasm32-unknown-unknown --features=x11
wasm-bindgen --out-name flyandshoot --out-dir web/wasm --target web target/wasm32-unknown-unknown/release/flyandshoot.wasm

rm web/wasm/*.ts
cp -a assets web/
cp CREDITS.TXT web/
