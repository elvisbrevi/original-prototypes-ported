#!/bin/sh
# Compila la versión web de un juego en dist/<juego> (index.html + mq_js_bundle.js + <juego>.wasm).
# Uso: tools/build_web.sh <juego>      (por ejemplo: tools/build_web.sh iman)
set -e
cd "$(dirname "$0")/.."
juego="${1:?uso: tools/build_web.sh <juego>}"
cargo build --release --target wasm32-unknown-unknown -p "$juego"
rm -rf "dist/$juego"
mkdir -p "dist/$juego"
cp "games/$juego/web/index.html" "games/$juego/web/mq_js_bundle.js" "dist/$juego/"
cp "target/wasm32-unknown-unknown/release/$juego.wasm" "dist/$juego/"
ls -l "dist/$juego"
