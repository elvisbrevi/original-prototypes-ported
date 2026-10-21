"""Regenera assets/ desde el WebGL de Neuro Battle (Unity 5.1.1f1, IL2CPP).

Uso (con el Python del venv de UnityPy, ver PLAN.md):
    ~/.venvs/museo/bin/python -I games/neuro-battle/tools/extraer.py <carpeta Release> <carpeta data> <carpeta Managed>

<carpeta Release> es la del WebGL (Neuro Battle.js y .html.mem), <carpeta data> el .data
desempaquetado con `tools/unity_assets.py emscripten`, y <carpeta Managed> una con el
UnityEngine.dll y el mscorlib.dll de Unity 5.1 (el WebGL no los trae; sirve la del build de Linux
de One Button Prototype, Unity 5.1.2).

Escribe en games/neuro-battle/assets/:
    texturas/<nombre>.png   las texturas de los sprites que se ven (recortadas al rectángulo que
                            ocupan sus sprites) y las tres pantallas de la GUI
    audio/<nombre>.ogg      los dos sonidos (AAC en el original, pasados a Ogg Vorbis con ffmpeg)
    escena.json             las cuatro escenas del build, los prefabs que instancian los scripts,
                            los sprites, los clips y controladores de Mecanim, el estilo del
                            puntaje, las fuentes y los ajustes del proyecto (tiempo, física 2D,
                            capas, ejes del InputManager)

El build no trae los ensamblados de C#: los campos de los MonoBehaviour se leen con type trees
que TypeTreeGeneratorAPI genera de unos ensamblados «stub» con solo los campos de cada script,
escritos a partir de los metadatos IL2CPP (tools/il2cpp.py) y compilados con el csc del SDK de
.NET contra el UnityEngine.dll de Unity 5.1. La fuente del puntaje («Light Pixel-7», de Style-7)
es gratis solo para uso personal y no se copia: el port usa Pixelify Sans (SIL OFL 1.1).

El original es un dato no confiable: solo se lee, nunca se ejecuta (ffmpeg solo decodifica el
audio).
"""
import glob
import json
import os
import shutil
import struct
import subprocess
import sys
import tempfile

import UnityPy
from UnityPy.files import SerializedFile
from UnityPy.helpers.TypeTreeGenerator import TypeTreeGenerator

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import il2cpp  # noqa: E402

RAIZ = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "assets")

# Las escenas en el orden de BuildSettings: titulo, instrucciones, level y game_over.
ESCENAS = ["mainData", "level0", "level1", "level2"]
ALIAS = {"library/unity default resources": "unity default resources",
         "resources/unity_builtin_extra": "unity_builtin_extra"}
ETIQUETAS = {0: "Untagged", 1: "Respawn", 2: "Finish", 3: "EditorOnly", 5: "MainCamera", 6: "Player",
             7: "GameController"}
# Los ejes del InputManager que leen los scripts.
EJES = ("Horizontal", "Vertical", "Rotacion", "Fire2")


def r6(x):
    return round(x, 6)


def v(d, n=3):
    return [r6(d[c]) for c in "xyzw"[:n]]


def color(c):
    return [r6(c[k]) for k in "rgba"]


def stubs(release, managed, carpeta):
    """Compila Assembly-CSharp.dll y Assembly-UnityScript.dll con los campos de cada script."""
    md = il2cpp.Metadatos(il2cpp.Memoria(os.path.join(release, "Neuro Battle.html.mem")))
    for n in ("mscorlib.dll", "UnityEngine.dll"):
        shutil.copy(os.path.join(managed, n), carpeta)
    csc = sorted(glob.glob("/usr/lib/dotnet/sdk/*/Roslyn/bincore/csc.dll")
                 + glob.glob(os.path.expanduser("~/.dotnet/sdk/*/Roslyn/bincore/csc.dll")))[-1]
    for imagen in il2cpp.JUEGO:
        lineas = []
        for t in md.imagenes[imagen]:
            d = md.clases[t]
            if d["nombre"] == "<Module>":
                continue
            if d["bits"] & 0x100000:
                # Un enum: sus valores son los campos estáticos, en orden.
                valores = [c["nombre"] for c in d["campos"] if c["estatico"]]
                lineas.append(f"public enum {d['nombre']} {{ {', '.join(valores)} }}")
                continue
            campos = []
            for c in d["campos"]:
                acceso = "public" if md.atributos(c["tipo"]) & 7 == 6 else "private"
                estatico = " static" if c["estatico"] else ""
                campos.append(f"    {acceso}{estatico} {md.nombre_tipo(c['tipo'], False)} {c['nombre']};")
            clase = f"public class {d['nombre']} : UnityEngine.MonoBehaviour {{\n" + "\n".join(campos) + "\n}"
            lineas.append(f"namespace {d['ns']} {{\n{clase}\n}}" if d["ns"] else clase)
        fuente = os.path.join(carpeta, imagen[:-4] + ".cs")
        with open(fuente, "w", encoding="utf-8") as out:
            out.write("\n".join(lineas) + "\n")
        subprocess.run(["dotnet", csc, "-nologo", "-noconfig", "-nostdlib+", "-nowarn:169,414,649",
                        "-target:library", "-r:" + os.path.join(carpeta, "mscorlib.dll"),
                        "-r:" + os.path.join(carpeta, "UnityEngine.dll"), "-out:" + os.path.join(carpeta, imagen),
                        fuente], check=True)


class Extractor:
    def __init__(self, release, datos, managed):
        self.env = UnityPy.load(datos)
        self.datos = datos
        self.archivos = {os.path.basename(n): f for n, f in self.env.files.items() if isinstance(f, SerializedFile)}
        self.temporal = tempfile.mkdtemp(prefix="neuro-stubs-")
        stubs(release, managed, self.temporal)
        self.generador = TypeTreeGenerator(self.archivos["mainData"].unity_version)
        self.generador.load_local_dll_folder(self.temporal)
        self.cache = {}
        tm = self.unico("mainData", "TagManager")
        self.capas_orden = {c["uniqueID"]: c["name"] for c in tm["m_SortingLayers"]}
        self.etiquetas = dict(ETIQUETAS)
        self.etiquetas.update({20000 + i: n for i, n in enumerate(tm["tags"])})
        self.sprites_usados = set()
        self.controladores_usados = set()
        self.prefabs_usados = set()
        self.audio_usado = set()
        self.estilos_usados = {}

    def cerrar(self):
        shutil.rmtree(self.temporal, ignore_errors=True)

    # --- lectura ---------------------------------------------------------------------------------

    def destino(self, archivo, ptr):
        fid, pid = ptr["m_FileID"], ptr["m_PathID"]
        if pid == 0:
            return None
        nombre = archivo.name if fid == 0 else archivo.externals[fid - 1].path
        return os.path.basename(ALIAS.get(nombre, nombre)), pid

    def leer(self, nombre, pid):
        clave = (nombre, pid)
        if clave not in self.cache:
            o = self.archivos[nombre].objects[pid]
            if o.type.name == "MonoBehaviour":
                base = o.read(check_read=False)
                s = self.destino(self.archivos[nombre], {"m_FileID": base.m_Script.m_FileID,
                                                         "m_PathID": base.m_Script.m_PathID})
                ms = self.leer(*s)[1]
                clase = (ms["m_Namespace"] + "." if ms["m_Namespace"] else "") + ms["m_ClassName"]
                d = o.read_typetree(nodes=self.generador.get_nodes_up(ms["m_AssemblyName"], clase), check_read=True)
                d["_clase"] = ms["m_ClassName"]
            else:
                d = o.read_typetree()
            self.cache[clave] = (o.type.name, d)
        return self.cache[clave]

    def seguir(self, archivo, ptr):
        d = self.destino(archivo, ptr)
        if d is None:
            return None
        tipo, datos = self.leer(*d)
        return tipo, datos, d

    def nombre_de(self, archivo, ptr):
        s = self.seguir(archivo, ptr)
        return s[1].get("m_Name") if s else None

    def de_tipo(self, nombre, tipo):
        return [(pid, self.leer(nombre, pid)[1]) for pid, o in self.archivos[nombre].objects.items()
                if o.type.name == tipo]

    def unico(self, nombre, tipo):
        return next(d for _, d in self.de_tipo(nombre, tipo))

    def raiz(self, nombre, pid):
        """La raíz de la jerarquía de un GameObject (el prefab al que pertenece)."""
        archivo = self.archivos[nombre]
        while True:
            go = self.leer(nombre, pid)[1]
            t_ptr = next(c["component"] if "component" in c else c[1] for c in go["m_Component"]
                         if self.seguir(archivo, c["component"] if "component" in c else c[1])[0] == "Transform")
            padre = self.leer(nombre, t_ptr["m_PathID"])[1]["m_Father"]["m_PathID"]
            if not padre:
                return pid
            pid = self.leer(nombre, padre)[1]["m_GameObject"]["m_PathID"]

    # --- escenas ---------------------------------------------------------------------------------

    def capa_orden(self, id_firmado):
        return self.capas_orden[id_firmado & 0xFFFFFFFF]

    def referencia(self, archivo, x):
        """Un PPtr de un campo: un objeto de la misma escena, un prefab, o un recurso por nombre."""
        s = self.seguir(archivo, x)
        if s is None:
            return None
        tipo, d, (nombre, pid) = s
        if tipo == "Transform":
            tipo, pid = "GameObject", d["m_GameObject"]["m_PathID"]
        if tipo == "GameObject":
            if nombre in ESCENAS:
                return {"objeto": pid}
            raiz = self.raiz(nombre, pid)
            self.prefabs_usados.add((nombre, raiz))
            return {"prefab": self.leer(nombre, raiz)[1]["m_Name"]}
        if tipo == "AudioClip":
            self.audio_usado.add(d["m_Name"])
        return {tipo: d.get("m_Name")}

    def campos_script(self, archivo, d):
        campos = {}
        for k, x in d.items():
            if k in ("m_GameObject", "m_Enabled", "m_Script", "m_Name", "_clase"):
                continue
            if isinstance(x, dict) and set(x) == {"m_FileID", "m_PathID"}:
                campos[k] = self.referencia(archivo, x)
            elif isinstance(x, list) and x and isinstance(x[0], dict) and set(x[0]) == {"m_FileID", "m_PathID"}:
                campos[k] = [self.referencia(archivo, e) for e in x]
            elif isinstance(x, dict) and "m_Normal" in x:
                # Un GUIStyle serializado en el script: va a los estilos del proyecto.
                nombre = f"{d['_clase']}.{k}"
                self.estilos_usados[nombre] = self.estilo(archivo, x)
                campos[k] = {"estilo": nombre}
            elif isinstance(x, dict) and set(x) == {"x", "y", "z"}:
                campos[k] = v(x)
            else:
                campos[k] = x
        return campos

    def componente(self, archivo, ptr):
        tipo, d, _ = self.seguir(archivo, ptr)
        if tipo == "Transform":
            return None
        c = {"tipo": tipo}
        if "m_Enabled" in d:
            c["activo"] = bool(d["m_Enabled"])
        if tipo == "SpriteRenderer":
            sprite = self.nombre_de(archivo, d["m_Sprite"])
            if sprite:
                self.sprites_usados.add(sprite)
            c.update(sprite=sprite, color=color(d["m_Color"]), capa_orden=self.capa_orden(d["m_SortingLayerID"]),
                     orden=d["m_SortingOrder"])
        elif tipo == "BoxCollider2D":
            c.update(trigger=bool(d["m_IsTrigger"]), centro=v(d["m_Offset"], 2),
                     material=self.nombre_de(archivo, d["m_Material"]), tamano=v(d["m_Size"], 2))
        elif tipo == "Rigidbody2D":
            c.update(masa=d["m_Mass"], arrastre=d["m_LinearDrag"], arrastre_angular=d["m_AngularDrag"],
                     escala_gravedad=d["m_GravityScale"], angulo_fijo=bool(d["m_Constraints"] & 4),
                     restricciones=d["m_Constraints"], cinematico=bool(d["m_IsKinematic"]),
                     sueno=d["m_SleepingMode"], interpolacion=d["m_Interpolate"],
                     deteccion=d["m_CollisionDetection"])
        elif tipo == "Animator":
            controlador = self.nombre_de(archivo, d["m_Controller"])
            self.controladores_usados.add(controlador)
            c.update(controlador=controlador, modo=d["m_UpdateMode"])
        elif tipo == "AudioSource":
            clip = self.nombre_de(archivo, d["m_audioClip"])
            if clip:
                self.audio_usado.add(clip)
            c.update(clip=clip, al_despertar=bool(d["m_PlayOnAwake"]), volumen=r6(d["m_Volume"]),
                     bucle=bool(d["Loop"]), mudo=bool(d["Mute"]))
        elif tipo == "MonoBehaviour":
            c.update(script=d["_clase"], campos=self.campos_script(archivo, d))
        elif tipo == "Camera":
            c.update(fondo=color(d["m_BackGroundColor"]), ortografica=bool(d["orthographic"]),
                     tamano_ortografico=d["orthographic size"], campo_vision=d["field of view"],
                     cerca=d["near clip plane"], lejos=d["far clip plane"], borrado=d["m_ClearFlags"])
        elif tipo in ("GUILayer", "FlareLayer", "AudioListener"):
            pass
        else:
            c["sin_traducir"] = True
        return c

    def objetos(self, nombre, raiz=None):
        """Los GameObjects del archivo con su Transform; con `raiz`, solo esa jerarquía (un prefab)."""
        archivo = self.archivos[nombre]
        transformes = dict(self.de_tipo(nombre, "Transform"))
        go_de_t = {pid: d["m_GameObject"]["m_PathID"] for pid, d in transformes.items()}
        lista = []
        for pid, d in sorted(self.de_tipo(nombre, "GameObject")):
            comps = [c["component"] if "component" in c else c[1] for c in d["m_Component"]]
            t_ptr = next(c for c in comps if self.seguir(archivo, c)[0] == "Transform")
            t = transformes[t_ptr["m_PathID"]]
            padre = t["m_Father"]["m_PathID"]
            lista.append({
                "id": pid, "nombre": d["m_Name"], "activo": bool(d["m_IsActive"]), "capa": d["m_Layer"],
                "etiqueta": self.etiquetas.get(d["m_Tag"], str(d["m_Tag"])),
                "posicion": v(t["m_LocalPosition"]), "rotacion": v(t["m_LocalRotation"], 4),
                "escala": v(t["m_LocalScale"]), "padre": go_de_t[padre] if padre else None,
                "hijos": [go_de_t[h["m_PathID"]] for h in t["m_Children"]],
                "componentes": [c for c in (self.componente(archivo, p) for p in comps) if c]})
        if raiz is None:
            return lista
        por_id = {o["id"]: o for o in lista}
        salida, pendientes = [], [raiz]
        while pendientes:
            o = por_id[pendientes.pop(0)]
            salida.append(o)
            pendientes += o["hijos"]
        return salida

    def prefabs(self):
        """Los prefabs que instancian los scripts (y los que instancian ellos), por su nombre."""
        salida, hechos = {}, set()
        while self.prefabs_usados - hechos:
            nombre, raiz = sorted(self.prefabs_usados - hechos)[0]
            hechos.add((nombre, raiz))
            objetos = self.objetos(nombre, raiz)
            assert objetos[0]["nombre"] not in salida, objetos[0]["nombre"]
            salida[objetos[0]["nombre"]] = objetos
        return dict(sorted(salida.items()))

    # --- Mecanim ---------------------------------------------------------------------------------

    def con_nombre(self, tipo, nombre):
        for a in self.archivos:
            if not a.startswith("sharedassets"):
                continue
            for pid, o in self.archivos[a].objects.items():
                if o.type.name == tipo and o.peek_name() == nombre:
                    return a, pid, self.leer(a, pid)[1]
        raise KeyError(nombre)

    def controladores(self):
        salida, clips_usados = {}, set()
        for nombre in sorted(self.controladores_usados):
            a, _, d = self.con_nombre("AnimatorController", nombre)
            tos = {k: n for k, n in d["m_TOS"]}
            archivo = self.archivos[a]
            clips = [self.nombre_de(archivo, c) for c in d["m_AnimationClips"]]
            clips_usados.update(clips)
            c = d["m_Controller"]
            sm = c["m_StateMachineArray"][0]["data"]
            defecto = c["m_DefaultValues"]["data"]
            tipos = {1: "float", 3: "int", 4: "bool", 9: "trigger"}
            listas = {"float": "m_FloatValues", "int": "m_IntValues", "bool": "m_BoolValues", "trigger": "m_BoolValues"}
            parametros = [{"nombre": tos[x["m_ID"]], "tipo": tipos[x["m_Type"]],
                           "defecto": defecto[listas[tipos[x["m_Type"]]]][x["m_Index"]]}
                          for x in c["m_Values"]["data"]["m_ValueArray"]]

            def transicion(t):
                t = t["data"]
                return {"destino": t["m_DestinationState"], "duracion": t["m_TransitionDuration"],
                        "duracion_fija": t["m_HasFixedDuration"], "desfase": t["m_TransitionOffset"],
                        "con_salida": t["m_HasExitTime"], "salida": t["m_ExitTime"],
                        "atomica": t["m_InterruptionSource"] == 0, "a_si_misma": t["m_CanTransitionToSelf"],
                        "condiciones": [{"modo": k["data"]["m_ConditionMode"], "parametro": tos.get(k["data"]["m_EventID"]),
                                         "umbral": k["data"]["m_EventThreshold"]} for k in t["m_ConditionConstantArray"]]}
            estados = []
            for e in sm["m_StateConstantArray"]:
                e = e["data"]
                hoja = e["m_LeafInfoArray"][0]
                clip_de = {cid: clips[hoja["m_IndexOffset"] + i] for i, cid in enumerate(hoja["m_IDArray"])}
                nodos = [n["data"] for n in e["m_BlendTreeConstantArray"][0]["data"]["m_NodeArray"]]
                assert len(nodos) == 1, "sin blend trees"
                estados.append({"nombre": tos[e["m_NameID"]], "velocidad": e["m_Speed"], "bucle": e["m_Loop"],
                                "clip": clip_de.get(nodos[0]["m_ClipID"], ""),
                                "transiciones": [transicion(t) for t in e["m_TransitionConstantArray"]]})
            salida[nombre] = {"clips": clips, "parametros": parametros, "estados": estados,
                              "defecto": sm["m_DefaultState"],
                              "desde_cualquiera": [transicion(t) for t in sm["m_AnyStateTransitionConstantArray"]]}
        return salida, clips_usados

    def clips(self, nombres):
        salida = {}
        for nombre in sorted(n for n in nombres if n):
            a, _, d = self.con_nombre("AnimationClip", nombre)
            mc = d["m_MuscleClip"]
            datos_clip = mc["m_Clip"]["data"]
            enlaces = d["m_ClipBindingConstant"]["genericBindings"]
            claves = []
            if enlaces:
                # Una sola curva: el sprite del SpriteRenderer (classID 212), en el clip comprimido.
                assert len(enlaces) == 1 and enlaces[0]["classID"] == 212 and enlaces[0]["isPPtrCurve"]
                assert datos_clip["m_DenseClip"]["m_CurveCount"] == 0 and not datos_clip["m_ConstantClip"]["data"]
                datos = datos_clip["m_StreamedClip"]["data"]
                crudo = struct.pack(f"<{len(datos)}I", *datos)
                pos = 0
                while pos < len(crudo):
                    t, n = struct.unpack_from("<fI", crudo, pos)
                    pos += 8
                    for _ in range(n):
                        _, _, _, _, valor = struct.unpack_from("<Iffff", crudo, pos)
                        pos += 20
                        claves.append([max(t, 0.0), int(valor)])
                sprites = [self.nombre_de(self.archivos[a], p) for p in d["m_ClipBindingConstant"]["pptrCurveMapping"]]
                # La clave final (t = infinito) repite la última.
                claves = [[t, sprites[i]] for t, i in claves if t < 1e30]
                self.sprites_usados.update(s for _, s in claves)
            salida[nombre] = {"fps": d["m_SampleRate"], "fin": mc["m_StopTime"], "bucle": mc["m_LoopTime"],
                              "claves": claves, "valores": []}
        return salida

    # --- sprites y texturas ----------------------------------------------------------------------

    def sprites(self):
        """Cada sprite usado como el rectángulo que envuelve su malla, con la parte de la textura que
        le corresponde según uvTransform (como en One Button Prototype)."""
        salida, por_textura = {}, {}
        for nombre in sorted(self.sprites_usados):
            a, _, d = self.con_nombre("Sprite", nombre)
            rd = d["m_RD"]
            _, t, _ = self.seguir(self.archivos[a], rd["texture"])
            xs = [p["pos"]["x"] for p in rd["vertices"]]
            ys = [p["pos"]["y"] for p in rd["vertices"]]
            u = rd["uvTransform"]
            x0, x1 = min(xs) * u["x"] + u["y"], max(xs) * u["x"] + u["y"]
            y0, y1 = min(ys) * u["z"] + u["w"], max(ys) * u["z"] + u["w"]
            ancho, alto = max(xs) - min(xs), max(ys) - min(ys)
            salida[nombre] = {"textura": t["m_Name"],
                              "rect": [x0, t["m_Height"] - y1, x1 - x0, y1 - y0],
                              "pivote": [r6(-min(xs) / ancho), r6(-min(ys) / alto)],
                              "tamano": [r6(ancho), r6(alto)], "ppu": d["m_PixelsToUnits"]}
            por_textura.setdefault(t["m_Name"], []).append(salida[nombre])
        return salida, por_textura

    def escribir_texturas(self, por_textura, enteras):
        """Las texturas de los sprites recortadas al rectángulo que ocupan (con un píxel de margen),
        y las de la GUI enteras."""
        os.makedirs(os.path.join(RAIZ, "texturas"), exist_ok=True)
        tamanos = {}
        for obj in self.env.objects:
            if obj.type.name != "Texture2D":
                continue
            nombre = obj.peek_name()
            if nombre in enteras:
                t = obj.read()
                t.image.convert("RGB").save(os.path.join(RAIZ, "texturas", nombre + ".png"))
                tamanos[nombre] = [t.m_Width, t.m_Height]
                continue
            if nombre not in por_textura:
                continue
            imagen = obj.read().image
            sprites = por_textura.pop(nombre)
            x0 = max(0, int(min(s["rect"][0] for s in sprites)) - 1)
            y0 = max(0, int(min(s["rect"][1] for s in sprites)) - 1)
            x1 = min(imagen.width, int(max(s["rect"][0] + s["rect"][2] for s in sprites)) + 2)
            y1 = min(imagen.height, int(max(s["rect"][1] + s["rect"][3] for s in sprites)) + 2)
            imagen.crop((x0, y0, x1, y1)).save(os.path.join(RAIZ, "texturas", nombre + ".png"))
            for s in sprites:
                s["rect"] = [r6(s["rect"][0] - x0), r6(s["rect"][1] - y0), r6(s["rect"][2]), r6(s["rect"][3])]
            tamanos[nombre] = [x1 - x0, y1 - y0]
        assert not por_textura, por_textura
        assert set(enteras) <= set(tamanos), set(enteras) - set(tamanos)
        return tamanos

    # --- audio -----------------------------------------------------------------------------------

    def audio(self):
        """Los clips de los AudioSource y los scripts: AAC en un contenedor MP4 (el formato de
        Unity 5.1 para WebGL) dentro de sharedassets2.resource, pasados a Ogg Vorbis."""
        os.makedirs(os.path.join(RAIZ, "audio"), exist_ok=True)
        salida = {}
        for nombre in sorted(self.audio_usado):
            _, _, d = self.con_nombre("AudioClip", nombre)
            r = d["m_Resource"]
            with open(os.path.join(self.datos, r["m_Source"]), "rb") as f:
                f.seek(r["m_Offset"])
                crudo = f.read(r["m_Size"])
            entrada = os.path.join(self.temporal, nombre + ".m4a")
            with open(entrada, "wb") as out:
                out.write(crudo)
            subprocess.run(["ffmpeg", "-loglevel", "error", "-y", "-i", entrada, "-map_metadata", "-1",
                            "-c:a", "libvorbis", "-q:a", "6", os.path.join(RAIZ, "audio", nombre + ".ogg")],
                           check=True)
            salida[nombre] = {"duracion": r6(d["m_Length"]), "canales": d["m_Channels"],
                              "frecuencia": d["m_Frequency"]}
        return salida

    # --- GUI -------------------------------------------------------------------------------------

    def estilo(self, archivo, s):
        def estado(e):
            return {"fondo": self.nombre_de(archivo, e["m_Background"]), "color": color(e["m_TextColor"])}

        def lados(x):
            return [x["m_Left"], x["m_Right"], x["m_Top"], x["m_Bottom"]]
        return {"normal": estado(s["m_Normal"]), "hover": estado(s["m_Hover"]), "active": estado(s["m_Active"]),
                "on_normal": estado(s["m_OnNormal"]), "on_hover": estado(s["m_OnHover"]),
                "on_active": estado(s["m_OnActive"]), "borde": lados(s["m_Border"]),
                "relleno": lados(s["m_Padding"]), "fuente": self.nombre_de(archivo, s["m_Font"]),
                "tamano": s["m_FontSize"], "estilo": s["m_FontStyle"], "alineacion": s["m_Alignment"],
                "ajuste": bool(s["m_WordWrap"]), "recorte": s["m_TextClipping"],
                "desplazamiento": v(s["m_ContentOffset"], 2)}

    def fuentes(self):
        """Las métricas de las fuentes de los estilos (la fuente misma no se copia)."""
        salida = {}
        for nombre in sorted({e["fuente"] for e in self.estilos_usados.values() if e["fuente"]}):
            _, _, f = self.con_nombre("Font", nombre)
            salida[nombre] = {"tamano": f["m_FontSize"], "ascenso": f["m_Ascent"],
                              "interlineado": f["m_LineSpacing"], "nombres": f["m_FontNames"]}
        return salida

    # --- proyecto --------------------------------------------------------------------------------

    def ajustes(self):
        ps = self.unico("mainData", "PlayerSettings")
        t = self.unico("mainData", "TimeManager")
        f = self.unico("mainData", "Physics2DSettings")
        tm = self.unico("mainData", "TagManager")
        bs = self.unico("mainData", "BuildSettings")
        im = self.unico("mainData", "InputManager")
        return {
            "version": self.archivos["mainData"].unity_version,
            "producto": ps["productName"], "compania": ps["companyName"],
            "pantalla_web": [ps["defaultScreenWidthWeb"], ps["defaultScreenHeightWeb"]],
            "tiempo": {"paso_fijo": t["Fixed Timestep"], "paso_maximo": t["Maximum Allowed Timestep"],
                       "escala": t["m_TimeScale"]},
            "fisica2d": {"gravedad": v(f["m_Gravity"], 2), "iteraciones_velocidad": f["m_VelocityIterations"],
                         "iteraciones_posicion": f["m_PositionIterations"],
                         "umbral_velocidad": f["m_VelocityThreshold"],
                         "correccion_lineal_maxima": f["m_MaxLinearCorrection"],
                         "correccion_angular_maxima": f["m_MaxAngularCorrection"],
                         "traslacion_maxima": f["m_MaxTranslationSpeed"], "rotacion_maxima": f["m_MaxRotationSpeed"],
                         "baumgarte": f["m_BaumgarteScale"], "baumgarte_toi": f["m_BaumgarteTimeOfImpactScale"],
                         "tiempo_para_dormir": f["m_TimeToSleep"],
                         "tolerancia_lineal": f["m_LinearSleepTolerance"],
                         "tolerancia_angular": f["m_AngularSleepTolerance"],
                         "rayos_tocan_triggers": f["m_RaycastsHitTriggers"],
                         "matriz": f["m_LayerCollisionMatrix"]},
            "capas_orden": [c["name"] for c in tm["m_SortingLayers"]],
            "capas": {str(i): n for i, n in enumerate(tm["layers"]) if n},
            "ejes": [{"nombre": a["m_Name"], "negativo": a["negativeButton"], "positivo": a["positiveButton"],
                      "negativo_alt": a["altNegativeButton"], "positivo_alt": a["altPositiveButton"],
                      "gravedad": r6(a["gravity"]), "zona_muerta": r6(a["dead"]),
                      "sensibilidad": r6(a["sensitivity"]), "salto": a["snap"], "invertir": a["invert"],
                      "tipo": a["type"], "eje": a["axis"], "mando": a["joyNum"]}
                     for a in im["m_Axes"] if a["m_Name"] in EJES],
            "niveles": [os.path.splitext(os.path.basename(n))[0] for n in bs["scenes"]],
        }

    def escribir(self):
        escena = dict(self.ajustes())
        escenas = [self.objetos(n) for n in ESCENAS]
        prefabs = self.prefabs()
        controladores, clips_usados = self.controladores()
        clips = self.clips(clips_usados)
        sprites, por_textura = self.sprites()
        # Las pantallas que dibujan titulo, intrucciones y gameover con GUI.DrawTexture.
        enteras = sorted({c["campos"]["aTexture"]["Texture2D"] for objs in escenas for o in objs
                          for c in o["componentes"] if c.get("script") in ("titulo", "intrucciones", "gameover")})
        texturas = self.escribir_texturas(por_textura, enteras)
        audio = self.audio()
        escena.update(texturas=texturas, sprites=sprites, clips=clips, controladores=controladores,
                      materiales={}, materiales_fisicos={
                          d["m_Name"]: {"friccion": d["friction"], "rebote": d["bounciness"]}
                          for a in self.archivos if a.startswith("sharedassets")
                          for _, d in self.de_tipo(a, "PhysicsMaterial2D")},
                      estilos=dict(sorted(self.estilos_usados.items())), fuentes=self.fuentes(), audio=audio,
                      escena=[], escenas=escenas, prefabs=prefabs)
        with open(os.path.join(RAIZ, "escena.json"), "w", encoding="utf-8") as out:
            json.dump(escena, out, ensure_ascii=False, indent=1)
        print("texturas:", len(texturas), "sprites:", len(sprites), "clips:", len(clips),
              "controladores:", len(controladores), "audio:", sorted(audio),
              "objetos:", [len(e) for e in escenas], "prefabs:", {n: len(p) for n, p in prefabs.items()})


if __name__ == "__main__":
    if len(sys.argv) != 4:
        sys.exit(__doc__)
    extractor = Extractor(*sys.argv[1:])
    try:
        extractor.escribir()
    finally:
        extractor.cerrar()
