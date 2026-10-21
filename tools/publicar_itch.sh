#!/bin/sh
# Publica la versión web de un juego en su página de itch.io: la compila con tools/build_web.sh y la
# sube con butler al canal html5. Las descargas originales de la página no se tocan.
# Uso: tools/publicar_itch.sh <juego>      (antes: butler login; la API key no entra al repo)
set -e
cd "$(dirname "$0")/.."
juego="${1:?uso: tools/publicar_itch.sh <juego>}"
# El proyecto de itch.io de cada juego (casi todos se llaman como su carpeta).
case "$juego" in
    one-button-prototype) proyecto=prototipo-2000 ;;
    *) proyecto="$juego" ;;
esac
tools/build_web.sh "$juego"
butler push "dist/$juego" "elvisbrevi/$proyecto:html5" --userversion "$(git rev-parse --short HEAD)"
