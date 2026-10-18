"""Extract IMAN's XNA 3.1 content (uncompressed XNB v4 + XACT wave bank) to PNG/WAV/text.

Usage: python3 -I xnb_extract.py <Content dir> <output assets dir>
"""
import os
import struct
import sys
import zlib


class Reader:
    def __init__(self, data, pos=0):
        self.d = data
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
        result = shift = 0
        while True:
            b = self.u8()
            result |= (b & 0x7F) << shift
            if not b & 0x80:
                return result
            shift += 7

    def string(self):
        return self.bytes(self.var()).decode("utf-8")

    def char(self):
        # .NET BinaryReader.ReadChar with UTF-8 encoding.
        b = self.u8()
        if b < 0x80:
            return chr(b)
        n = 2 if b >> 5 == 0b110 else 3 if b >> 4 == 0b1110 else 4
        return (bytes([b]) + self.bytes(n - 1)).decode("utf-8")

    def rect(self):
        return struct.unpack_from("<4i", self.bytes(16))


def write_png(path, w, h, rgba):
    raw = b"".join(b"\x00" + rgba[y * w * 4:(y + 1) * w * 4] for y in range(h))

    def chunk(tag, body):
        c = tag + body
        return struct.pack(">I", len(body)) + c + struct.pack(">I", zlib.crc32(c) & 0xFFFFFFFF)

    png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0))
    png += chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b"")
    with open(path, "wb") as f:
        f.write(png)


def open_xnb(path):
    data = open(path, "rb").read()
    if data[:3] != b"XNB" or data[4] != 4 or data[5] & 0x80:
        raise ValueError(f"{path}: unsupported XNB header {data[:6]!r}")
    r = Reader(data, 10)
    readers = [(r.string(), r.i32()) for _ in range(r.var())]
    if r.var() != 0:
        raise ValueError(f"{path}: shared resources not supported")
    return r, [name for name, _ in readers]


def rgb565(c):
    r, g, b = (c >> 11) & 31, (c >> 5) & 63, c & 31
    return (r << 3 | r >> 2, g << 2 | g >> 4, b << 3 | b >> 2)


def decode_dxt3(data, w, h):
    out = bytearray(w * h * 4)
    blocks_x = (w + 3) // 4
    for by in range((h + 3) // 4):
        for bx in range(blocks_x):
            o = (by * blocks_x + bx) * 16
            alpha = int.from_bytes(data[o:o + 8], "little")
            c0, c1, idx = struct.unpack_from("<HHI", data, o + 8)
            p0, p1 = rgb565(c0), rgb565(c1)
            palette = [p0, p1,
                       tuple((2 * a + b) // 3 for a, b in zip(p0, p1)),
                       tuple((a + 2 * b) // 3 for a, b in zip(p0, p1))]
            for i in range(16):
                x, y = bx * 4 + i % 4, by * 4 + i // 4
                if x < w and y < h:
                    a = (alpha >> (4 * i)) & 15
                    out[(y * w + x) * 4:(y * w + x) * 4 + 4] = bytes(palette[(idx >> (2 * i)) & 3] + (a * 17,))
    return bytes(out)


def read_texture(r):
    fmt, w, h, mips = r.i32(), r.u32(), r.u32(), r.u32()
    size = r.u32()
    data = r.bytes(size)
    for _ in range(mips - 1):
        r.bytes(r.u32())
    if fmt == 30:  # SurfaceFormat.Dxt3
        return w, h, decode_dxt3(data, w, h)
    if fmt != 1:  # SurfaceFormat.Color (A8R8G8B8, stored BGRA)
        raise ValueError(f"surface format {fmt} not supported")
    rgba = bytearray(data)
    rgba[0::4], rgba[2::4] = data[2::4], data[0::4]
    return w, h, bytes(rgba)


def extract_texture(src, dst):
    r, readers = open_xnb(src)
    assert "Texture2DReader" in readers[r.var() - 1]
    w, h, rgba = read_texture(r)
    write_png(dst, w, h, rgba)
    return w, h


def extract_font(src, dst_png, dst_txt):
    r, readers = open_xnb(src)
    assert "SpriteFontReader" in readers[r.var() - 1]
    r.var()
    w, h, rgba = read_texture(r)
    write_png(dst_png, w, h, rgba)

    def rect_list():
        r.var()
        return [r.rect() for _ in range(r.u32())]

    glyphs = rect_list()
    cropping = rect_list()
    r.var()
    chars = [r.char() for _ in range(r.u32())]
    line_spacing = r.i32()
    spacing = r.f32()
    r.var()
    kerning = [struct.unpack("<3f", r.bytes(12)) for _ in range(r.u32())]
    default = r.char() if r.u8() else None
    assert r.p == len(r.d), (r.p, len(r.d))
    assert len(glyphs) == len(cropping) == len(chars) == len(kerning)

    with open(dst_txt, "w") as f:
        f.write("# IMAN SpriteFont: line_spacing spacing default_char, then per glyph:\n")
        f.write("# codepoint glyph(x y w h) cropping(x y w h) kerning(left width right)\n")
        f.write(f"{line_spacing} {spacing} {ord(default) if default else -1}\n")
        for c, g, cr, k in zip(chars, glyphs, cropping, kerning):
            f.write(f"{ord(c)} {' '.join(map(str, g))} {' '.join(map(str, cr))} {k[0]} {k[1]} {k[2]}\n")
    return w, h, len(chars)


def extract_wavebank(src, dst):
    d = open(src, "rb").read()
    assert d[:4] == b"WBND"
    _, header_version = struct.unpack_from("<II", d, 4)
    regions = [struct.unpack_from("<II", d, 12 + i * 8) for i in range(5)]
    bank_off = regions[0][0]
    count = struct.unpack_from("<I", d, bank_off + 4)[0]
    meta_size = struct.unpack_from("<I", d, bank_off + 72)[0]
    meta_off, data_off = regions[1][0], regions[4][0]
    out = []
    for i in range(count):
        flags_dur, fmt, play_off, play_len = struct.unpack_from("<4I", d, meta_off + i * meta_size)
        tag, channels = fmt & 3, (fmt >> 2) & 7
        rate, align, bits16 = (fmt >> 5) & 0x3FFFF, (fmt >> 23) & 0xFF, fmt >> 31
        if tag != 0:
            raise ValueError(f"wave {i}: format tag {tag} (only PCM supported)")
        if channels != 1:
            raise ValueError(f"wave {i}: {channels} channels (only mono supported)")
        pcm = d[data_off + play_off:data_off + play_off + play_len]
        if bits16:
            src = [s / 32768 for s in struct.unpack(f"<{len(pcm) // 2}h", pcm)]
        else:
            src = [(b - 128) / 128 for b in pcm]
        # quad-snd plays every buffer at 44.1 kHz without resampling, so resample here.
        out_rate = 44100
        n = len(src) * out_rate // rate
        samples = []
        for j in range(n):
            t = j * rate / out_rate
            k = int(t)
            a, b = src[k], src[min(k + 1, len(src) - 1)]
            samples.append(round(max(-1.0, min(1.0, a + (b - a) * (t - k))) * 32767))
        body = struct.pack(f"<{n}h", *samples)
        path = dst if count == 1 else dst.replace(".wav", f"_{i}.wav")
        with open(path, "wb") as f:
            f.write(b"RIFF" + struct.pack("<I", 36 + len(body)) + b"WAVEfmt ")
            f.write(struct.pack("<IHHIIHH", 16, 1, 1, out_rate, out_rate * 2, 2, 16))
            f.write(b"data" + struct.pack("<I", len(body)) + body)
        out.append((path, rate, 16 if bits16 else 8, len(src)))
    return header_version, out


def main(content, out):
    textures = {
        "Graficos/Personajes/niño": "personajes/nino.png",
        "Graficos/Personajes/iman": "personajes/iman.png",
        "Graficos/Menu/titulo": "menu/titulo.png",
        "Graficos/Menu/cortina": "menu/cortina.png",
        "Graficos/Ramas/_01": "ramas/01.png",
        "Shaders/SketchTexture": "shaders/sketch.png",
    }
    for kind in ("Fronts", "Suelos", "Backgrounds", "Cielos"):
        for n in ("01", "02"):
            textures[f"Graficos/{kind}/_{n}"] = f"{kind.lower()}/{n}.png"
    for n in ("01", "02", "03", "04", "05"):
        textures[f"Graficos/Plataformas/_{n}"] = f"plataformas/{n}.png"
    textures["Graficos/Plataformas/null"] = "plataformas/null.png"

    for src, dst in sorted(textures.items()):
        os.makedirs(os.path.join(out, os.path.dirname(dst)), exist_ok=True)
        w, h = extract_texture(os.path.join(content, src + ".xnb.deploy"), os.path.join(out, dst))
        print(f"texture {dst:28} {w}x{h}")

    os.makedirs(os.path.join(out, "fuente"), exist_ok=True)
    w, h, n = extract_font(os.path.join(content, "Textos/fuente.xnb.deploy"),
                           os.path.join(out, "fuente/fuente.png"), os.path.join(out, "fuente/fuente.txt"))
    print(f"font {w}x{h}, {n} glyphs")

    os.makedirs(os.path.join(out, "audio"), exist_ok=True)
    version, waves = extract_wavebank(os.path.join(content, "Audio/Wave Bank.xwb.deploy"),
                                      os.path.join(out, "audio/jump.wav"))
    print(f"wavebank header v{version}: {waves}")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
