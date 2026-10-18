"""Convierte contenido XNA 3.1 (XNB v4 sin comprimir) a PNG y texto, conservando las rutas.

Uso: python3 -I tools/xnb_extract.py <carpeta Content> <salida> [lista.txt]

- Texture2D (Color, DXT1, DXT3, DXT5) -> <ruta>.png
- SpriteFont -> <ruta>.png (textura) + <ruta>.txt (glifos, recortes, kerning; formato de
  crates/xna: line_spacing spacing default_char y una línea por glifo)
- SoundEffect -> <ruta>.wav con el mismo formato PCM del original (quad-snd y WebAudio remuestrean)
Si se da una lista, solo se convierten las rutas de contenido que tenga (una por línea, sin
extensión, como en Content.Load: "Graficos/Personajes/coipo"); las líneas con # son comentarios.
Los archivos de ClickOnce (.xnb.deploy) se aceptan igual que los .xnb.

Basado en games/iman/tools/xnb_extract.py. Los originales son datos no confiables: solo se leen.
"""
import os
import struct
import sys
import zlib


class Lector:
    def __init__(self, datos, pos=0):
        self.d = datos
        self.p = pos

    def u8(self):
        v = self.d[self.p]
        self.p += 1
        return v

    def i32(self):
        v = struct.unpack_from("<i", self.d, self.p)[0]
        self.p += 4
        return v

    def u32(self):
        v = struct.unpack_from("<I", self.d, self.p)[0]
        self.p += 4
        return v

    def f32(self):
        v = struct.unpack_from("<f", self.d, self.p)[0]
        self.p += 4
        return v

    def bytes(self, n):
        v = self.d[self.p:self.p + n]
        self.p += n
        return v

    def var(self):
        resultado = desplazamiento = 0
        while True:
            b = self.u8()
            resultado |= (b & 0x7F) << desplazamiento
            if not b & 0x80:
                return resultado
            desplazamiento += 7

    def cadena(self):
        return self.bytes(self.var()).decode("utf-8")

    def caracter(self):
        # BinaryReader.ReadChar de .NET con codificación UTF-8.
        b = self.u8()
        if b < 0x80:
            return chr(b)
        n = 2 if b >> 5 == 0b110 else 3 if b >> 4 == 0b1110 else 4
        return (bytes([b]) + self.bytes(n - 1)).decode("utf-8")

    def rect(self):
        return struct.unpack_from("<4i", self.bytes(16))


def escribir_png(ruta, w, h, rgba):
    filas = b"".join(b"\x00" + rgba[y * w * 4:(y + 1) * w * 4] for y in range(h))

    def trozo(tipo, cuerpo):
        c = tipo + cuerpo
        return struct.pack(">I", len(cuerpo)) + c + struct.pack(">I", zlib.crc32(c) & 0xFFFFFFFF)

    png = b"\x89PNG\r\n\x1a\n" + trozo(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0))
    png += trozo(b"IDAT", zlib.compress(filas, 9)) + trozo(b"IEND", b"")
    with open(ruta, "wb") as f:
        f.write(png)


def abrir_xnb(ruta):
    datos = open(ruta, "rb").read()
    if datos[:3] != b"XNB" or datos[4] != 4:
        raise ValueError(f"cabecera XNB no soportada {datos[:6]!r}")
    if datos[5] & 0x80:
        raise ValueError("XNB comprimido (LZX): no soportado")
    r = Lector(datos, 10)
    lectores = [(r.cadena(), r.i32()) for _ in range(r.var())]
    if r.var() != 0:
        raise ValueError("recursos compartidos no soportados")
    return r, [nombre.split(",")[0].rsplit(".", 1)[-1] for nombre, _ in lectores]


def rgb565(c):
    r, g, b = (c >> 11) & 31, (c >> 5) & 63, c & 31
    return (r << 3 | r >> 2, g << 2 | g >> 4, b << 3 | b >> 2)


def decodificar_dxt(datos, w, h, formato):
    """DXT1 (formato 28), DXT3 (30) y DXT5 (32) de XNA 3.1."""
    salida = bytearray(w * h * 4)
    tam_bloque = 8 if formato == 28 else 16
    bloques_x = (w + 3) // 4
    for by in range((h + 3) // 4):
        for bx in range(bloques_x):
            o = (by * bloques_x + bx) * tam_bloque
            alfas = None
            if formato == 30:
                bits = int.from_bytes(datos[o:o + 8], "little")
                alfas = [((bits >> (4 * i)) & 15) * 17 for i in range(16)]
            elif formato == 32:
                a0, a1 = datos[o], datos[o + 1]
                bits = int.from_bytes(datos[o + 2:o + 8], "little")
                tabla = [a0, a1] + ([((6 - i) * a0 + (i + 1) * a1) // 7 for i in range(6)] if a0 > a1 else
                                    [((4 - i) * a0 + (i + 1) * a1) // 5 for i in range(4)] + [0, 255])
                alfas = [tabla[(bits >> (3 * i)) & 7] for i in range(16)]
            oc = o + (0 if formato == 28 else 8)
            c0, c1, indices = struct.unpack_from("<HHI", datos, oc)
            p0, p1 = rgb565(c0), rgb565(c1)
            if formato != 28 or c0 > c1:
                paleta = [p0 + (255,), p1 + (255,),
                          tuple((2 * a + b) // 3 for a, b in zip(p0, p1)) + (255,),
                          tuple((a + 2 * b) // 3 for a, b in zip(p0, p1)) + (255,)]
            else:
                paleta = [p0 + (255,), p1 + (255,),
                          tuple((a + b) // 2 for a, b in zip(p0, p1)) + (255,), (0, 0, 0, 0)]
            for i in range(16):
                x, y = bx * 4 + i % 4, by * 4 + i // 4
                if x < w and y < h:
                    color = paleta[(indices >> (2 * i)) & 3]
                    if alfas is not None:
                        color = color[:3] + (alfas[i],)
                    salida[(y * w + x) * 4:(y * w + x) * 4 + 4] = bytes(color)
    return bytes(salida)


def leer_textura(r):
    formato, w, h, mips = r.i32(), r.u32(), r.u32(), r.u32()
    datos = r.bytes(r.u32())
    for _ in range(mips - 1):
        r.bytes(r.u32())
    if formato in (28, 30, 32):
        return w, h, decodificar_dxt(datos, w, h, formato)
    if formato != 1:  # SurfaceFormat.Color (A8R8G8B8, guardado como BGRA)
        raise ValueError(f"formato de superficie {formato} no soportado")
    rgba = bytearray(datos)
    rgba[0::4], rgba[2::4] = datos[2::4], datos[0::4]
    return w, h, bytes(rgba)


def textura(r, destino):
    w, h, rgba = leer_textura(r)
    escribir_png(destino + ".png", w, h, rgba)
    return f"textura {w}x{h}"


def fuente(r, destino):
    r.var()
    w, h, rgba = leer_textura(r)
    escribir_png(destino + ".png", w, h, rgba)

    def rectangulos():
        r.var()
        return [r.rect() for _ in range(r.u32())]

    glifos = rectangulos()
    recortes = rectangulos()
    r.var()
    caracteres = [r.caracter() for _ in range(r.u32())]
    interlineado = r.i32()
    espaciado = r.f32()
    r.var()
    kerning = [struct.unpack("<3f", r.bytes(12)) for _ in range(r.u32())]
    por_defecto = r.caracter() if r.u8() else None
    assert r.p == len(r.d), (r.p, len(r.d))
    assert len(glifos) == len(recortes) == len(caracteres) == len(kerning)
    with open(destino + ".txt", "w") as f:
        f.write("# SpriteFont: line_spacing spacing default_char, y por glifo:\n")
        f.write("# codepoint glyph(x y w h) cropping(x y w h) kerning(left width right)\n")
        f.write(f"{interlineado} {espaciado} {ord(por_defecto) if por_defecto else -1}\n")
        for c, g, cr, k in zip(caracteres, glifos, recortes, kerning):
            f.write(f"{ord(c)} {' '.join(map(str, g))} {' '.join(map(str, cr))} {k[0]} {k[1]} {k[2]}\n")
    return f"fuente {w}x{h}, {len(caracteres)} glifos"


def sonido(r, destino):
    formato = r.bytes(r.u32())
    datos = r.bytes(r.u32())
    inicio_bucle, largo_bucle, duracion_ms = r.i32(), r.i32(), r.i32()
    assert r.p == len(r.d), (r.p, len(r.d))
    etiqueta, canales, frecuencia, bytes_por_segundo, alineacion, bits = struct.unpack_from("<HHIIHH", formato)
    if etiqueta != 1:
        raise ValueError(f"formato de audio {etiqueta} no soportado (solo PCM)")
    fmt = struct.pack("<HHIIHH", etiqueta, canales, frecuencia, bytes_por_segundo, alineacion, bits)
    with open(destino + ".wav", "wb") as f:
        f.write(b"RIFF" + struct.pack("<I", 4 + 8 + len(fmt) + 8 + len(datos)) + b"WAVE")
        f.write(b"fmt " + struct.pack("<I", len(fmt)) + fmt)
        f.write(b"data" + struct.pack("<I", len(datos)) + datos)
    return f"sonido PCM {canales} canal(es), {frecuencia} Hz, {bits} bits, {duracion_ms} ms"


CONVERSORES = {"Texture2DReader": textura, "SpriteFontReader": fuente, "SoundEffectReader": sonido}


def main(contenido, salida, lista=None):
    elegidos = None
    if lista:
        with open(lista, encoding="utf-8") as f:
            elegidos = {l.strip() for l in f if l.strip() and not l.startswith("#")}
    hechos = set()
    for raiz, _, archivos in sorted(os.walk(contenido)):
        for nombre in sorted(archivos):
            base = nombre.removesuffix(".deploy")
            if not base.endswith(".xnb"):
                continue
            relativa = os.path.relpath(os.path.join(raiz, base[:-4]), contenido).replace(os.sep, "/")
            if elegidos is not None and relativa not in elegidos:
                continue
            r, lectores = abrir_xnb(os.path.join(raiz, nombre))
            principal = lectores[r.var() - 1]
            if principal not in CONVERSORES:
                print(f"{relativa:50} {principal}: no se convierte")
                continue
            destino = os.path.join(salida, relativa)
            os.makedirs(os.path.dirname(destino), exist_ok=True)
            print(f"{relativa:50} {CONVERSORES[principal](r, destino)}")
            hechos.add(relativa)
    if elegidos is not None and elegidos - hechos:
        sys.exit(f"faltan en {contenido}: {sorted(elegidos - hechos)}")


if __name__ == "__main__":
    if len(sys.argv) not in (3, 4):
        sys.exit(__doc__)
    main(*sys.argv[1:])
