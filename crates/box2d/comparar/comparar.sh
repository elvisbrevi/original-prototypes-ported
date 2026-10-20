#!/bin/sh
# Compara la traducción con el Box2D 2.3.1 original: compila el Box2D del box2d.js de kripken
# (el que trae Construct 2, con b2_maxPolygonVertices = 16 como en la compilación de Scirra) con
# escenas.cpp, y las mismas escenas en Rust (examples/escenas.rs); las trayectorias tienen que
# coincidir bit a bit.
# Uso: crates/box2d/comparar/comparar.sh [carpeta de trabajo]
set -e
raiz="$(cd "$(dirname "$0")/../../.." && pwd)"
trabajo="${1:-/tmp/box2d-comparar}"
commit=1e6e9da36a3160a9f40cda4cddcb4f83abb4d82c
mkdir -p "$trabajo"
if [ ! -d "$trabajo/box2d.js" ]; then
    git clone -q https://github.com/kripken/box2d.js.git "$trabajo/box2d.js"
fi
git -C "$trabajo/box2d.js" checkout -q "$commit"
rm -rf "$trabajo/Box2D"
cp -r "$trabajo/box2d.js/Box2D_v2.3.1/Box2D" "$trabajo/Box2D"
sed -i 's/#define b2_maxPolygonVertices\t8/#define b2_maxPolygonVertices\t16/' "$trabajo/Box2D/Common/b2Settings.h"
g++ -O2 -std=c++11 -I"$trabajo" "$raiz/crates/box2d/comparar/escenas.cpp" \
    $(find "$trabajo/Box2D" -name '*.cpp') -o "$trabajo/escenas"
cargo build --release -q -p box2d --example escenas --manifest-path "$raiz/Cargo.toml"
for escena in caja pila bala moto corredor; do
    "$trabajo/escenas" "$escena" > "$trabajo/c_$escena.txt"
    "$raiz/target/release/examples/escenas" "$escena" > "$trabajo/rs_$escena.txt"
    if cmp -s "$trabajo/c_$escena.txt" "$trabajo/rs_$escena.txt"; then
        echo "$escena: idéntica ($(wc -l < "$trabajo/rs_$escena.txt") pasos)"
    else
        echo "$escena: DIFIERE"
        exit 1
    fi
done
