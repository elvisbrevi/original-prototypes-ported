#!/bin/sh
# Compila la versión web en dist/web (lista para subir a itch.io con butler).
set -e
cd "$(dirname "$0")/.."
cargo build --release --target wasm32-unknown-unknown
rm -rf dist/web
mkdir -p dist/web
cp web/index.html web/mq_js_bundle.js dist/web/
cp target/wasm32-unknown-unknown/release/iman.wasm dist/web/
ls -l dist/web
