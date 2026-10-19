#!/bin/sh
# Regenera assets/ desde la exportación HTML5 original de Construct 2: las imágenes y los sonidos
# se usan tal cual, y data.js (el proyecto: tipos, animaciones, comportamientos, layouts) se lee
# al arrancar. Las fuentes (Liberation Sans, en lugar de la "Arial" del sistema) están en
# crates/construct2/fuentes.
# Uso: tools/extraer.sh <carpeta html5>
set -e
cd "$(dirname "$0")/.."
html5="${1:?uso: tools/extraer.sh <carpeta html5>}"
rm -rf assets
mkdir -p assets/images assets/media
cp "$html5"/images/*.png assets/images/
cp "$html5"/media/*.ogg assets/media/
cp "$html5/data.js" assets/data.js
ls -R assets
