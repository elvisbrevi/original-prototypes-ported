"""Regenera assets/ desde la carpeta demo_Data del build de Linux de One Button Prototype
(Unity 5.1.2f1, Mono).

Uso (con el Python del venv de UnityPy, ver PLAN.md):
    ~/.venvs/museo/bin/python -I games/one-button-prototype/tools/extraer.py <carpeta demo_Data>

Escribe en games/one-button-prototype/assets/:
    texturas/<nombre>.png   las texturas de los sprites que se ven (recortadas al rectángulo que
                            ocupan sus sprites) y las de la piel de la GUI integrada de Unity
    fuentes/OpenSans-Regular.ttf
                            la fuente del contador de FPS, tal como viene en el build
    escena.json             las tres escenas del build (mainData, level0 y level1, en el orden de
                            BuildSettings), los sprites, los clips y controladores de Mecanim, los
                            estilos de la piel integrada (GameSkin), las fuentes y los ajustes del
                            proyecto (tiempo, física 2D, capas)

El build no trae type trees: las clases del motor se leen con los de UnityPy y los campos de los
MonoBehaviour con TypeTreeGeneratorAPI a partir de los ensamblados de Managed/. El original es un
dato no confiable: solo se lee, nunca se ejecuta.
"""
import json
import os
import struct
import sys
import zlib

import UnityPy
from UnityPy.files import SerializedFile
from UnityPy.helpers.TypeTreeGenerator import TypeTreeGenerator

RAIZ = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "assets")

# Las escenas en el orden de BuildSettings (Assets/_Niveles/1.unity, 2.unity y 3.unity): cada una
# usa el sharedassets de su número.
ESCENAS = ["mainData", "level0", "level1"]
# Los recursos integrados de Unity se nombran así en las referencias externas.
ALIAS = {"library/unity default resources": "unity default resources",
         "resources/unity_builtin_extra": "unity_builtin_extra"}
# Los estilos de la piel integrada que usan GUI.Window y GUI.Button.
ESTILOS = {"m_box": "box", "m_button": "button", "m_label": "label", "m_window": "window"}
# Propiedades de BoxCollider2D que animan los clips de agacharse (el atributo es el CRC32 del
# nombre). Los clips son de los Standard Assets hechos con Unity 4, donde el centro se llamaba
# m_Center; en Unity 5.0 pasó a ser m_Offset, así que esa curva ya no tiene a qué aplicarse.
PROPIEDADES = {zlib.crc32(n.encode()): n for n in ("m_Size.x", "m_Size.y", "m_Offset.x", "m_Offset.y",
                                                     "m_Center.x", "m_Center.y")}
ETIQUETAS = {0: "Untagged", 1: "Respawn", 2: "Finish", 3: "EditorOnly", 5: "MainCamera", 6: "Player",
             7: "GameController"}


def r6(x):
    return round(x, 6)


def v(d, n=3):
    return [r6(d[c]) for c in "xyzw"[:n]]


def color(c):
    return [r6(c[k]) for k in "rgba"]


def color32(rgba):
    return [r6(((rgba >> s) & 255) / 255) for s in (0, 8, 16, 24)]


class Extractor:
    def __init__(self, datos):
        self.env = UnityPy.load(datos)
        self.archivos = {os.path.basename(n): f for n, f in self.env.files.items() if isinstance(f, SerializedFile)}
        self.generador = TypeTreeGenerator(self.archivos["mainData"].unity_version)
        self.generador.load_local_dll_folder(os.path.join(datos, "Managed"))
        self.cache = {}
        tm = self.unico("mainData", "TagManager")
        self.capas_orden = {c["uniqueID"]: c["name"] for c in tm["m_SortingLayers"]}
        self.sprites_usados = set()
        self.controladores_usados = set()

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
                d = o.read_typetree(nodes=self.generador.get_nodes_up(ms["m_AssemblyName"], clase))
                d["_clase"] = ms["m_ClassName"]
            else:
                d = o.read_typetree()
            self.cache[clave] = (o.type.name, d)
        return self.cache[clave]

    def seguir(self, archivo, ptr):
        """(tipo, datos, (archivo, path_id)) del objeto al que apunta un PPtr, o None."""
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

    # --- escenas ---------------------------------------------------------------------------------

    def capa_orden(self, id_firmado):
        return self.capas_orden[id_firmado & 0xFFFFFFFF]

    def campos_script(self, archivo, d):
        campos = {}
        for k, x in d.items():
            # m_Script sale mal alineado del type tree generado y se resuelve aparte.
            if k in ("m_GameObject", "m_Enabled", "m_Script", "m_Name", "_clase"):
                continue
            if isinstance(x, dict) and set(x) == {"m_FileID", "m_PathID"}:
                s = self.seguir(archivo, x)
                if s is None:
                    campos[k] = None
                elif s[0] == "Transform":
                    campos[k] = {"objeto": s[1]["m_GameObject"]["m_PathID"]}
                else:
                    campos[k] = {s[0]: s[1].get("m_Name")}
            elif isinstance(x, dict) and set(x) == {"m_Bits"}:
                campos[k] = x["m_Bits"]
            elif isinstance(x, dict) and set(x) == {"x", "y", "width", "height"}:
                campos[k] = [x["x"], x["y"], x["width"], x["height"]]
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
            if d["m_Enabled"] and sprite:
                self.sprites_usados.add(sprite)
            c.update(sprite=sprite, color=color(d["m_Color"]), capa_orden=self.capa_orden(d["m_SortingLayerID"]),
                     orden=d["m_SortingOrder"])
        elif tipo in ("BoxCollider2D", "CircleCollider2D", "PolygonCollider2D"):
            c.update(trigger=bool(d["m_IsTrigger"]), centro=v(d["m_Offset"], 2),
                     material=self.nombre_de(archivo, d["m_Material"]))
            if tipo == "BoxCollider2D":
                c["tamano"] = v(d["m_Size"], 2)
            elif tipo == "CircleCollider2D":
                c["radio"] = r6(d["m_Radius"])
            else:
                c["caminos"] = [[[r6(p["x"]), r6(p["y"])] for p in camino] for camino in d["m_Points"]["m_Paths"]]
        elif tipo == "Rigidbody2D":
            # Unity 5.1 cambió fixedAngle por las restricciones: el bit 4 es FreezeRotation.
            c.update(masa=d["m_Mass"], arrastre=d["m_LinearDrag"], arrastre_angular=d["m_AngularDrag"],
                     escala_gravedad=d["m_GravityScale"], angulo_fijo=bool(d["m_Constraints"] & 4),
                     cinematico=bool(d["m_IsKinematic"]), sueno=d["m_SleepingMode"],
                     interpolacion=d["m_Interpolate"], deteccion=d["m_CollisionDetection"])
        elif tipo == "Animator":
            controlador = self.nombre_de(archivo, d["m_Controller"])
            self.controladores_usados.add(controlador)
            c.update(controlador=controlador, modo=d["m_UpdateMode"])
        elif tipo == "MonoBehaviour":
            c.update(script=d["_clase"], campos=self.campos_script(archivo, d))
        elif tipo == "Camera":
            c.update(fondo=color(d["m_BackGroundColor"]), ortografica=bool(d["orthographic"]),
                     tamano_ortografico=d["orthographic size"], campo_vision=d["field of view"],
                     cerca=d["near clip plane"], lejos=d["far clip plane"], borrado=d["m_ClearFlags"])
        elif tipo == "GUIText":
            c.update(texto=d["m_Text"], fuente=self.nombre_de(archivo, d["m_Font"]), tamano=d["m_FontSize"],
                     estilo=d["m_FontStyle"], anclaje=d["m_Anchor"], alineacion=d["m_Alignment"],
                     desplazamiento=v(d["m_PixelOffset"], 2), interlineado=d["m_LineSpacing"],
                     color=color32(d["m_Color"]["rgba"]))
        elif tipo in ("GUILayer", "FlareLayer", "AudioListener"):
            pass
        else:
            c["sin_traducir"] = True
        return c

    def objetos(self, nombre):
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
                "etiqueta": ETIQUETAS.get(d["m_Tag"], str(d["m_Tag"])),
                "posicion": v(t["m_LocalPosition"]), "rotacion": v(t["m_LocalRotation"], 4),
                "escala": v(t["m_LocalScale"]), "padre": go_de_t[padre] if padre else None,
                "hijos": [go_de_t[h["m_PathID"]] for h in t["m_Children"]],
                "componentes": [c for c in (self.componente(archivo, p) for p in comps) if c]})
        return lista

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
                        # Unity 5: sin fuente de interrupción, la transición no se interrumpe.
                        "atomica": t["m_InterruptionSource"] == 0, "a_si_misma": t["m_CanTransitionToSelf"],
                        "condiciones": [{"modo": k["data"]["m_ConditionMode"], "parametro": tos.get(k["data"]["m_EventID"]),
                                         "umbral": k["data"]["m_EventThreshold"]} for k in t["m_ConditionConstantArray"]]}
            estados = []
            for e in sm["m_StateConstantArray"]:
                e = e["data"]
                # Los clips de las hojas del estado son los de m_AnimationClips desde m_IndexOffset.
                hoja = e["m_LeafInfoArray"][0]
                clip_de = {cid: clips[hoja["m_IndexOffset"] + i] for i, cid in enumerate(hoja["m_IDArray"])}
                nodos = [n["data"] for n in e["m_BlendTreeConstantArray"][0]["data"]["m_NodeArray"]]
                estado = {"nombre": tos[e["m_NameID"]], "velocidad": e["m_Speed"], "bucle": e["m_Loop"],
                          "transiciones": [transicion(t) for t in e["m_TransitionConstantArray"]]}
                if len(nodos) == 1:
                    estado["clip"] = clip_de[nodos[0]["m_ClipID"]]
                else:
                    raiz = nodos[0]
                    assert raiz["m_BlendType"] == 0, "solo blend trees 1D"
                    estado["clip"] = ""
                    estado["mezcla"] = {"parametro": tos[raiz["m_BlendEventID"]],
                                        "umbrales": raiz["m_Blend1dData"]["data"]["m_ChildThresholdArray"],
                                        "clips": [clip_de[nodos[i]["m_ClipID"]] for i in raiz["m_ChildIndices"]]}
                estados.append(estado)
            salida[nombre] = {"clips": clips, "parametros": parametros, "estados": estados,
                              "defecto": sm["m_DefaultState"],
                              "desde_cualquiera": [transicion(t) for t in sm["m_AnyStateTransitionConstantArray"]]}
        return salida, clips_usados

    def clips(self, nombres):
        salida = {}
        for nombre in sorted(nombres):
            a, _, d = self.con_nombre("AnimationClip", nombre)
            mc = d["m_MuscleClip"]
            datos = mc["m_Clip"]["data"]["m_StreamedClip"]["data"]
            crudo = struct.pack(f"<{len(datos)}I", *datos)
            pos, claves = 0, []
            while pos < len(crudo):
                t, n = struct.unpack_from("<fI", crudo, pos)
                pos += 8
                for _ in range(n):
                    _, _, _, _, valor = struct.unpack_from("<Iffff", crudo, pos)
                    pos += 20
                    claves.append([max(t, 0.0), int(valor)])
            enlaces = d["m_ClipBindingConstant"]["genericBindings"]
            datos_clip = mc["m_Clip"]["data"]
            # Las curvas van en orden: las del clip comprimido (streamed), las densas y las constantes.
            n_stream = datos_clip["m_StreamedClip"]["curveCount"]
            assert n_stream == 1 and enlaces[0]["classID"] == 212 and enlaces[0]["isPPtrCurve"]
            assert datos_clip["m_DenseClip"]["m_CurveCount"] == 0
            constantes = datos_clip["m_ConstantClip"]["data"]
            valores = []
            for b, x in zip(enlaces[1:], constantes):
                assert b["classID"] == 61 and not b["isPPtrCurve"] and b["path"] == 0
                valores.append({"componente": "BoxCollider2D", "propiedad": PROPIEDADES[b["attribute"]],
                                "valor": r6(x)})
            sprites = [self.nombre_de(self.archivos[a], p) for p in d["m_ClipBindingConstant"]["pptrCurveMapping"]]
            # La clave final (t = infinito) repite la última.
            claves = [[t, sprites[i]] for t, i in claves if t < 1e30]
            self.sprites_usados.update(s for _, s in claves)
            salida[nombre] = {"fps": d["m_SampleRate"], "fin": mc["m_StopTime"], "bucle": mc["m_LoopTime"],
                              "claves": claves, "valores": valores}
        return salida

    # --- sprites y texturas ----------------------------------------------------------------------

    def sprites(self):
        """Cada sprite usado como un cuadrilátero: los límites de su malla (unidades, desde el
        pivote) y el rectángulo de la textura que les corresponde según uvTransform (píxel =
        posición * píxeles por unidad + pivote en la textura). Las mallas son ajustadas (Tight),
        así que el rectángulo es menor que el del sprite."""
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
                              # Con el origen arriba a la izquierda, como el PNG.
                              "rect": [x0, t["m_Height"] - y1, x1 - x0, y1 - y0],
                              "pivote": [r6(-min(xs) / ancho), r6(-min(ys) / alto)],
                              "tamano": [r6(ancho), r6(alto)], "ppu": d["m_PixelsToUnits"]}
            por_textura.setdefault(t["m_Name"], []).append(salida[nombre])
        return salida, por_textura

    def escribir_texturas(self, por_textura):
        """Las texturas recortadas al rectángulo que ocupan sus sprites (con un píxel de margen):
        la hoja de trampas de 2048x2048, por ejemplo, solo aporta los pinchos."""
        os.makedirs(os.path.join(RAIZ, "texturas"), exist_ok=True)
        tamanos = {}
        for obj in self.env.objects:
            if obj.type.name != "Texture2D":
                continue
            nombre = obj.peek_name()
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
        return tamanos

    # --- GUI -------------------------------------------------------------------------------------

    def estilos(self):
        """Los estilos de la piel integrada de Unity (GameSkin, en unity default resources) y sus
        texturas."""
        dr = self.archivos["unity default resources"]
        pid = next(p for p, o in dr.objects.items() if o.type.name == "MonoBehaviour" and o.peek_name() == "GameSkin")
        piel = dr.objects[pid].read_typetree(nodes=self.generador.get_nodes_up("UnityEngine.dll", "UnityEngine.GUISkin"))
        fuente_piel = self.nombre_de(dr, piel["m_Font"])
        texturas = set()

        def estado(e):
            fondo = self.nombre_de(dr, e["m_Background"])
            if fondo:
                texturas.add(fondo)
            return {"fondo": fondo, "color": color(e["m_TextColor"])}

        def lados(x):
            return [x["m_Left"], x["m_Right"], x["m_Top"], x["m_Bottom"]]
        salida = {}
        for campo, nombre in ESTILOS.items():
            s = piel[campo]
            salida[nombre] = {
                "normal": estado(s["m_Normal"]), "hover": estado(s["m_Hover"]), "active": estado(s["m_Active"]),
                "on_normal": estado(s["m_OnNormal"]), "on_hover": estado(s["m_OnHover"]),
                "on_active": estado(s["m_OnActive"]),
                "borde": lados(s["m_Border"]), "relleno": lados(s["m_Padding"]),
                "fuente": self.nombre_de(dr, s["m_Font"]) or fuente_piel,
                "tamano": s["m_FontSize"], "estilo": s["m_FontStyle"], "alineacion": s["m_Alignment"],
                "ajuste": s["m_WordWrap"], "recorte": s["m_TextClipping"],
                "desplazamiento": v(s["m_ContentOffset"], 2)}
        os.makedirs(os.path.join(RAIZ, "texturas"), exist_ok=True)
        tamanos = {}
        for p, o in dr.objects.items():
            if o.type.name == "Texture2D" and o.peek_name() in texturas:
                t = o.read()
                t.image.save(os.path.join(RAIZ, "texturas", t.m_Name + ".png"))
                tamanos[t.m_Name] = [t.m_Width, t.m_Height]
        assert set(tamanos) == texturas, texturas - set(tamanos)
        return salida, tamanos

    def fuentes(self):
        """Arial, la fuente integrada de la GUI (del sistema; sin datos), y OpenSansRegular, la
        del contador de FPS, que viene entera en el build."""
        salida = {}
        dr = self.archivos["unity default resources"]
        arial = next(self.leer("unity default resources", p)[1] for p, o in dr.objects.items()
                     if o.type.name == "Font" and o.peek_name() == "Arial")
        _, _, regular = self.con_nombre("Font", "OpenSansRegular")
        os.makedirs(os.path.join(RAIZ, "fuentes"), exist_ok=True)
        with open(os.path.join(RAIZ, "fuentes", "OpenSans-Regular.ttf"), "wb") as out:
            out.write(bytes(regular["m_FontData"]))
        for f in (arial, regular):
            salida[f["m_Name"]] = {"tamano": f["m_FontSize"], "ascenso": f["m_Ascent"],
                                   "interlineado": f["m_LineSpacing"], "nombres": f["m_FontNames"]}
        return salida

    # --- proyecto --------------------------------------------------------------------------------

    def ajustes(self):
        ps = self.unico("mainData", "PlayerSettings")
        t = self.unico("mainData", "TimeManager")
        f = self.unico("mainData", "Physics2DSettings")
        tm = self.unico("mainData", "TagManager")
        bs = self.unico("mainData", "BuildSettings")
        return {
            "version": self.archivos["mainData"].unity_version,
            "producto": ps["productName"], "compania": ps["companyName"],
            "pantalla_web": [ps["defaultScreenWidthWeb"], ps["defaultScreenHeightWeb"]],
            "pantalla": [ps["defaultScreenWidth"], ps["defaultScreenHeight"]],
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
            "niveles": bs["scenes"],
        }

    def escribir(self):
        escena = dict(self.ajustes())
        escenas = [self.objetos(n) for n in ESCENAS]
        controladores, clips_usados = self.controladores()
        clips = self.clips(clips_usados)
        sprites, por_textura = self.sprites()
        texturas = self.escribir_texturas(por_textura)
        estilos, texturas_gui = self.estilos()
        texturas.update(texturas_gui)
        escena.update(texturas=texturas, sprites=sprites, clips=clips, controladores=controladores,
                      materiales={}, materiales_fisicos={
                          d["m_Name"]: {"friccion": d["friction"], "rebote": d["bounciness"]}
                          for a in self.archivos if a.startswith("sharedassets")
                          for _, d in self.de_tipo(a, "PhysicsMaterial2D")},
                      estilos=estilos, fuentes=self.fuentes(), escena=[], escenas=escenas, prefabs={})
        with open(os.path.join(RAIZ, "escena.json"), "w", encoding="utf-8") as out:
            json.dump(escena, out, ensure_ascii=False, indent=1)
        print("texturas:", len(texturas), "sprites:", len(sprites), "clips:", len(clips),
              "objetos:", [len(e) for e in escenas])


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    Extractor(sys.argv[1]).escribir()
