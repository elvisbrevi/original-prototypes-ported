"""Desensamblador mínimo de píxel shaders ps_2_0 (Direct3D 9) dentro de un efecto XNA (.xnb).

Uso: python3 -I ps20_desensamblar.py <efecto.xnb>
Solo cubre las instrucciones que usan los efectos de Proyecto Nutria; sirve para leer las
constantes y las operaciones exactas y reescribirlas en GLSL.
"""
import struct
import sys

OPS = {0x01: "mov", 0x02: "add", 0x03: "sub", 0x04: "mad", 0x05: "mul", 0x06: "rcp", 0x07: "rsq",
       0x08: "dp3", 0x09: "dp4", 0x0A: "min", 0x0B: "max", 0x0C: "slt", 0x0D: "sge", 0x0E: "exp",
       0x0F: "log", 0x12: "lrp", 0x13: "frc", 0x1F: "dcl", 0x20: "pow", 0x23: "abs", 0x24: "nrm",
       0x42: "texld", 0x51: "def", 0x58: "cmp", 0x5A: "dp2add"}
TIPOS = {0: "r", 1: "v", 2: "c", 3: "t", 8: "oC", 9: "oDepth", 10: "s"}


def registro(t, destino):
    tipo = ((t >> 28) & 7) | ((t >> 8) & 0x18)
    nombre = f"{TIPOS.get(tipo, f'?{tipo}')}{t & 0x7FF}"
    if destino:
        mascara = (t >> 16) & 0xF
        if mascara != 0xF:
            nombre += "." + "".join(c for i, c in enumerate("xyzw") if mascara >> i & 1)
        return nombre
    swz = (t >> 16) & 0xFF
    comp = "".join("xyzw"[(swz >> (2 * i)) & 3] for i in range(4))
    if comp != "xyzw":
        nombre += "." + (comp[0] if len(set(comp)) == 1 else comp)
    mod = (t >> 24) & 0xF
    return {0: nombre, 1: f"-{nombre}", 0xB: f"abs({nombre})", 0xC: f"-abs({nombre})"}.get(mod, f"mod{mod}({nombre})")


def main(ruta):
    d = open(ruta, "rb").read()
    inicio = d.find(struct.pack("<I", 0xFFFF0200))
    if inicio < 0:
        sys.exit("no hay bytecode ps_2_0")
    p = inicio + 4
    print(f"ps_2_0 en el desplazamiento {inicio}")
    while p < len(d):
        token = struct.unpack_from("<I", d, p)[0]
        p += 4
        if token == 0x0000FFFF:
            print("end")
            break
        op = token & 0xFFFF
        if op == 0xFFFE:
            largo = (token >> 16) & 0x7FFF
            etiqueta = d[p:p + 4]
            print(f"; comentario {etiqueta!r} ({largo} dwords)")
            p += 4 * largo
            continue
        largo = (token >> 24) & 0xF
        args = struct.unpack_from(f"<{largo}I", d, p)
        p += 4 * largo
        nombre = OPS.get(op, f"op{op:#x}")
        if op == 0x51:
            valores = struct.unpack("<4f", struct.pack("<4I", *args[1:5]))
            print(f"def {registro(args[0], True)}, " + ", ".join(f"{v:.9g}" for v in valores))
        elif op == 0x1F:
            print(f"dcl {registro(args[1], True)}")
        else:
            sat = "_sat" if (args[0] >> 20) & 1 else ""
            print(f"{nombre}{sat} {registro(args[0], True)}, " + ", ".join(registro(a, False) for a in args[1:]))


if __name__ == "__main__":
    main(sys.argv[1])
