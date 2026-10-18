"""Lector de exportaciones HTML5 de Construct 2 (data.js + c2runtime.js).

Uso:
    python3 -I tools/c2_eventos.py resumen <carpeta html5> [--nombres <nombres.json>]
    python3 -I tools/c2_eventos.py eventos <carpeta html5> [--nombres <nombres.json>]
    python3 -I tools/c2_eventos.py fuente  <carpeta html5> <referencia>    (p. ej. Y.prototype.j.Tq)

`resumen` lista el proyecto, los plugins y comportamientos, los tipos de objeto (con sus imágenes),
los layouts con sus capas e instancias, y la tabla de referencias del runtime. `eventos` vuelca las
hojas de eventos a pseudocódigo. `fuente` imprime el código (minificado) de una condición, acción o
expresión, para identificarla.

Construct 2 exporta con el runtime minificado por Closure Compiler: los nombres de los tipos de
objeto pasan a ser t0, t1... y los plugins y sus condiciones/acciones/expresiones quedan como
referencias minificadas (`kc`, `kc.prototype.ca.Mr`). La herramienta adivina el plugin de cada
constructor por las cadenas de su código, y el resto se anota a mano en un JSON por juego:

    {"plugins": {"kc": "Sprite"}, "tipos": {"t0": "Pelota"},
     "aces": {"kc.prototype.ca.Mr": "SetX", "Y.prototype.j.Tq": "CompareVar"}}

La estructura de data.js sigue al cargador del runtime de Construct 2 (loadProject, EventSheet,
EventBlock, Condition, Action, Parameter, ExpNode). Los archivos son datos no confiables: solo se
leen como texto.
"""
import json
import os
import re
import sys

# Huellas de los plugins oficiales: cadenas que aparecen en el código de su runtime.
HUELLAS = [
    ("Audio", {".ogg", "canplaythrough"}),
    ("Touch", {"devicemotion", "touchstart"}),
    ("Browser", {"backbutton", "online"}),
    ("Button", {"checkbox", "button"}),
    ("TextBox", {"textarea"}),
    ("Mouse", {"mousewheel"}),
    ("Keyboard", {"keydown"}),
    ("Text", {"pt ", "top"}),
    ("WebStorage", {"localStorage"}),
    ("Ajax", {"XMLHttpRequest"}),
    ("Particles", {"spray"}),
]

COMPARACIONES = ["=", "≠", "<", "≤", ">", "≥"]
OPERADORES = {4: "+", 5: "-", 6: "*", 7: "/", 8: "%", 9: "^", 10: "&", 11: "|", 12: "=", 13: "<>",
              14: "<", 15: "<=", 16: ">", 17: ">="}


class Proyecto:
    def __init__(self, carpeta, nombres=None):
        with open(os.path.join(carpeta, "data.js"), encoding="utf-8-sig") as f:
            self.p = json.load(f)["project"]
        with open(os.path.join(carpeta, "c2runtime.js"), encoding="utf-8-sig") as f:
            self.runtime = f.read()
        self.nombres = {"plugins": {}, "tipos": {}, "aces": {}}
        if nombres:
            with open(nombres, encoding="utf-8") as f:
                for k, v in json.load(f).items():
                    self.nombres.setdefault(k, {}).update(v)
        self.refs = self.tabla_de_referencias()
        self.tipos = self.p[3]
        self.clases = self.clasificar_aces()
        self.etiquetas_ctor = self.etiquetar_constructores()

    # --- runtime ---------------------------------------------------------------------------

    def tabla_de_referencias(self):
        """cr.getObjectRefTable(): en el runtime minificado, la última función `return[...]`."""
        r = self.runtime
        m = list(re.finditer(r"getObjectRefTable\s*=\s*function\s*\(\)\s*\{\s*return\s*\[", r)) or \
            list(re.finditer(r"function [\w$]+\(\)\{return\[", r))
        if not m:
            raise ValueError("no encuentro la tabla de referencias en c2runtime.js")
        inicio = m[-1].end()
        fin = r.index("]", inicio)
        return [x.strip() for x in r[inicio:fin].split(",")]

    def region(self, ctor):
        """Código de un plugin o comportamiento: desde `function ctor(` hasta el siguiente
        constructor de la tabla."""
        r = self.runtime
        posiciones = {}
        for nombre in {x for x in self.refs if ".prototype." not in x}:
            m = re.search(r"function " + re.escape(nombre) + r"\(", r) or \
                re.search(r"(?<![\w$.])" + re.escape(nombre) + r"\s*=\s*function\(", r)
            if m:
                posiciones[nombre] = m.start()
        if ctor not in posiciones:
            return ""
        inicio = posiciones[ctor]
        siguientes = [p for p in posiciones.values() if p > inicio]
        return r[inicio:min(siguientes) if siguientes else len(r)]

    def literales(self, codigo):
        cadenas = re.findall(r'"(?:[^"\\\n]|\\.)*"|\'(?:[^\'\\\n]|\\.)*\'', codigo)
        return {c[1:-1] for c in cadenas}

    def clasificar_aces(self):
        """Qué propiedad minificada son cnds, acts y exps, según dónde se usan en los eventos."""
        usos = {}

        def marcar(indice, clase):
            ref = self.refs[indice] if 0 <= indice < len(self.refs) else None
            if ref and ".prototype." in ref:
                usos.setdefault(ref.split(".")[2], set()).add(clase)

        def exp(m):
            if not isinstance(m, list) or not m:
                return
            t = m[0]
            if t == 19:
                marcar(m[1], "exp")
                for x in (m[2] if len(m) == 3 else []):
                    exp(x)
            elif t == 20:
                marcar(m[2], "exp")
                for x in (m[5] if len(m) == 6 else []):
                    exp(x)
            elif t == 22:
                marcar(m[3], "exp")
                for x in (m[6] if len(m) == 7 else []):
                    exp(x)
            else:
                for x in m[1:]:
                    exp(x)

        def bloque(ev):
            if ev[0] != 0:
                return
            for c in ev[5]:
                marcar(c[1], "cnd")
                for prm in (c[9] if len(c) == 10 else []):
                    exp(prm[1] if len(prm) > 1 else None)
            for a in ev[6]:
                marcar(a[1], "act")
                for prm in (a[5] if len(a) == 6 else []):
                    exp(prm[1] if len(prm) > 1 else None)
            for sub in (ev[7] if len(ev) == 8 else []):
                bloque(sub)

        for hoja in self.p[6]:
            for ev in hoja[1]:
                bloque(ev)
        return {k: "/".join(sorted(v)) for k, v in usos.items()}

    def etiquetar_constructores(self):
        etiquetas = {}
        con_animaciones = {t[1] for t in self.tipos if t[7]}
        comportamientos = {}
        for t in self.tipos:
            for b in t[8] or []:
                comportamientos.setdefault(b[1], set()).add(b[0])
        sistema = {r.split(".")[0] for r in self.refs if ".prototype." in r} - \
                  {r for r in self.refs if ".prototype." not in r}
        for i, ref in enumerate(self.refs):
            if ".prototype." in ref:
                continue
            if ref in self.nombres["plugins"]:
                etiquetas[ref] = self.nombres["plugins"][ref]
            elif i in comportamientos:
                etiquetas[ref] = "comportamiento " + "/".join(sorted(comportamientos[i]))
            elif i in con_animaciones:
                etiquetas[ref] = "Sprite"
            else:
                lits = self.literales(self.region(ref))
                etiquetas[ref] = next((n for n, h in HUELLAS if h <= lits), "?")
        for ref in sistema:
            etiquetas.setdefault(ref, self.nombres["plugins"].get(ref, "System"))
        return etiquetas

    # --- nombres legibles ------------------------------------------------------------------

    def tipo(self, indice):
        if indice == -1:
            return "System"
        t = self.tipos[indice]
        nombre = self.nombres["tipos"].get(t[0])
        if nombre:
            return nombre
        imagen = None
        if t[7]:
            imagen = t[7][0][7][0][0] if t[7][0][7] else None
        elif t[6]:
            imagen = t[6][0]
        etiqueta = os.path.basename(imagen).split("-sheet")[0].rsplit(".", 1)[0] if imagen else \
            self.etiquetas_ctor.get(self.refs[t[1]], "?")
        return f"{t[0]}<{etiqueta}>"

    def ace(self, indice):
        ref = self.refs[indice]
        if ref in self.nombres["aces"]:
            return self.nombres["aces"][ref]
        partes = ref.split(".")
        if len(partes) == 4:
            return f"{self.clases.get(partes[2], '?')}:{partes[3]}"
        return ref

    def exp(self, m):
        if m is None:
            return "∅"
        t = m[0]
        if t in (0, 1):
            return repr(m[1]) if t == 1 else str(m[1])
        if t == 2:
            return json.dumps(m[1], ensure_ascii=False)
        if t == 3:
            return f"-({self.exp(m[1])})"
        if t in OPERADORES:
            return f"({self.exp(m[1])} {OPERADORES[t]} {self.exp(m[2])})"
        if t == 18:
            return f"({self.exp(m[1])} ? {self.exp(m[2])} : {self.exp(m[3])})"
        if t == 19:
            args = ", ".join(self.exp(x) for x in (m[2] if len(m) == 3 else []))
            return f"{self.ace(m[1])}({args})"
        if t == 20:
            inst = f"({self.exp(m[4])})" if m[4] else ""
            args = ", ".join(self.exp(x) for x in (m[5] if len(m) == 6 else []))
            return f"{self.tipo(m[1])}{inst}.{self.ace(m[2])}({args})"
        if t == 21:
            inst = f"({self.exp(m[3])})" if m[3] else ""
            return f"{self.tipo(m[1])}{inst}.var{m[4]}"
        if t == 22:
            inst = f"({self.exp(m[5])})" if m[5] else ""
            args = ", ".join(self.exp(x) for x in (m[6] if len(m) == 7 else []))
            return f"{self.tipo(m[1])}{inst}.{m[2]}.{self.ace(m[3])}({args})"
        if t == 23:
            return m[1]
        return f"<exp {t} {m}>"

    def parametro(self, m):
        t = m[0]
        if t in (0, 1, 5, 7):
            return self.exp(m[1])
        if t == 8:
            return COMPARACIONES[m[1]] if 0 <= m[1] < len(COMPARACIONES) else f"cmp{m[1]}"
        if t == 3:
            return f"opción{m[1]}"
        if t == 4:
            return self.tipo(m[1])
        if t == 6:
            return f"layout {json.dumps(m[1], ensure_ascii=False)}"
        if t == 9:
            return f"tecla {m[1]}"
        if t == 10:
            return f"var{m[1]}"
        if t == 11:
            return m[1]
        if t == 2:
            return f"sonido {m[1][0]!r}{' (música)' if m[1][1] else ''}"
        if t == 12:
            return f"archivo {m[1]!r}"
        if t == 13:
            return "[" + ", ".join(self.parametro(x) for x in m[1:]) + "]"
        return f"<param {t} {m[1:]}>"

    # --- salidas ---------------------------------------------------------------------------

    def resumen(self):
        p = self.p
        print(f"Proyecto {p[26] if len(p) > 26 else p[0]!r}: {p[10]}x{p[11]}, primer layout {p[1]!r}, "
              f"modo de pantalla {p[12]}, medios en {p[8]!r}, versión {p[16] if len(p) > 16 else '?'}")
        print("\nPlugins:")
        for pl in p[2]:
            ref = self.refs[pl[0]]
            flags = "".join(c for c, v in zip("GWPSAZE", [pl[1], pl[2], pl[3], pl[4], pl[6], pl[7], pl[8]]) if v)
            print(f"  [{pl[0]:2}] {ref:6} {self.etiquetas_ctor.get(ref, '?'):24} {flags}")
        comportamientos = [i for i, r in enumerate(self.refs) if ".prototype." not in r and
                           self.etiquetas_ctor.get(r, "").startswith("comportamiento")]
        for i in comportamientos:
            print(f"  [{i:2}] {self.refs[i]:6} {self.etiquetas_ctor[self.refs[i]]}")
        print("  (G global, W en el mundo, P posición, S tamaño, A apariencia, Z orden, E efectos)")

        print("\nTipos de objeto:")
        for i, t in enumerate(self.tipos):
            plugin = self.etiquetas_ctor.get(self.refs[t[1]], "?")
            linea = f"  {i:2} {self.tipo(i):28} {plugin:12}"
            if t[2]:
                linea += " familia"
            if t[3]:
                linea += f" vars={len(t[3])}"
            if t[8]:
                linea += " comportamientos=" + ",".join(b[0] for b in t[8])
            print(linea)
            for anim in t[7] or []:
                cuadros = anim[7]
                print(f"       animación {anim[0]!r} velocidad={anim[1]} bucle={anim[2]} "
                      f"cuadros={len(cuadros)}: " + ", ".join(sorted({os.path.basename(c[0]) for c in cuadros})))
                for c in cuadros:
                    print(f"         {os.path.basename(c[0])} recorte=({c[2]},{c[3]},{c[4]},{c[5]}) "
                          f"duración={c[6]} origen=({c[7]:.3f},{c[8]:.3f}) puntos={c[9]} "
                          f"colisión={[round(v, 3) for v in c[10]] if c[10] else []}")
            if t[6]:
                print(f"       textura {t[6][0]}")

        print("\nLayouts:")
        for lay in p[5]:
            print(f"  {lay[0]!r} {lay[1]}x{lay[2]} hoja={lay[4]!r}")
            for capa in lay[6]:
                print(f"    capa {capa[0]!r} visible={capa[3]} fondo={capa[4]} transparente={capa[5]} "
                      f"parallax=({capa[6]},{capa[7]}) opacidad={capa[8]}")
                for inst in capa[14]:
                    w = inst[0]
                    print(f"      {self.tipo(inst[1]):28} uid={inst[2]} x={w[0]:.6g} y={w[1]:.6g} "
                          f"w={w[3]:.6g} h={w[4]:.6g} ángulo={w[6]:.6g} opacidad={w[7]:.6g} "
                          f"vars={inst[3]} comp={inst[4]} props={inst[5] if len(inst) > 5 else None}")
            for inst in lay[7]:
                print(f"    (sin mundo) {self.tipo(inst[1])} uid={inst[2]} props={inst[5] if len(inst) > 5 else None}")

        print("\nMedios:", ", ".join(f"{m[0]} ({m[1]} B)" for m in p[7]))
        print("\nTabla de referencias:")
        for i, ref in enumerate(self.refs):
            if ".prototype." in ref:
                partes = ref.split(".")
                dueño = self.etiquetas_ctor.get(partes[0], "?")
                print(f"  {i:3} {ref:26} {dueño:20} {self.clases.get(partes[2], '?'):4} {self.nombres['aces'].get(ref, '')}")
            else:
                print(f"  {i:3} {ref:26} {self.etiquetas_ctor.get(ref, '?')}")

    def eventos(self):
        for hoja in self.p[6]:
            print(f"\n== Hoja de eventos {hoja[0]!r}")
            for ev in hoja[1]:
                self.evento(ev, 0)

    def evento(self, ev, nivel):
        sangria = "  " * nivel
        if ev[0] == 1:
            tipo = "texto" if ev[2] == 1 else "número"
            extra = " constante" if ev[5] else ""
            extra += " estática" if ev[4] else ""
            print(f"{sangria}variable {tipo}{extra} {ev[1]} = {json.dumps(ev[3], ensure_ascii=False)}")
            return
        if ev[0] == 2:
            print(f"{sangria}incluir hoja {ev[1]!r}")
            return
        if ev[1]:
            print(f"{sangria}grupo {ev[1][1]!r} {'activo' if ev[1][0] else 'inactivo'}")
        cabecera = f"{sangria}evento" + (" (O)" if ev[2] else "")
        print(cabecera)
        for c in ev[5]:
            negado = "no " if c[5] else ""
            disparador = " [disparador]" if c[3] else ""
            bucle = " [bucle]" if c[4] else ""
            comp = f".{c[2]}" if c[2] else ""
            args = ", ".join(self.parametro(x) for x in (c[9] if len(c) == 10 else []))
            print(f"{sangria}  si {negado}{self.tipo(c[0])}{comp}.{self.ace(c[1])}({args}){disparador}{bucle}")
        for a in ev[6]:
            comp = f".{a[2]}" if a[2] else ""
            args = ", ".join(self.parametro(x) for x in (a[5] if len(a) == 6 else []))
            print(f"{sangria}  -> {self.tipo(a[0])}{comp}.{self.ace(a[1])}({args})")
        for sub in (ev[7] if len(ev) == 8 else []):
            self.evento(sub, nivel + 1)

    def fuente(self, ref):
        partes = ref.split(".")
        nombre = partes[-1]
        region = self.region(partes[0])
        encontrado = False
        for codigo, origen in ((region, f"región de {partes[0]}"), (self.runtime, "runtime")):
            for m in re.finditer(r"[.\s,;{(]" + re.escape(nombre) + r"\s*=\s*function\s*\(", codigo):
                print(f"-- {origen}, desplazamiento {m.start()}")
                print(cuerpo(codigo, m.start() + 1))
                encontrado = True
            if encontrado:
                return
        print("no encontrado")


def cuerpo(codigo, inicio):
    """Texto desde `inicio` hasta la llave que cierra la primera función."""
    i = codigo.index("{", inicio)
    nivel = 0
    for j in range(i, len(codigo)):
        if codigo[j] == "{":
            nivel += 1
        elif codigo[j] == "}":
            nivel -= 1
            if nivel == 0:
                return codigo[inicio:j + 1]
    return codigo[inicio:inicio + 2000]


if __name__ == "__main__":
    args = sys.argv[1:]
    nombres = None
    if "--nombres" in args:
        i = args.index("--nombres")
        nombres = args[i + 1]
        del args[i:i + 2]
    if len(args) >= 2 and args[0] in ("resumen", "eventos"):
        proyecto = Proyecto(args[1], nombres)
        getattr(proyecto, args[0])()
    elif len(args) == 3 and args[0] == "fuente":
        Proyecto(args[1], nombres).fuente(args[2])
    else:
        sys.exit(__doc__)
