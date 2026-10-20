"""Regenera assets/ desde el APK de The Meaning of Christmas (Java + libGDX 0.9.2, Android).

Uso:
    python3 -I games/the-meaning-of-christmas/tools/extraer.py <The Meaning Of Christmas.apk>

Escribe en games/the-meaning-of-christmas/assets/:
    imagenes/<nombre>.png   las imágenes de assets/data que carga el código (campanas.png,
                            game over.png y monedas.png vienen en el APK pero no se usan)
    fuente/arial-15.fnt     la fuente por defecto de BitmapFont, que libGDX empaqueta en el APK
    fuente/arial-15.png
    musica/<nombre>.ogg     las dos músicas, de MP3 a Ogg Vorbis con ffmpeg (macroquad no lee
                            MP3)

El APK es un zip y solo se lee: nada de él se ejecuta. ffmpeg recibe los MP3 como datos.
"""
import hashlib
import os
import shutil
import subprocess
import sys
import tempfile
import zipfile

RAIZ = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "assets")
SHA256 = "1b0d85a222e8bdc0234387f4a0fdadfa7c153bac6958285c6d04d961ce761cc6"
IMAGENES = ["titulo", "imagen titulo", "noche", "luna", "dinero", "franja", "regalo1", "regalo2",
            "regalo3", "pixel"]
FUENTE = ["arial-15.fnt", "arial-15.png"]
MUSICA = ["noche de paz", "angel"]


def main(apk):
    with open(apk, "rb") as f:
        digest = hashlib.sha256(f.read()).hexdigest()
    if digest != SHA256:
        sys.exit(f"sha256 inesperado: {digest}")
    for sub in ("imagenes", "fuente", "musica"):
        os.makedirs(os.path.join(RAIZ, sub), exist_ok=True)
    with zipfile.ZipFile(apk) as z:
        for n in IMAGENES:
            with open(os.path.join(RAIZ, "imagenes", n + ".png"), "wb") as out:
                out.write(z.read(f"assets/data/{n}.png"))
        for n in FUENTE:
            with open(os.path.join(RAIZ, "fuente", n), "wb") as out:
                out.write(z.read(f"com/badlogic/gdx/utils/{n}"))
        with tempfile.TemporaryDirectory() as tmp:
            for n in MUSICA:
                mp3 = os.path.join(tmp, "entrada.mp3")
                with open(mp3, "wb") as out:
                    out.write(z.read(f"assets/data/{n}.mp3"))
                ogg = os.path.join(RAIZ, "musica", n + ".ogg")
                subprocess.run(["ffmpeg", "-v", "error", "-y", "-i", mp3, "-map_metadata", "-1",
                                "-fflags", "+bitexact", "-flags:a", "+bitexact",
                                "-c:a", "libvorbis", "-q:a", "4", ogg], check=True)
    print("imágenes:", len(IMAGENES), "fuente:", len(FUENTE), "música:", len(MUSICA))


if __name__ == "__main__":
    if len(sys.argv) != 2 or not shutil.which("ffmpeg"):
        sys.exit(__doc__)
    main(sys.argv[1])
