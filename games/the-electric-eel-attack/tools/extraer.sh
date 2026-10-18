#!/bin/sh
# Regenera assets/ desde la carpeta Content del original (los .xnb y los mapas XML).
# Uso: tools/extraer.sh "<ruta>/Application Files/The Electric Eel Attack_1_0_0_13/Content"
set -e
cd "$(dirname "$0")/.."
contenido="${1:?uso: tools/extraer.sh <carpeta Content>}"
rm -rf assets
python3 -I ../../tools/xnb_extract.py "$contenido" assets tools/contenido.txt
mkdir -p assets/Mapas
for mapa in titulo nivel1 nivel2 nivel3 nivel4; do
    cp "$contenido/Mapas/$mapa.deploy" "assets/Mapas/$mapa.xml"
done
