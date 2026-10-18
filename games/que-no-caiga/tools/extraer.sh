#!/bin/sh
# Regenera assets/ desde la exportación HTML5 original de Construct 2: las imágenes y los sonidos
# se usan tal cual, y data.js (el proyecto: tipos, animaciones, layouts) se lee al arrancar.
# Las fuentes son Liberation Sans 2.1.5 (SIL OFL 1.1), con las métricas de Arial: el original
# usaba las fuentes del sistema ("Arial" y "Arial Black").
# Uso: tools/extraer.sh <carpeta html5> [carpeta con LiberationSans-*.ttf]
set -e
cd "$(dirname "$0")/.."
html5="${1:?uso: tools/extraer.sh <carpeta html5> [carpeta de fuentes]}"
fuentes="${2:-/usr/share/fonts/truetype/liberation}"
rm -rf assets
mkdir -p assets/images assets/media assets/fuentes
cp "$html5"/images/*.png assets/images/
cp "$html5"/media/*.ogg assets/media/
cp "$html5/data.js" assets/data.js
cp "$fuentes/LiberationSans-Regular.ttf" "$fuentes/LiberationSans-Bold.ttf" assets/fuentes/
cp /usr/share/doc/fonts-liberation/copyright assets/fuentes/LICENCIA-OFL.txt
ls -R assets
