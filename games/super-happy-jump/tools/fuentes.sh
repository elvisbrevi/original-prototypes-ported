#!/bin/sh
# Regenera fuentes/: Nunito (SIL OFL 1.1, de google/fonts) con los pesos 800 y 900, en lugar de la
# Arial Rounded MT Bold que trae el original (de Monotype, no se puede redistribuir). Necesita
# fontTools (por ejemplo en el venv de UnityPy: ~/.venvs/museo/bin/pip install fonttools).
# Uso: tools/fuentes.sh [python con fontTools]
set -e
cd "$(dirname "$0")/.."
python="${1:-$HOME/.venvs/museo/bin/python}"
commit=604936664fd62c14271209b51f98e7f495dd1a3e
base="https://raw.githubusercontent.com/google/fonts/$commit/ofl/nunito"
trabajo=$(mktemp -d)
curl -sSfL -o "$trabajo/Nunito.ttf" "$base/Nunito%5Bwght%5D.ttf"
echo "bb55a5ca5c2042335b3991af27c4d0705d0ef41cac6164ac737fd8f2a1e85207  $trabajo/Nunito.ttf" | sha256sum -c -
mkdir -p fuentes
curl -sSfL -o fuentes/LICENCIA-OFL.txt "$base/OFL.txt"
"$python" -I -m fontTools.varLib.instancer "$trabajo/Nunito.ttf" wght=800 --update-name-table -o fuentes/Nunito-ExtraBold.ttf
"$python" -I -m fontTools.varLib.instancer "$trabajo/Nunito.ttf" wght=900 --update-name-table -o fuentes/Nunito-Black.ttf
rm -r "$trabajo"
ls -l fuentes
