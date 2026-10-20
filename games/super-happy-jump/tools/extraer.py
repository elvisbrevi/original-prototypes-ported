"""Regenera assets/ desde el web.unity3d original de Super Happy Jump (Unity 4.6.0f3, Web Player).

Uso (con el Python del venv de UnityPy, ver PLAN.md):
    ~/.venvs/museo/bin/python -I games/super-happy-jump/tools/extraer.py <web.unity3d>

Escribe en games/super-happy-jump/assets/:
    texturas/<nombre>.png   las texturas que usan los sprites, la piel de la GUI y las partículas,
                            decodificadas por UnityPy (DXT5, ARGB4444, RGB24, ARGB32)
    audio/<nombre>.wav      los cinco AudioClip, tal como los guardó Unity (PCM)
    escena.json             la escena (mainData), los prefabs que instancia el código, los sprites,
                            los clips y controladores de Mecanim, los sistemas de partículas, la piel
                            de la GUI y los ajustes del proyecto (tiempo, física 2D, capas)

El port lee escena.json al arrancar. La fuente "Arial Rounded Bold" que trae el bundle no se
extrae: es Arial Rounded MT Bold de Monotype, que no se puede redistribuir (ver README).
El original es un dato no confiable: solo se lee, nunca se ejecuta.
"""
import json
import os
import struct
import sys

import UnityPy
from UnityPy.files import BundleFile, SerializedFile

RAIZ = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "assets")

# Texturas que se ven: las de los sprites de la escena y los prefabs, los fondos de los estilos de
# la GUI que usa HUD.cs y las de los materiales de partículas. Los estilos button_share,
# button_facebook y button_twitter de la piel no se usan.
TEXTURAS = ["hero", "coin", "cerca", "muro", "murob", "bach", "suelo", "pixel", "pixel-large",
            "CFX_T_Smoke_4Frames", "CFX_T_Anim4_Triangle3", "titulo", "button_play", "cinta",
            "window", "base_titulo", "label_record"]
# Prefabs que instancia el código: Spawn.muro, PlatformerCharacter2D.polvo y .hitSalto.
PREFABS = ["muro", "polvo", "dobleSalto"]
ESTILOS = ["GUI", "button_play", "ventana", "font_default", "label_record", "base_titulo", "titulo"]
FUENTES_INCORPORADAS = {("library/unity default resources", 10102): "Arial"}


def cargar(ruta):
    env = UnityPy.load(ruta)
    archivos = {}
    for _, f in env.files.items():
        for nombre, a in (f.files.items() if isinstance(f, BundleFile) else []):
            if isinstance(a, SerializedFile):
                archivos[nombre] = a
    return env, archivos


class Resolvedor:
    """Sigue los PPtr (m_FileID, m_PathID) entre los archivos serializados del bundle."""

    def __init__(self, archivos):
        self.archivos = archivos
        self.cache = {}

    def destino(self, archivo, ptr):
        fid, pid = ptr["m_FileID"], ptr["m_PathID"]
        if pid == 0:
            return None
        nombre = archivo.name if fid == 0 else archivo.externals[fid - 1].path
        nombre = {"resources/unity_builtin_extra": "resources/unity_builtin_extra"}.get(nombre, nombre)
        return nombre, pid

    def objeto(self, archivo, ptr):
        d = self.destino(archivo, ptr)
        if d is None or d[0] not in self.archivos:
            return None, d
        a = self.archivos[d[0]]
        return a.objects.get(d[1]), d

    def leer(self, archivo, ptr):
        obj, d = self.objeto(archivo, ptr)
        if obj is None:
            return None, None, d
        clave = (d[0], d[1])
        if clave not in self.cache:
            self.cache[clave] = obj.read_typetree()
        return obj, self.cache[clave], d


def v(d, n=3):
    return [round(d[c], 6) for c in "xyzw"[:n]]


def color(c):
    return [round(c[k], 6) for k in "rgba"]


def color32(rgba):
    return [round(((rgba >> s) & 255) / 255, 6) for s in (0, 8, 16, 24)]


def curva(c):
    return [[round(k["time"], 6), round(k["value"], 6), round(k["inSlope"], 6), round(k["outSlope"], 6)]
            for k in c["m_Curve"]]


def min_max_curva(c):
    return {"modo": c["minMaxState"], "escalar": round(c["scalar"], 6),
            "max": curva(c["maxCurve"]), "min": curva(c["minCurve"])}


def gradiente(g):
    colores = [[round(g[f"ctime{i}"] / 65535, 6)] + color32(g[f"key{i}"]["rgba"])[:3]
               for i in range(g["m_NumColorKeys"])]
    alfas = [[round(g[f"atime{i}"] / 65535, 6), color32(g[f"key{i}"]["rgba"])[3]]
             for i in range(g["m_NumAlphaKeys"])]
    return {"colores": colores, "alfas": alfas}


def min_max_gradiente(g):
    return {"modo": g["minMaxState"], "max_color": color32(g["maxColor"]["rgba"]),
            "min_color": color32(g["minColor"]["rgba"]),
            "max": gradiente(g["maxGradient"]), "min": gradiente(g["minGradient"])}


class Extractor:
    def __init__(self, ruta):
        self.env, self.archivos = cargar(ruta)
        self.r = Resolvedor(self.archivos)
        self.main = self.archivos["mainData"]
        self.shared = self.archivos["sharedassets0.assets"]
        self.tt = {}
        for nombre, a in self.archivos.items():
            for pid, obj in a.objects.items():
                self.tt[(nombre, pid)] = (obj, obj.read_typetree())
        tm = self.unico("TagManager")
        self.etiquetas = {5: "MainCamera", 6: "Player", 0: "Untagged", 1: "Respawn", 2: "Finish",
                          3: "EditorOnly", 7: "GameController"}
        for i, t in enumerate(tm["tags"]):
            if t:
                self.etiquetas[20000 + i] = t
        self.capas_orden = {c["uniqueID"]: c["name"] for c in tm["m_SortingLayers"]}

    def unico(self, tipo):
        return next(d for (n, _), (o, d) in self.tt.items() if o.type.name == tipo)

    def de_tipo(self, archivo, tipo):
        return [(pid, d) for (n, pid), (o, d) in self.tt.items() if n == archivo and o.type.name == tipo]

    def nombre_de(self, archivo, ptr):
        obj, d, destino = self.r.leer(archivo, ptr)
        if obj is None:
            if destino in FUENTES_INCORPORADAS:
                return FUENTES_INCORPORADAS[destino]
            return None
        return d.get("m_Name")

    # --- objetos --------------------------------------------------------------------------------

    def campos_script(self, archivo, d):
        campos = {}
        for k, x in d.items():
            if k in ("m_GameObject", "m_Enabled", "m_Script", "m_Name"):
                continue
            if isinstance(x, dict) and set(x) == {"m_FileID", "m_PathID"}:
                obj, dato, destino = self.r.leer(archivo, x)
                if obj is None:
                    campos[k] = None
                elif obj.type.name == "Transform":
                    campos[k] = {"objeto": dato["m_GameObject"]["m_PathID"]}
                elif obj.type.name == "GameObject":
                    campos[k] = {"prefab": dato["m_Name"]} if destino[0] != archivo.name else {"objeto": destino[1]}
                else:
                    campos[k] = {obj.type.name: dato.get("m_Name")}
            elif isinstance(x, dict) and set(x) == {"m_Bits"}:
                campos[k] = x["m_Bits"]
            elif isinstance(x, dict) and set(x) == {"x", "y", "width", "height"}:
                campos[k] = [x["x"], x["y"], x["width"], x["height"]]
            else:
                campos[k] = x
        return campos

    def componente(self, archivo, ptr):
        obj, d, _ = self.r.leer(archivo, ptr)
        tipo = obj.type.name
        if tipo == "Transform":
            return None
        c = {"tipo": tipo}
        if "m_Enabled" in d:
            c["activo"] = bool(d["m_Enabled"])
        if tipo == "SpriteRenderer":
            c.update(sprite=self.nombre_de(archivo, d["m_Sprite"]), color=color(d["m_Color"]),
                     capa_orden=self.capas_orden[d["m_SortingLayerID"]], orden=d["m_SortingOrder"])
        elif tipo in ("BoxCollider2D", "CircleCollider2D"):
            c.update(trigger=bool(d["m_IsTrigger"]), centro=v(d["m_Center"], 2),
                     material=self.nombre_de(archivo, d["m_Material"]))
            if tipo == "BoxCollider2D":
                c["tamano"] = v(d["m_Size"], 2)
            else:
                c["radio"] = d["m_Radius"]
        elif tipo == "Rigidbody2D":
            c.update(masa=d["m_Mass"], arrastre=d["m_LinearDrag"], arrastre_angular=d["m_AngularDrag"],
                     escala_gravedad=d["m_GravityScale"], angulo_fijo=bool(d["m_FixedAngle"]),
                     cinematico=bool(d["m_IsKinematic"]), sueno=d["m_SleepingMode"],
                     interpolacion=d["m_Interpolate"], deteccion=d["m_CollisionDetection"])
        elif tipo == "Animator":
            c["controlador"] = self.nombre_de(archivo, d["m_Controller"])
        elif tipo == "AudioSource":
            c.update(clip=self.nombre_de(archivo, d["m_audioClip"]), al_despertar=d["m_PlayOnAwake"],
                     volumen=d["m_Volume"], tono=d["m_Pitch"], bucle=d["Loop"], mudo=d["Mute"])
        elif tipo == "MonoBehaviour":
            script = self.r.leer(archivo, d["m_Script"])[1]
            c.update(script=script["m_ClassName"], campos=self.campos_script(archivo, d))
        elif tipo == "Camera":
            c.update(fondo=color(d["m_BackGroundColor"]), ortografica=d["orthographic"],
                     tamano_ortografico=d["orthographic size"], campo_vision=d["field of view"],
                     cerca=d["near clip plane"], lejos=d["far clip plane"], borrado=d["m_ClearFlags"])
        elif tipo == "ParticleSystem":
            c.update(self.particulas(d))
        elif tipo == "ParticleSystemRenderer":
            c.update(material=self.nombre_de(archivo, d["m_Materials"][0]), modo=d["m_RenderMode"],
                     escala_largo=d["m_LengthScale"], escala_velocidad=d["m_VelocityScale"],
                     tamano_maximo=d["m_MaxParticleSize"], orden_particulas=d["m_SortMode"],
                     capa_orden=self.capas_orden[d["m_SortingLayerID"]], orden=d["m_SortingOrder"])
        elif tipo in ("MeshRenderer", "MeshFilter", "MeshCollider", "GUILayer", "FlareLayer",
                      "AudioListener"):
            # Los quads de GroundSpawn y Destroyer (material Default-Diffuse) quedan fuera de la
            # vista; ScreenshotView está detrás de la cámara. Se anotan sin datos.
            pass
        else:
            c["sin_traducir"] = True
        return c

    def particulas(self, d):
        im, sh, em = d["InitialModule"], d["ShapeModule"], d["EmissionModule"]
        p = {"duracion": d["lengthInSec"], "retraso": d["startDelay"], "velocidad_simulacion": d["speed"],
             "bucle": d["looping"], "precalentar": d["prewarm"], "al_despertar": d["playOnAwake"],
             "espacio_local": d["moveWithTransform"],
             "inicial": {"vida": min_max_curva(im["startLifetime"]), "rapidez": min_max_curva(im["startSpeed"]),
                         "color": min_max_gradiente(im["startColor"]), "tamano": min_max_curva(im["startSize"]),
                         "rotacion": min_max_curva(im["startRotation"]), "gravedad": im["gravityModifier"],
                         "heredar_velocidad": im["inheritVelocity"], "maximo": im["maxNumParticles"]},
             "forma": {"tipo": sh["type"], "radio": sh["radius"], "angulo": sh["angle"],
                       "largo": sh["length"], "caja": [sh["boxX"], sh["boxY"], sh["boxZ"]],
                       "direccion_aleatoria": bool(sh["randomDirection"])} if sh["enabled"] else None,
             "emision": {"tipo": em["m_Type"], "ritmo": min_max_curva(em["rate"]),
                         "rafagas": [[em[f"time{i}"], em[f"cnt{i}"]] for i in range(em["m_BurstCount"])]}
             if em["enabled"] else None}
        if d["SizeModule"]["enabled"]:
            p["tamano_en_vida"] = min_max_curva(d["SizeModule"]["curve"])
        if d["ColorModule"]["enabled"]:
            p["color_en_vida"] = min_max_gradiente(d["ColorModule"]["gradient"])
        if d["RotationModule"]["enabled"]:
            p["rotacion_en_vida"] = min_max_curva(d["RotationModule"]["curve"])
        uv = d["UVModule"]
        if uv["enabled"]:
            p["hoja"] = {"cuadro": min_max_curva(uv["frameOverTime"]), "x": uv["tilesX"], "y": uv["tilesY"],
                         "tipo": uv["animationType"], "fila": uv["rowIndex"], "ciclos": uv["cycles"],
                         "fila_aleatoria": bool(uv["randomRow"])}
        vel = d["VelocityModule"]
        if vel["enabled"]:
            p["velocidad_en_vida"] = {"x": min_max_curva(vel["x"]), "y": min_max_curva(vel["y"]),
                                      "z": min_max_curva(vel["z"]), "mundo": bool(vel["inWorldSpace"])}
        sin_traducir = [m for m in ("ForceModule", "ExternalForcesModule", "ClampVelocityModule",
                                    "SizeBySpeedModule", "RotationBySpeedModule", "ColorBySpeedModule",
                                    "CollisionModule", "SubModule") if d[m]["enabled"]]
        if sin_traducir:
            p["sin_traducir"] = sin_traducir
        return p

    def objetos(self, archivo, raiz=None):
        """GameObjects del archivo con su Transform; si `raiz`, solo esa jerarquía (un prefab)."""
        lista = []
        transformes = {pid: d for pid, d in self.de_tipo(archivo.name, "Transform")}
        go_de_t = {pid: d["m_GameObject"]["m_PathID"] for pid, d in transformes.items()}
        for pid, d in sorted(self.de_tipo(archivo.name, "GameObject")):
            comps = [c[1] for c in d["m_Component"]]
            t_ptr = next(c for c in comps if self.r.objeto(archivo, c)[0].type.name == "Transform")
            t = transformes[t_ptr["m_PathID"]]
            padre = t["m_Father"]["m_PathID"]
            o = {"id": pid, "nombre": d["m_Name"], "activo": d["m_IsActive"], "capa": d["m_Layer"],
                 "etiqueta": self.etiquetas.get(d["m_Tag"], str(d["m_Tag"])),
                 "posicion": v(t["m_LocalPosition"]), "rotacion": v(t["m_LocalRotation"], 4),
                 "escala": v(t["m_LocalScale"]), "padre": go_de_t[padre] if padre else None,
                 "hijos": [go_de_t[h["m_PathID"]] for h in t["m_Children"]],
                 "componentes": [c for c in (self.componente(archivo, p) for p in comps) if c]}
            lista.append(o)
        if raiz is None:
            return lista
        por_id = {o["id"]: o for o in lista}
        r = next(o for o in lista if o["nombre"] == raiz and o["padre"] is None)
        salida, pendientes = [], [r["id"]]
        while pendientes:
            o = por_id[pendientes.pop(0)]
            salida.append(o)
            pendientes += o["hijos"]
        return salida

    # --- recursos ---------------------------------------------------------------------------------

    def sprites(self):
        texturas = {pid: d for pid, d in self.de_tipo(self.shared.name, "Texture2D")}
        salida = {}
        for _, d in self.de_tipo(self.shared.name, "Sprite"):
            t = texturas[d["m_RD"]["texture"]["m_PathID"]]
            r = d["m_RD"]["textureRect"]
            vs = d["m_RD"]["vertices"]
            xs = [p["pos"]["x"] for p in vs]
            ys = [p["pos"]["y"] for p in vs]
            ppu = d["m_PixelsToUnits"]
            # El rectángulo en píxeles con el origen arriba a la izquierda (el PNG) y el pivote en
            # fracciones del rectángulo, desde los vértices de la malla del sprite.
            salida[d["m_Name"]] = {
                "textura": t["m_Name"],
                "rect": [r["x"], t["m_Height"] - r["y"] - r["height"], r["width"], r["height"]],
                "pivote": [round(-min(xs) / (max(xs) - min(xs)), 6), round(-min(ys) / (max(ys) - min(ys)), 6)],
                "tamano": [round(max(xs) - min(xs), 6), round(max(ys) - min(ys), 6)],
                "ppu": ppu}
        return salida

    def clips(self):
        salida = {}
        for pid, d in self.de_tipo(self.shared.name, "AnimationClip"):
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
            enlaces = d["m_ClipBindingConstant"]
            sprites = [self.tt[(self.shared.name, p["m_PathID"])][1]["m_Name"] for p in enlaces["pptrCurveMapping"]]
            assert all(b["classID"] == 212 and b["isPPtrCurve"] for b in enlaces["genericBindings"])
            salida[d["m_Name"]] = {"fps": d["m_SampleRate"], "fin": mc["m_StopTime"], "bucle": mc["m_LoopTime"],
                                   "claves": [[t, sprites[i]] for t, i in claves]}
        return salida

    def controladores(self):
        salida = {}
        for pid, d in self.de_tipo(self.shared.name, "AnimatorController"):
            tos = {k: n for k, n in d["m_TOS"]}
            clips = [self.tt[(self.shared.name, c["m_PathID"])][1]["m_Name"] for c in d["m_AnimationClips"]]
            c = d["m_Controller"]
            sm = c["m_StateMachineArray"][0]["data"]
            valores = c["m_Values"]["data"]["m_ValueArray"]
            defecto = c["m_DefaultValues"]["data"]
            tipos = {1: "float", 3: "int", 4: "bool", 9: "trigger"}
            parametros = []
            for val in valores:
                tipo = tipos[val["m_Type"]]
                lista = {"float": "m_FloatValues", "int": "m_IntValues", "bool": "m_BoolValues"}.get(tipo, "m_BoolValues")
                parametros.append({"nombre": tos[val["m_ID"]], "tipo": tipo, "defecto": defecto[lista][val["m_Index"]]})

            def transicion(t):
                t = t["data"]
                return {"destino": t["m_DestinationState"], "duracion": t["m_TransitionDuration"],
                        "desfase": t["m_TransitionOffset"], "atomica": t["m_Atomic"],
                        "a_si_misma": t["m_CanTransitionToSelf"],
                        "condiciones": [{"modo": k["data"]["m_ConditionMode"], "parametro": tos.get(k["data"]["m_EventID"]),
                                         "umbral": k["data"]["m_EventThreshold"], "salida": k["data"]["m_ExitTime"]}
                                        for k in t["m_ConditionConstantArray"]]}
            estados = []
            for e in sm["m_StateConstantArray"]:
                e = e["data"]
                nodos = e["m_BlendTreeConstantArray"][0]["data"]["m_NodeArray"]
                assert len(nodos) == 1
                estados.append({"nombre": tos.get(e["m_NameID"]), "clip": tos.get(nodos[0]["data"]["m_ClipID"]),
                                "velocidad": e["m_Speed"], "bucle": e["m_Loop"],
                                "transiciones": [transicion(t) for t in e["m_TransitionConstantArray"]]})
            salida[d["m_Name"]] = {"clips": clips, "parametros": parametros, "estados": estados,
                                   "defecto": sm["m_DefaultState"],
                                   "desde_cualquiera": [transicion(t) for t in sm["m_AnyStateTransitionConstantArray"]]}
        return salida

    def materiales(self):
        salida = {}
        for _, d in self.de_tipo(self.shared.name, "Material"):
            shader = self.r.leer(self.shared, d["m_Shader"])[1]
            nombre_shader = None
            if shader:
                guion = shader.get("m_Script", "")
                nombre_shader = guion.split('"')[1] if guion.startswith("Shader") else shader.get("m_Name")
            props = d["m_SavedProperties"]
            tex = {(t[0]["name"] if isinstance(t[0], dict) else t[0]): self.nombre_de(self.shared, t[1]["m_Texture"])
                   for t in props["m_TexEnvs"]}
            colores = {(c[0]["name"] if isinstance(c[0], dict) else c[0]): color(c[1]) for c in props["m_Colors"]}
            salida[d["m_Name"]] = {"shader": nombre_shader, "textura": tex.get("_MainTex"),
                                   "tinte": colores.get("_TintColor"), "color": colores.get("_Color")}
        return salida

    def estilos(self):
        piel = next(d for pid, d in self.de_tipo(self.shared.name, "MonoBehaviour") if d.get("m_Name") == "skin")
        fuente_piel = self.nombre_de(self.shared, piel["m_Font"])

        def estado(e):
            return {"fondo": self.nombre_de(self.shared, e["m_Background"]), "color": color(e["m_TextColor"])}

        def lados(x):
            return [x["m_Left"], x["m_Right"], x["m_Top"], x["m_Bottom"]]
        salida = {}
        for s in piel["m_CustomStyles"]:
            if s["m_Name"] not in ESTILOS:
                continue
            salida[s["m_Name"]] = {
                "normal": estado(s["m_Normal"]), "hover": estado(s["m_Hover"]), "active": estado(s["m_Active"]),
                "borde": lados(s["m_Border"]), "relleno": lados(s["m_Padding"]),
                "fuente": self.nombre_de(self.shared, s["m_Font"]) or fuente_piel,
                "tamano": s["m_FontSize"], "estilo": s["m_FontStyle"], "alineacion": s["m_Alignment"],
                "ajuste": s["m_WordWrap"], "recorte": s["m_TextClipping"],
                "desplazamiento": v(s["m_ContentOffset"], 2)}
        return salida

    def fuentes(self):
        salida = {}
        for _, d in self.de_tipo(self.shared.name, "Font"):
            salida[d["m_Name"]] = {"tamano": d["m_FontSize"], "ascenso": d["m_Ascent"],
                                   "interlineado": d["m_LineSpacing"], "nombres": d["m_FontNames"]}
        return salida

    def materiales_fisicos(self):
        return {d["m_Name"]: {"friccion": d["friction"], "rebote": d["bounciness"]}
                for _, d in self.de_tipo(self.shared.name, "PhysicsMaterial2D")}

    def ajustes(self):
        ps = self.unico("PlayerSettings")
        t = self.unico("TimeManager")
        f = self.unico("Physics2DSettings")
        tm = self.unico("TagManager")
        return {
            "version": self.main.unity_version,
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
            "capas": {str(i): n for i, n in enumerate(
                [tm.get(f"Builtin Layer {i}", "") for i in range(8)] +
                [tm.get(f"User Layer {i}", "") for i in range(8, 32)]) if n},
        }

    def escribir(self):
        os.makedirs(os.path.join(RAIZ, "texturas"), exist_ok=True)
        os.makedirs(os.path.join(RAIZ, "audio"), exist_ok=True)
        texturas = {}
        for obj in self.env.objects:
            if obj.type.name == "Texture2D":
                d = obj.read()
                if d.m_Name in TEXTURAS:
                    d.image.save(os.path.join(RAIZ, "texturas", d.m_Name + ".png"))
                    texturas[d.m_Name] = [d.m_Width, d.m_Height]
            elif obj.type.name == "AudioClip":
                d = obj.read()
                for nombre, datos in d.samples.items():
                    with open(os.path.join(RAIZ, "audio", os.path.basename(nombre)), "wb") as out:
                        out.write(datos)
        assert sorted(texturas) == sorted(TEXTURAS), set(TEXTURAS) - set(texturas)
        escena = dict(self.ajustes())
        escena.update(
            texturas=texturas, sprites=self.sprites(), clips=self.clips(),
            controladores=self.controladores(), materiales=self.materiales(),
            materiales_fisicos=self.materiales_fisicos(),
            estilos=self.estilos(), fuentes=self.fuentes(),
            escena=self.objetos(self.main),
            prefabs={n: self.objetos(self.shared, n) for n in PREFABS})
        with open(os.path.join(RAIZ, "escena.json"), "w", encoding="utf-8") as out:
            json.dump(escena, out, ensure_ascii=False, indent=1)
        print("texturas:", len(texturas), "objetos:", len(escena["escena"]),
              "prefabs:", {n: len(p) for n, p in escena["prefabs"].items()})


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    Extractor(sys.argv[1]).escribir()
