#!/bin/sh
# Compila la versión web de un juego en dist/<juego> (index.html + mq_js_bundle.js + <juego>.wasm).
# Uso: tools/build_web.sh <juego>      (por ejemplo: tools/build_web.sh iman)
#
# El index.html sale de la plantilla crates/museo/web/index.html con el título del juego, que se lee
# de `[package.metadata.museo] titulo = "..."` en games/<juego>/Cargo.toml.
set -e
cd "$(dirname "$0")/.."
juego="${1:?uso: tools/build_web.sh <juego>}"
manifiesto="games/$juego/Cargo.toml"
titulo=$(sed -n 's/^titulo *= *"\(.*\)"$/\1/p' "$manifiesto")
[ -n "$titulo" ] || { echo "falta [package.metadata.museo] titulo en $manifiesto" >&2; exit 1; }

cargo build --release --target wasm32-unknown-unknown -p "$juego"
destino="dist/$juego"
rm -rf "$destino"
mkdir -p "$destino"
cp "target/wasm32-unknown-unknown/release/$juego.wasm" "$destino/"

# El título va escapado para HTML y para el reemplazo de sed.
titulo_html=$(printf '%s' "$titulo" | sed 's/&/\&amp;/g; s/</\&lt;/g; s/>/\&gt;/g; s/[\/&|]/\\&/g')
sed "s|{{TITULO}}|$titulo_html|; s|{{WASM}}|$juego.wasm|" crates/museo/web/index.html >"$destino/index.html"

# mq_js_bundle.js de macroquad 0.4.16 asigna `register_plugin` sin declararla dentro de un bloque
# "use strict" (plugin de quad-net) y lanza un ReferenceError al cargar. Se declara la variable.
original='function(){function i(){}register_plugin=function(e){e.env.ws_connect'
arreglado='function(){var register_plugin;function i(){}register_plugin=function(e){e.env.ws_connect'
grep -qF "$original" crates/museo/web/mq_js_bundle.js || { echo "mq_js_bundle.js cambió: revisar el parche" >&2; exit 1; }
sed "s|$original|$arreglado|" crates/museo/web/mq_js_bundle.js >"$destino/mq_js_bundle.js"
ls -l "$destino"
