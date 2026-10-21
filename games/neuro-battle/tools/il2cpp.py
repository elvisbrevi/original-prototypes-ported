"""Lee los metadatos IL2CPP de Neuro Battle (Unity 5.1.1f1, WebGL) y vuelca el código de sus scripts.

Uso:
    python3 -I games/neuro-battle/tools/il2cpp.py <carpeta Release> <salida>

En Unity 5.1.1 IL2CPP todavía no escribía un global-metadata.dat: los metadatos son datos
estáticos de C++ compilados al asm.js, así que viven en su memoria inicial (`Neuro Battle.html.mem`,
que Emscripten carga en la dirección 8). Lo que se usa de ellos:

    Il2CppMetadataRegistration   ocho asambleas, la tabla de los Il2CppType (5166), los métodos
                                 genéricos con su función y la tabla que resuelven las referencias
                                 a métodos del código generado
    Il2CppImage (0x40 bytes)     nombre, lista de TypeInfo, nombres y definiciones de los campos
                                 (índice del nombre, índice del tipo, offset)
    TypeInfo (0xa8 bytes)        nombre, espacio de nombres, lista de MethodInfo, tamaño de
                                 instancia, flags, cantidad de métodos y de campos
    MethodInfo (0x38 bytes)      nombre, índice de la función en la tabla de su firma, tipo que lo
                                 declara, tipo de retorno, parámetros, flags
    String literal               objeto String estático: clase, monitor, largo y UTF-16

En el asm.js (minificado) las funciones de cada firma están en una tabla, que se identifica por
el dynCall_<firma> que exporta el módulo; con el índice del MethodInfo se llega al nombre de la
función y a su cuerpo. Los métodos estáticos también reciben un primer argumento (nulo), y todos
terminan con el MethodInfo* oculto (casi siempre nulo).

Escribe en <salida>:
    metadatos.txt   los tipos de Assembly-CSharp y Assembly-UnityScript con sus campos (tipo,
                    offset) y sus métodos (firma y función del asm.js)
    motor.txt       los métodos de UnityEngine y mscorlib que llaman los scripts, con su función
    asmjs/<Tipo>.js el cuerpo de cada método de los scripts del juego, con las llamadas, las
                    referencias a tipos y métodos, y los textos anotados en comentarios

Los originales son datos no confiables: solo se leen como bytes y texto, nunca se ejecutan.
"""
import bisect
import os
import re
import struct
import sys

BASE = 8  # STATIC_BASE: la dirección donde Emscripten carga la memoria inicial.
REGISTRO = 0x294EB0  # Il2CppMetadataRegistration (la única estructura que apunta a la tabla de tipos).

PRIMITIVOS = {0x01: "void", 0x02: "bool", 0x03: "char", 0x04: "sbyte", 0x05: "byte", 0x06: "short",
              0x07: "ushort", 0x08: "int", 0x09: "uint", 0x0A: "long", 0x0B: "ulong", 0x0C: "float",
              0x0D: "double", 0x0E: "string", 0x16: "TypedReference", 0x18: "IntPtr", 0x19: "UIntPtr",
              0x1C: "object"}
JUEGO = ("Assembly-CSharp.dll", "Assembly-UnityScript.dll")


class Memoria:
    def __init__(self, ruta):
        with open(ruta, "rb") as f:
            self.datos = f.read()

    def valida(self, a, n=4):
        return BASE <= a and a - BASE + n <= len(self.datos)

    def u32(self, a):
        return struct.unpack_from("<I", self.datos, a - BASE)[0]

    def u16(self, a):
        return struct.unpack_from("<H", self.datos, a - BASE)[0]

    def u8(self, a):
        return self.datos[a - BASE]

    def cadena(self, a, maximo=400):
        if not self.valida(a, 1):
            return None
        o = a - BASE
        fin = self.datos.find(b"\0", o, o + maximo)
        if fin == -1:
            return None
        crudo = self.datos[o:fin]
        if not all(32 <= c < 127 or c >= 0x80 for c in crudo):
            return None
        return crudo.decode("utf-8", "replace")

    def lista(self, a):
        """Arreglo de punteros terminado en nulo."""
        salida = []
        while a and self.valida(a) and self.u32(a):
            salida.append(self.u32(a))
            a += 4
        return salida


class Metadatos:
    def __init__(self, mem):
        self.mem = mem
        m = mem.u32
        self.tipos_tabla = [m(m(REGISTRO + 0x2C) + 4 * i) for i in range(m(REGISTRO + 0x28))]
        self.referencias = [m(m(REGISTRO + 0x34) + 4 * i) for i in range(m(REGISTRO + 0x30))]
        self.genericos = [(m(p), m(p + 4), m(p + 8)) for p in
                          (m(REGISTRO + 0x24) + 12 * i for i in range(m(REGISTRO + 0x20)))]
        self.imagenes = {}
        self.clases = {}  # TypeInfo* -> datos
        self.metodos = {}  # MethodInfo* -> datos
        for i in range(m(REGISTRO)):
            asamblea = m(m(REGISTRO + 4) + 4 * i)
            imagen = asamblea - 0x40
            self.leer_imagen(imagen)

    def leer_imagen(self, imagen):
        m, mem = self.mem.u32, self.mem
        nombre = mem.cadena(m(imagen))
        tipos = [m(m(imagen + 8) + 4 * i) for i in range(m(imagen + 0x0C))]
        nombres_campo = [m(m(imagen + 0x14) + 4 * i) for i in range(m(imagen + 0x18))]
        defs = [struct.unpack_from("<4I", mem.datos, m(imagen + 0x1C) + 16 * i - BASE) for i in range(m(imagen + 0x20))]
        self.imagenes[nombre] = tipos
        siguiente = 0
        for t in tipos:
            n_campos = m(t + 0x88)
            campos = []
            for d in defs[siguiente:siguiente + n_campos]:
                tipo = self.tipos_tabla[d[1]] if d[1] < len(self.tipos_tabla) else 0
                campos.append({"nombre": mem.cadena(nombres_campo[d[0]]), "tipo": tipo, "offset": d[2],
                               "estatico": bool(self.atributos(tipo) & 0x10)})
            siguiente += n_campos
            metodos = mem.lista(m(t + 0x10))
            self.clases[t] = {"imagen": nombre, "nombre": mem.cadena(m(t + 8)), "ns": mem.cadena(m(t + 0x0C)) or "",
                              "tamano": m(t + 0x58), "flags": m(t + 0x74), "bits": m(t + 0x8C),
                              "campos": campos, "metodos": metodos}
            for mi in metodos:
                self.metodos[mi] = self.leer_metodo(mi, t)

    def leer_metodo(self, mi, clase):
        m, mem = self.mem.u32, self.mem
        n_param = mem.u8(mi + 0x22)
        parametros = []
        p = m(mi + 0x14)
        for k in range(n_param):
            parametros.append((mem.cadena(m(p + 20 * k)), m(p + 20 * k + 16)))
        return {"nombre": mem.cadena(m(mi)), "puntero": m(mi + 4), "clase": clase, "retorno": m(mi + 0x0C),
                "parametros": parametros, "flags": mem.u16(mi + 0x1C), "estatico": bool(mem.u16(mi + 0x1C) & 0x10)}

    # --- tipos -------------------------------------------------------------------------------

    def bits_tipo(self, t):
        return self.mem.u32(t + 4) if t and self.mem.valida(t, 8) else 0

    def atributos(self, t):
        return self.bits_tipo(t) & 0xFFFF

    def codigo(self, t):
        return (self.bits_tipo(t) >> 16) & 0xFF

    def nombre_clase(self, c, corto=True):
        if c in self.clases:
            d = self.clases[c]
            return d["nombre"] if corto or not d["ns"] else d["ns"] + "." + d["nombre"]
        n = self.mem.cadena(self.mem.u32(c + 8)) if self.mem.valida(c + 8) else None
        return n or f"<clase {c:#x}>"

    def nombre_tipo(self, t, corto=True):
        mem = self.mem
        if not t or not mem.valida(t, 8):
            return "?"
        datos, codigo = mem.u32(t), self.codigo(t)
        sufijo = "&" if self.bits_tipo(t) & 0x40000000 else ""
        if codigo in (0x11, 0x12):
            return self.nombre_clase(datos, corto) + sufijo
        if codigo in PRIMITIVOS:
            return PRIMITIVOS[codigo] + sufijo
        if codigo == 0x1D:
            return self.nombre_tipo(datos, corto) + "[]" + sufijo
        if codigo == 0x0F:
            return self.nombre_tipo(datos, corto) + "*"
        if codigo == 0x15:
            # Il2CppGenericClass: la definición y el contexto (class_inst).
            definicion = self.nombre_clase(mem.u32(datos), corto) if mem.valida(datos) else "?"
            argumentos = self.argumentos(mem.u32(datos + 4)) if mem.valida(datos + 4) else "?"
            return f"{definicion.split('`')[0]}<{argumentos}>{sufijo}"
        if codigo in (0x13, 0x1E):
            n = mem.cadena(mem.u32(datos + 4)) if mem.valida(datos + 4) else None
            return (n or ("T" if codigo == 0x13 else "M")) + sufijo
        return f"<tipo {codigo:#x}>"

    def argumentos(self, inst):
        """Il2CppGenericInst: cantidad y arreglo de Il2CppType*."""
        if not inst or not self.mem.valida(inst, 8):
            return "?"
        n, argv = self.mem.u32(inst), self.mem.u32(inst + 4)
        if n > 16 or not self.mem.valida(argv, 4 * n):
            return "?"
        return ", ".join(self.nombre_tipo(self.mem.u32(argv + 4 * i)) for i in range(n))

    def es_enum(self, t):
        """Los enum (y los primitivos) traen el bit 20 en los bits del TypeInfo; los struct, no."""
        c = self.mem.u32(t)
        return c in self.clases and bool(self.clases[c]["bits"] & 0x100000)

    def letra(self, t, retorno=False):
        """La letra de un tipo en la firma de Emscripten."""
        codigo = self.codigo(t)
        if self.bits_tipo(t) & 0x40000000:
            return "i"
        if codigo == 0x01:
            return "v"
        if codigo == 0x0C:
            return "f"
        if codigo == 0x0D:
            return "d"
        if codigo in (0x0A, 0x0B):
            return "i" if retorno else "ii"
        return "i"

    def firmas(self, info):
        """Las firmas posibles del método (con o sin retorno por puntero si devuelve un struct)."""
        params = "i" + "".join(self.letra(t) for _, t in info["parametros"]) + "i"
        r = info["retorno"]
        if self.codigo(r) == 0x11 and not self.es_enum(r) and not self.bits_tipo(r) & 0x40000000:
            # Un struct se devuelve por un puntero que va primero.
            return ["vi" + params, "i" + params]
        candidatas = [self.letra(r, True) + params]
        if self.codigo(r) == 0x15:
            candidatas.append("vi" + params)
        return candidatas

    def firma(self, info):
        retorno = self.nombre_tipo(info["retorno"])
        params = ", ".join(f"{self.nombre_tipo(t)} {n}" for n, t in info["parametros"])
        estatico = "static " if info["estatico"] else ""
        return f"{estatico}{retorno} {self.nombre_clase(info['clase'])}::{info['nombre']}({params})"

    def generico(self, gm):
        """Il2CppGenericMethod: el método definido y su contexto (class_inst, method_inst)."""
        mem = self.mem
        if not mem.valida(gm, 12):
            return None
        definicion = mem.u32(gm)
        info = self.metodos.get(definicion)
        if not info:
            return None
        clase = self.nombre_clase(info["clase"])
        ci, mi = mem.u32(gm + 4), mem.u32(gm + 8)
        if ci:
            clase = f"{clase.split('`')[0]}<{self.argumentos(ci)}>"
        nombre = info["nombre"] + (f"<{self.argumentos(mi)}>" if mi else "")
        return f"{clase}::{nombre}"


class AsmJs:
    def __init__(self, ruta):
        with open(ruta, encoding="latin-1") as f:
            self.js = f.read()
        self.ini = self.js.find("// EMSCRIPTEN_START_FUNCS")
        self.fin = self.js.find("// EMSCRIPTEN_END_FUNCS")
        cola = self.js[self.fin:self.js.find("// EMSCRIPTEN_END_ASM")]
        self.tablas = {m.group(1): m.group(2).split(",") for m in re.finditer(r"var ([\w$]+)=\[([^\]]*)\]", cola)}
        r = cola.find("return{")
        exportes = dict(x.split(":") for x in cola[r + 7:cola.find("}", r)].split(","))
        self.funciones = {m.group(1): self.ini + m.start()
                          for m in re.finditer(r"function ([\w$]+)\(([^)]*)\)\{", self.js[self.ini:self.fin])}
        self.orden = sorted(self.funciones.values())
        # Las llamadas dentro de un try pasan por env.invoke_<firma>(índice, argumentos...).
        cabecera = self.js[self.js.find('"use asm"'):self.ini]
        self.invocadores = {m.group(1): m.group(2) for m in re.finditer(r"var ([\w$]+)=env\.invoke_(\w+);", cabecera)}
        self.firmas = {}
        for k, v in exportes.items():
            if k.startswith("dynCall_"):
                p = self.funciones[v]
                m = re.search(r"([\w$]+)\[[\w$]+&(\d+)\]\(", self.js[p:p + 600])
                self.firmas[k[8:]] = m.group(1)

    def funcion(self, firma, indice):
        tabla = self.tablas.get(self.firmas.get(firma))
        if not tabla:
            return None
        nombre = tabla[indice & (len(tabla) - 1)]
        # La primera entrada de cada tabla es el talón que aborta; los huecos lo repiten.
        return None if nombre == tabla[0] else nombre

    def cuerpo(self, nombre):
        p = self.funciones[nombre]
        i = bisect.bisect_right(self.orden, p)
        q = self.orden[i] if i < len(self.orden) else self.fin
        return self.js[p:q]


def separar(cuerpo):
    """Una sentencia por línea, con sangría por llaves (el asm.js minificado viene en una línea)."""
    salida, nivel, linea = [], 0, ""
    i = 0
    while i < len(cuerpo):
        c = cuerpo[i]
        if c == "/" and cuerpo.startswith("/*", i):
            fin = cuerpo.find("*/", i) + 2
            linea += cuerpo[i:fin]
            i = fin
            continue
        linea += c
        if c == "{":
            salida.append("  " * nivel + linea.strip())
            nivel += 1
            linea = ""
        elif c == "}":
            if linea[:-1].strip():
                salida.append("  " * nivel + linea[:-1].strip())
            nivel = max(0, nivel - 1)
            salida.append("  " * nivel + "}")
            linea = ""
        elif c == ";":
            salida.append("  " * nivel + linea.strip())
            linea = ""
        i += 1
    if linea.strip():
        salida.append(linea.strip())
    return "\n".join(salida)


class Volcado:
    def __init__(self, release):
        self.md = Metadatos(Memoria(os.path.join(release, "Neuro Battle.html.mem")))
        self.asm = AsmJs(os.path.join(release, "Neuro Battle.js"))
        self.llamadas = set()  # funciones que llaman los scripts del juego
        self.nombre_de = {}  # función del asm.js -> nombres de métodos
        self.funcion_de = {}  # MethodInfo* -> función
        for mi, info in self.md.metodos.items():
            for firma in self.md.firmas(info):
                f = self.asm.funcion(firma, info["puntero"])
                if f:
                    self.funcion_de[mi] = f
                    self.nombre_de.setdefault(f, []).append(
                        f"{self.md.nombre_clase(info['clase'])}::{info['nombre']}")
                    break
        # Los métodos genéricos: (Il2CppGenericMethod*, índice de la función, índice del invocador).
        # Su firma se deduce de la definición, así que solo nombran funciones sin otro nombre.
        genericos = {}
        for gm, puntero, _ in self.md.genericos:
            nombre = self.md.generico(gm)
            if not nombre:
                continue
            for firma in self.firmas_genericas(gm):
                f = self.asm.funcion(firma, puntero)
                if f:
                    genericos.setdefault(f, []).append(nombre)
                    break
        for f, nombres in genericos.items():
            self.nombre_de.setdefault(f, nombres)

    def firmas_genericas(self, gm):
        info = self.md.metodos.get(self.md.mem.u32(gm))
        if not info:
            return []
        # Los parámetros genéricos (T) son referencias en estos usos: GetComponent<T> y similares.
        return self.md.firmas(info)

    def literal(self, a):
        """Un objeto String estático (clase String, monitor, largo, UTF-16)."""
        mem = self.md.mem
        if not mem.valida(a, 12) or a % 4:
            return None
        clase, monitor, largo = struct.unpack_from("<3I", mem.datos, a - BASE)
        if monitor != 0 or not 0 < largo < 400 or self.md.nombre_clase(clase) != "String":
            return None
        if not mem.valida(a + 12, 2 * largo):
            return None
        return mem.datos[a + 12 - BASE:a + 12 - BASE + 2 * largo].decode("utf-16-le", "replace")

    def anotar_numero(self, n):
        md = self.md
        if n < 4096 or not md.mem.valida(n):
            return None
        s = self.literal(n)
        if s is not None:
            return '"' + s.replace("*/", "*\\/") + '"'
        if n in md.metodos:
            return "MethodInfo " + md.firma(md.metodos[n])
        if n in md.clases:
            return "TypeInfo " + md.nombre_clase(n, False)
        return None

    def anotar(self, cuerpo):
        def llamada(m):
            self.llamadas.add(m.group(1))
            nombres = self.nombre_de.get(m.group(1))
            return m.group(0) + (f"/*{' | '.join(nombres[:3])}*/" if nombres else "")
        texto = re.sub(r"(?<![\w$.])(?<!function )([\w$]+)\(", llamada, cuerpo)

        def invocacion(m):
            firma = self.asm.invocadores[m.group(1)]
            f = self.asm.funcion(firma, int(m.group(2)))
            if f:
                self.llamadas.add(f)
            nombres = self.nombre_de.get(f, [f]) if f else ["?"]
            return m.group(0) + f"/*invoke {' | '.join(nombres[:3])}*/"
        claves = "|".join(re.escape(k) for k in self.asm.invocadores)
        texto = re.sub(rf"(?<![\w$.])({claves})\((\d+)", invocacion, texto)

        def numero(m):
            # El minificador escribe algunas direcciones en notación exponencial (296e4).
            nota = self.anotar_numero(int(float(m.group(0))))
            return m.group(0) + (f"/*{nota}*/" if nota else "")
        texto = re.sub(r"(?<![\w$.])(?:\d{4,}|\d+e\d+)(?![\w.])", numero, texto)

        # Las referencias a tipos (DO) y a métodos (FO) que el código resuelve al primer uso.
        def tipo(m):
            i = int(m.group(2))
            t = self.md.tipos_tabla[i] if 0 <= i < len(self.md.tipos_tabla) else 0
            return m.group(0) + f"/*typeof({self.md.nombre_tipo(t, False)})*/"
        texto = re.sub(r"(?<![\w$.])(DO)\((-?\d+)\)", tipo, texto)

        def metodo(m):
            token = int(m.group(1)) & 0xFFFFFFFF
            i = token & 0x7FFFFFFF
            ref = self.md.referencias[i] if i < len(self.md.referencias) else 0
            if token & 0x80000000:
                nota = self.md.generico(ref) or "?"
            else:
                nota = self.md.firma(self.md.metodos[ref]) if ref in self.md.metodos else "?"
            return m.group(0) + f"/*{nota}*/"
        texto = re.sub(r"(?<![\w$.])FO\((-?\d+)\)", metodo, texto)
        return texto

    def escribir(self, salida):
        md = self.md
        os.makedirs(os.path.join(salida, "asmjs"), exist_ok=True)
        lineas = []
        for imagen in JUEGO:
            lineas.append(f"// {imagen}")
            for t in md.imagenes[imagen]:
                d = md.clases[t]
                if d["nombre"] == "<Module>":
                    continue
                lineas.append(f"class {d['ns'] + '.' if d['ns'] else ''}{d['nombre']}  // TypeInfo {t:#x}, "
                              f"{d['tamano']} bytes, flags {d['flags']:#x}")
                for c in d["campos"]:
                    lugar = "        " if c["estatico"] else f"+{c['offset']:<3}    "
                    acceso = {1: "private ", 6: "public "}.get(md.atributos(c["tipo"]) & 7, "")
                    estatico = "static " if c["estatico"] else ""
                    lineas.append(f"    {lugar}{acceso}{estatico}{md.nombre_tipo(c['tipo'])} {c['nombre']}")
                for mi in d["metodos"]:
                    lineas.append(f"    {md.firma(md.metodos[mi])}  -> {self.funcion_de.get(mi, '?')}")
                lineas.append("")
                self.escribir_clase(salida, t)
        with open(os.path.join(salida, "metadatos.txt"), "w", encoding="utf-8") as out:
            out.write("\n".join(lineas))
        # Los métodos del motor (UnityEngine, mscorlib) que llaman los scripts del juego.
        motor = set()
        for mi, f in self.funcion_de.items():
            if f in self.llamadas and md.clases[md.metodos[mi]["clase"]]["imagen"] not in JUEGO:
                motor.add(f"{f:6} {md.firma(md.metodos[mi])}")
        motor = sorted(motor, key=lambda x: x[7:])
        with open(os.path.join(salida, "motor.txt"), "w", encoding="utf-8") as out:
            out.write("// Los métodos del motor que llaman los scripts: función del asm.js y firma.\n")
            out.write("\n".join(motor) + "\n")
        sin = sum(1 for mi in md.metodos if mi not in self.funcion_de)
        print(f"tipos: {len(md.clases)}, métodos: {len(md.metodos)} ({sin} sin función), "
              f"métodos genéricos: {len(md.genericos)}")

    def escribir_clase(self, salida, t):
        md = self.md
        d = md.clases[t]
        partes = [f"// {d['nombre']} ({d['imagen']}): el asm.js de cada método, con las llamadas, los tipos,"
                  " los métodos y los textos anotados.",
                  "// Memoria: a = HEAP8, b = HEAP16, c = HEAP32, d = HEAPU8, g = HEAPF32, h = HEAPF64;"
                  " fa = Math.fround.",
                  "// Campos: " + ", ".join(f"{c['nombre']} (+{c['offset']})" if not c["estatico"]
                                           else f"static {c['nombre']}" for c in d["campos"]), ""]
        for mi in d["metodos"]:
            f = self.funcion_de.get(mi)
            partes.append(f"// {md.firma(md.metodos[mi])}")
            partes.append(separar(self.anotar(self.asm.cuerpo(f))) if f else "// (sin función)")
            partes.append("")
        with open(os.path.join(salida, "asmjs", d["nombre"] + ".js"), "w", encoding="utf-8") as out:
            out.write("\n".join(partes))


if __name__ == "__main__":
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    Volcado(sys.argv[1]).escribir(sys.argv[2])
