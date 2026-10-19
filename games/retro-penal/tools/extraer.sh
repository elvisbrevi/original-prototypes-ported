#!/bin/sh
# Regenera assets/ desde la exportación HTML5 original de Construct 2: las imágenes y los sonidos
# se usan tal cual, y data.js (el proyecto: tipos, animaciones, layouts) se lee al arrancar.
# Solo se copian los OGG que suenan: chute, comenzar3 y pitazo3 se precargan pero ningún evento
# los reproduce. Las fuentes (Liberation Sans, en lugar de la "Arial" del sistema) están en
# crates/construct2/fuentes.
# Uso: tools/extraer.sh <carpeta html5>
set -e
cd "$(dirname "$0")/.."
html5="${1:?uso: tools/extraer.sh <carpeta html5>}"
rm -rf assets
mkdir -p assets/images assets/media
cp "$html5"/images/*.png assets/images/
for sonido in click3 gol3 musica3 perdio3 publico3 seloperdio3 tiro3; do
    cp "$html5/media/$sonido.ogg" assets/media/
done
cp "$html5/data.js" assets/data.js
ls -R assets
