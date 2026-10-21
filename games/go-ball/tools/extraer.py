"""Regenera assets/ desde la carpeta «Go Ball (Demo)_Data» del build de Windows de Go Ball
(Unity 3.4.0f5, Mono).

Uso (con el Python del venv de UnityPy, ver PLAN.md):
    ~/.venvs/museo/bin/python -I games/go-ball/tools/extraer.py <carpeta Go Ball (Demo)_Data>

Escribe en games/go-ball/assets/:
    escena.json         las seis escenas del build (mainData y level0..4, en el orden de
                        BuildSettings) con su jerarquía y sus componentes, los ajustes de cada escena
                        (luz ambiente, cielo), los materiales con su shader y sus propiedades, las
                        fuentes de mapa de bits, la piel de la GUI, el destello de la luz, el terreno
                        del menú y los ajustes del proyecto (tiempo, física, calidad, entrada)
    mallas/<clave>.bin  las mallas: vértices, normales, UV, colores, tangentes, los triángulos de
                        cada submalla (Unity 3.4 las guarda como tiras) y los de colisión
    texturas/<clave>.png
                        las texturas que se usan (las DXT ya decodificadas), con las caras de los
                        dos cubemaps del diamante como <clave>_<cara>.png
    audio/<nombre>.ogg|.wav
                        los clips tal como vienen en el build
    terreno/            el mapa de alturas (8 bits: el máximo es 219 de 32766) y los parches de
                        pasto del terreno del menú
Y games/go-ball/src/recursos.rs, que embebe en el binario las mallas, las texturas y el audio.

Unity 3.4 guarda los type trees de las clases del motor, pero los MonoBehaviour comparten uno solo
(sin los campos del script): los de los scripts se generan con TypeTreeGeneratorAPI a partir de los
ensamblados de Managed/. La GUISkin y el Terrain son MonoBehaviour de UnityEngine.dll cuyo formato
es el nativo de esa versión y se leen a mano. El original es un dato no confiable: solo se lee,
nunca se ejecuta.
"""
import json
import os
import struct
import sys

import UnityPy
from PIL import Image
from UnityPy.files import SerializedFile
from UnityPy.helpers.TypeTreeGenerator import TypeTreeGenerator

RAIZ = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "assets")

# Las escenas en el orden de BuildSettings: menuPrincipal y nivel1..nivel5.
ESCENAS = ["mainData", "level0", "level1", "level2", "level3", "level4"]
ALIAS = {"library/unity default resources": "unity default resources"}
ETIQUETAS = {0: "Untagged", 1: "Respawn", 2: "Finish", 3: "EditorOnly", 5: "MainCamera", 6: "Player",
             7: "GameController"}
# Los shaders por el nombre con que se declaran (Shader "...").
CARAS = ["+x", "-x", "+y", "-y", "+z", "-z"]
SCRIPTS_NATIVOS = {"GUISkin", "Terrain"}
# Los estilos de la piel que usan los scripts: GUI.Button y GUI.Window con los de la piel, y los
# propios que piden con FindStyle (los demás no se exportan).
ESTILOS_USADOS = {"button", "window", "maximizar", "cerrar", "opciones", "reiniciar", "toolTip", "titulo"}
# Los caracteres que se exportan de las fuentes de mapa de bits (traen miles): ASCII y Latin-1,
# que cubren los textos del juego («¿Desea salir a la pantalla de inicio?»).
CARACTERES = set(range(32, 127)) | set(range(160, 256))


def r6(x):
    return round(float(x), 6)


def v(d, n=3):
    return [r6(d[c]) for c in "xyzw"[:n]]


def color(c):
    return [r6(c[k]) for k in "rgba"]


def color32(rgba):
    return [r6(((rgba >> s) & 255) / 255) for s in (0, 8, 16, 24)]


def seguro(nombre):
    return "".join(ch if ch.isalnum() or ch in " -_()." else "_" for ch in nombre).strip()


def duracion(datos, ext):
    """La duración en segundos de un clip: de la última página de un Ogg Vorbis (granule position
    sobre la frecuencia de la cabecera) o de los bloques fmt y data de un WAV."""
    if ext == "ogg":
        i = datos.index(b"\x01vorbis")
        frecuencia = struct.unpack_from("<I", datos, i + 12)[0]
        ultima = datos.rindex(b"OggS")
        return struct.unpack_from("<q", datos, ultima + 6)[0] / frecuencia
    pos, bytes_por_segundo = 12, None
    while pos < len(datos):
        nombre, largo = datos[pos:pos + 4], struct.unpack_from("<I", datos, pos + 4)[0]
        if nombre == b"fmt ":
            bytes_por_segundo = struct.unpack_from("<I", datos, pos + 16)[0]
        elif nombre == b"data":
            return largo / bytes_por_segundo
        pos += 8 + largo + (largo & 1)
    raise ValueError("WAV sin datos")


def triangulos(indices, tira):
    """Unity 3.4 guarda las submallas como tiras con triángulos degenerados: los separa en
    triángulos alternando el sentido, como D3D y OpenGL."""
    if not tira:
        return list(indices)
    salida = []
    for i in range(len(indices) - 2):
        a, b, c = indices[i], indices[i + 1], indices[i + 2]
        if a == b or b == c or a == c:
            continue
        salida += [a, b, c] if i % 2 == 0 else [b, a, c]
    return salida


class Lector:
    """Lectura a mano de un objeto (los formatos nativos que no describe ningún type tree)."""

    def __init__(self, datos, pos=0):
        self.b, self.p = datos, pos

    def i(self):
        x = struct.unpack_from("<i", self.b, self.p)[0]
        self.p += 4
        return x

    def f(self):
        x = struct.unpack_from("<f", self.b, self.p)[0]
        self.p += 4
        return x

    def s(self):
        n = self.i()
        x = self.b[self.p:self.p + n].decode("utf-8")
        self.p = (self.p + n + 3) & ~3
        return x

    def pptr(self):
        return {"m_FileID": self.i(), "m_PathID": self.i()}

    def booleano(self):
        x = self.b[self.p] != 0
        self.p += 4
        return x


class Extractor:
    def __init__(self, datos):
        self.datos = datos
        self.env = UnityPy.load(datos)
        self.archivos = {os.path.basename(n): f for n, f in self.env.files.items() if isinstance(f, SerializedFile)}
        self.generador = TypeTreeGenerator(self.archivos["mainData"].unity_version)
        self.generador.load_local_dll_folder(os.path.join(datos, "Managed"))
        self.cache = {}
        self.materiales = {}
        self.mallas = {}
        self.texturas = {}
        self.cubemaps = {}
        self.fuentes = {}
        self.clips = {}
        self.claves = {}
        self.usadas = set()

    # --- lectura ---------------------------------------------------------------------------------

    def destino(self, archivo, ptr):
        fid, pid = ptr["m_FileID"], ptr["m_PathID"]
        if pid == 0:
            return None
        nombre = archivo.name if fid == 0 else archivo.externals[fid - 1].path
        return os.path.basename(ALIAS.get(nombre, nombre)), pid

    def crudo(self, nombre, pid):
        o = self.archivos[nombre].objects[pid]
        o.reader.Position = o.byte_start
        return o.reader.read_bytes(o.byte_size)

    def leer(self, nombre, pid):
        clave = (nombre, pid)
        if clave not in self.cache:
            o = self.archivos[nombre].objects[pid]
            if o.type.name == "MonoBehaviour":
                base = o.read(check_read=False)
                s = self.destino(self.archivos[nombre], {"m_FileID": base.m_Script.m_FileID,
                                                         "m_PathID": base.m_Script.m_PathID})
                ms = self.leer(*s)[1]
                clase = ms["m_ClassName"]
                if clase in SCRIPTS_NATIVOS:
                    d = {"_crudo": self.crudo(nombre, pid)}
                else:
                    d = o.read_typetree(nodes=self.generador.get_nodes_up(ms["m_AssemblyName"], clase))
                d["_clase"] = clase
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

    def de_tipo(self, nombre, tipo):
        return [(pid, self.leer(nombre, pid)[1]) for pid, o in self.archivos[nombre].objects.items()
                if o.type.name == tipo]

    def unico(self, nombre, tipo):
        return next(d for _, d in self.de_tipo(nombre, tipo))

    def clave(self, tipo, ref, nombre):
        """Una clave legible y única por recurso: su nombre, o el nombre con el archivo y el id si
        otro recurso del mismo tipo se llama igual."""
        k = (tipo, ref)
        if k not in self.claves:
            usados = {c for (t, _), c in self.claves.items() if t == tipo}
            c = seguro(nombre) or "sin nombre"
            if c in usados:
                c = f"{c} ({ref[0].replace('.assets', '')}-{ref[1]})"
            self.claves[k] = c
        return self.claves[k]

    # --- recursos --------------------------------------------------------------------------------

    def textura(self, archivo, ptr):
        s = self.seguir(archivo, ptr)
        if s is None:
            return None
        tipo, d, ref = s
        clave = self.clave("textura", ref, d["m_Name"])
        if tipo == "Cubemap":
            self.cubemaps[clave] = ref
        else:
            self.texturas[clave] = ref
        return clave

    def material(self, archivo, ptr):
        s = self.seguir(archivo, ptr)
        if s is None:
            return None
        _, d, ref = s
        clave = self.clave("material", ref, d["m_Name"])
        if clave in self.materiales:
            return clave
        a = self.archivos[ref[0]]
        sh = self.seguir(a, d["m_Shader"])
        shader = sh[1]["m_Script"].split('"')[1] if sh else None
        props = d["m_SavedProperties"]
        texturas = {}
        for k, t in props["m_TexEnvs"]:
            nombre = k["name"] if isinstance(k, dict) else k
            texturas[nombre] = {"textura": self.textura(a, t["m_Texture"]), "escala": v(t["m_Scale"], 2),
                                "desplazamiento": v(t["m_Offset"], 2)}
        self.materiales[clave] = {
            "nombre": d["m_Name"], "shader": shader, "texturas": texturas,
            "flotantes": {(k["name"] if isinstance(k, dict) else k): r6(x) for k, x in props["m_Floats"]},
            "colores": {(k["name"] if isinstance(k, dict) else k): color(c) for k, c in props["m_Colors"]}}
        return clave

    def malla(self, archivo, ptr):
        s = self.seguir(archivo, ptr)
        if s is None:
            return None
        _, d, ref = s
        clave = self.clave("malla", ref, d["m_Name"])
        self.mallas[clave] = d
        return clave

    def escribir_malla(self, clave, d):
        """Formato (little endian): «MALLA3D\\0», nv, banderas (1 normales, 2 uv, 4 colores,
        8 tangentes, 16 uv2), posiciones (3 f32), normales (3 f32), uv (2 f32), uv2 (2 f32),
        colores (4 u8), tangentes (4 f32), nsub y por submalla sus índices (u32 n + n u16), y los
        triángulos de colisión (u32 n + n u16)."""
        nv = len(d["m_Vertices"])
        banderas = 0
        partes = []
        partes.append(b"".join(struct.pack("<3f", p["x"], p["y"], p["z"]) for p in d["m_Vertices"]))
        if len(d["m_Normals"]) == nv:
            banderas |= 1
            partes.append(b"".join(struct.pack("<3f", p["x"], p["y"], p["z"]) for p in d["m_Normals"]))
        if len(d["m_UV"]) == nv:
            banderas |= 2
            partes.append(b"".join(struct.pack("<2f", p["x"], p["y"]) for p in d["m_UV"]))
        if len(d["m_UV1"]) == nv:
            banderas |= 16
            partes.append(b"".join(struct.pack("<2f", p["x"], p["y"]) for p in d["m_UV1"]))
        if len(d["m_Colors"]) == nv:
            banderas |= 4
            partes.append(b"".join(struct.pack("<I", c["rgba"]) for c in d["m_Colors"]))
        if len(d["m_Tangents"]) == nv:
            banderas |= 8
            partes.append(b"".join(struct.pack("<4f", p["x"], p["y"], p["z"], p["w"]) for p in d["m_Tangents"]))
        assert d["m_Use16BitIndices"] == 1 and nv < 65536
        buf = bytes(d["m_IndexBuffer"])
        indices = struct.unpack(f"<{len(buf) // 2}H", buf)
        subs = []
        for sm in d["m_SubMeshes"]:
            ini = sm["firstByte"] // 2
            tri = triangulos(indices[ini:ini + sm["indexCount"]], sm["isTriStrip"] != 0)
            assert len(tri) // 3 == sm["triangleCount"], (clave, len(tri) // 3, sm["triangleCount"])
            subs.append(tri)
        col = list(d["m_CollisionTriangles"])
        salida = [b"MALLA3D\0", struct.pack("<2I", nv, banderas)] + partes + [struct.pack("<I", len(subs))]
        for tri in subs:
            salida += [struct.pack("<I", len(tri)), struct.pack(f"<{len(tri)}H", *tri)]
        salida += [struct.pack("<I", len(col)), struct.pack(f"<{len(col)}H", *col)]
        os.makedirs(os.path.join(RAIZ, "mallas"), exist_ok=True)
        with open(os.path.join(RAIZ, "mallas", clave + ".bin"), "wb") as out:
            out.write(b"".join(salida))
        aabb = d["m_LocalAABB"]
        return {"vertices": nv, "submallas": [len(t) // 3 for t in subs], "colision": len(col) // 3,
                "centro": v(aabb["m_Center"]), "extension": v(aabb["m_Extent"])}

    def escribir_texturas(self):
        os.makedirs(os.path.join(RAIZ, "texturas"), exist_ok=True)
        salida = {}
        for clave, (a, pid) in sorted(self.texturas.items()):
            t = self.archivos[a].objects[pid].read()
            imagen = t.image
            if t.m_TextureFormat == 1:  # Alpha8 (las fuentes): blanco con su alfa
                alfa = imagen.getchannel("A") if "A" in imagen.getbands() else imagen.convert("L")
                imagen = Image.merge("RGBA", (Image.new("L", imagen.size, 255),) * 3 + (alfa,))
            imagen.save(os.path.join(RAIZ, "texturas", clave + ".png"))
            ajustes = t.m_TextureSettings
            salida[clave] = {"tamano": [t.m_Width, t.m_Height], "formato": t.m_TextureFormat,
                             "mipmaps": bool(t.m_MipMap), "filtro": ajustes.m_FilterMode,
                             "repetir": ajustes.m_WrapMode == 0, "aniso": ajustes.m_Aniso}
        for clave, (a, pid) in sorted(self.cubemaps.items()):
            d = self.leer(a, pid)[1]
            assert d["m_TextureFormat"] == 5 and d["m_ImageCount"] == 6  # ARGB32
            datos = bytes(d["image data"])
            cara = len(datos) // 6
            w, h = d["m_Width"], d["m_Height"]
            for i, nombre in enumerate(CARAS):
                crudo = datos[i * cara:i * cara + w * h * 4]
                argb = Image.frombytes("RGBA", (w, h), crudo)
                a_, r, g, b = argb.split()
                Image.merge("RGBA", (r, g, b, a_)).save(os.path.join(RAIZ, "texturas", f"{clave}_{nombre}.png"))
            salida[clave] = {"tamano": [w, h], "formato": 5, "mipmaps": True, "filtro": 1, "repetir": False,
                             "aniso": 1, "cubemap": True}
        return salida

    def escribir_audio(self):
        os.makedirs(os.path.join(RAIZ, "audio"), exist_ok=True)
        salida = {}
        for nombre, (a, pid) in sorted(self.clips.items()):
            d = self.leer(a, pid)[1]
            datos = bytes(d["m_AudioData"])
            ext = {14: "ogg", 20: "wav"}[d["m_Type"]]
            with open(os.path.join(RAIZ, "audio", f"{nombre}.{ext}"), "wb") as out:
                out.write(datos)
            salida[nombre] = {"archivo": f"{nombre}.{ext}", "3d": d["m_3D"], "duracion": r6(duracion(datos, ext))}
        return salida

    # --- escenas ---------------------------------------------------------------------------------

    def referencia(self, archivo, x):
        """Un PPtr de un campo de script: el objeto de la escena o el recurso al que apunta."""
        s = self.seguir(archivo, x)
        if s is None:
            return None
        tipo, d, ref = s
        if tipo == "GameObject":
            return {"objeto": ref[1]}
        if "m_GameObject" in d:
            return {"componente": tipo, "objeto": d["m_GameObject"]["m_PathID"]}
        return {tipo: d.get("m_Name")}

    def campos_script(self, archivo, d):
        campos = {}
        for k, x in d.items():
            if k in ("m_GameObject", "m_Enabled", "m_Script", "m_Name", "_clase"):
                continue
            if isinstance(x, dict) and set(x) == {"m_FileID", "m_PathID"}:
                campos[k] = self.referencia(archivo, x)
            elif isinstance(x, dict) and set(x) == {"x", "y", "width", "height"}:
                campos[k] = [x["x"], x["y"], x["width"], x["height"]]
            else:
                campos[k] = x
        return campos

    def terreno(self, archivo, crudo):
        """El componente Terrain de Unity 3.4 (en UnityEngine.dll, con su formato nativo)."""
        l = Lector(crudo, 24)
        datos = self.seguir(archivo, l.pptr())
        campos = {"distancia_arboles": l.f(), "distancia_billboards": l.f(), "fundido_arboles": l.f(),
                  "maximo_arboles_completos": l.i(), "distancia_detalles": l.f(), "densidad_detalles": l.f(),
                  "error_pixel": l.f(), "distancia_mapa_base": l.f(), "lod_maximo": l.i(),
                  "sombras": l.booleano(), "lightmap": l.i(), "tamano_lightmap": l.i()}
        assert l.p == len(crudo)
        self.datos_terreno = datos
        return campos

    def particulas(self, tipo, d, archivo):
        if tipo == "EllipsoidParticleEmitter":
            return {"activo": bool(d["m_Enabled"]), "emitir": bool(d["m_Emit"]),
                    "tamano": [r6(d["minSize"]), r6(d["maxSize"])],
                    "energia": [r6(d["minEnergy"]), r6(d["maxEnergy"])],
                    "emision": [r6(d["minEmission"]), r6(d["maxEmission"])],
                    "velocidad_mundo": v(d["worldVelocity"]), "velocidad_local": v(d["localVelocity"]),
                    "velocidad_azar": v(d["rndVelocity"]), "escala_velocidad_emisor": r6(d["emitterVelocityScale"]),
                    "velocidad_tangente": v(d["tangentVelocity"]), "velocidad_angular": r6(d["angularVelocity"]),
                    "velocidad_angular_azar": r6(d["rndAngularVelocity"]), "rotacion_azar": bool(d["rndRotation"]),
                    "mundo": bool(d["Simulate in Worldspace?"]), "una_vez": bool(d["m_OneShot"]),
                    "elipsoide": v(d["m_Ellipsoid"]), "rango_minimo": r6(d["m_MinEmitterRange"])}
        if tipo == "ParticleAnimator":
            return {"animar_color": bool(d["Does Animate Color?"]),
                    "colores": [color32(d[f"colorAnimation[{i}]"]["rgba"]) for i in range(5)],
                    "eje_mundo": v(d["worldRotationAxis"]), "eje_local": v(d["localRotationAxis"]),
                    "crecer": r6(d["sizeGrow"]), "fuerza_azar": v(d["rndForce"]), "fuerza": v(d["force"]),
                    "amortiguacion": r6(d["damping"]), "autodestruir": bool(d["autodestruct"])}
        if tipo == "ParticleRenderer":
            uv = d["UV Animation"]
            return {"activo": bool(d["m_Enabled"]),
                    "materiales": [self.material(archivo, m) for m in d["m_Materials"]],
                    "estirar": d["m_StretchParticles"], "escala_largo": r6(d["m_LengthScale"]),
                    "escala_velocidad": r6(d["m_VelocityScale"]), "escala_velocidad_camara": r6(d["m_CameraVelocityScale"]),
                    "tamano_maximo": r6(d["m_MaxParticleSize"]),
                    "animacion_uv": [uv["x Tile"], uv["y Tile"], r6(uv["cycles"])]}
        if tipo == "WorldParticleCollider":
            return {"rebote": r6(d["m_BounceFactor"]), "perdida": r6(d["m_CollisionEnergyLoss"]),
                    "choca_con": d["m_CollidesWith"]["m_Bits"], "mensaje": bool(d["m_SendCollisionMessage"]),
                    "velocidad_minima": r6(d["m_MinKillVelocity"])}
        raise KeyError(tipo)

    def componente(self, archivo, ptr):
        tipo, d, ref = self.seguir(archivo, ptr)
        if tipo == "Transform":
            return None
        c = {"tipo": tipo}
        if "m_Enabled" in d:
            c["activo"] = bool(d["m_Enabled"])
        if tipo == "MeshFilter":
            c["malla"] = self.malla(archivo, d["m_Mesh"])
        elif tipo == "MeshRenderer":
            c["materiales"] = [self.material(archivo, m) for m in d["m_Materials"]]
        elif tipo in ("MeshCollider", "BoxCollider", "SphereCollider"):
            c.update(trigger=bool(d["m_IsTrigger"]), material=self.seguir(archivo, d["m_Material"]) and None)
            assert d["m_Material"]["m_PathID"] == 0, "sin PhysicMaterial: el material por defecto"
            if tipo == "MeshCollider":
                c.update(malla=self.malla(archivo, d["m_Mesh"]), convexo=bool(d["m_Convex"]),
                         esferas_suaves=bool(d["m_SmoothSphereCollisions"]))
            elif tipo == "BoxCollider":
                c.update(centro=v(d["m_Center"]), tamano=v(d["m_Size"]))
            else:
                c.update(centro=v(d["m_Center"]), radio=r6(d["m_Radius"]))
        elif tipo == "Rigidbody":
            c.update(masa=r6(d["m_Mass"]), arrastre=r6(d["m_Drag"]), arrastre_angular=r6(d["m_AngularDrag"]),
                     gravedad=bool(d["m_UseGravity"]), cinematico=bool(d["m_IsKinematic"]),
                     interpolacion=d["m_Interpolate"], restricciones=d["m_Constraints"],
                     deteccion=d["m_CollisionDetection"])
        elif tipo == "Light":
            destello = self.seguir(archivo, d["m_Flare"])
            c.update(clase=d["m_Type"], color=color(d["m_Color"]), intensidad=r6(d["m_Intensity"]),
                     rango=r6(d["m_Range"]), angulo=r6(d["m_SpotAngle"]), sombras=d["m_Shadows"]["m_Type"],
                     destello=destello[1]["m_Name"] if destello else None, modo=d["m_RenderMode"],
                     mascara=d["m_CullingMask"]["m_Bits"])
            if destello:
                self.destello = destello
        elif tipo == "Camera":
            r = d["m_NormalizedViewPortRect"]
            c.update(borrado=d["m_ClearFlags"], fondo=color(d["m_BackGroundColor"]),
                     ortografica=bool(d["orthographic"]), tamano_ortografico=r6(d["orthographic size"]),
                     campo_vision=r6(d["field of view"]), cerca=r6(d["near clip plane"]),
                     lejos=r6(d["far clip plane"]), vista=[r6(r["x"]), r6(r["y"]), r6(r["width"]), r6(r["height"])],
                     profundidad=r6(d["m_Depth"]), mascara=d["m_CullingMask"]["m_Bits"])
        elif tipo == "AudioSource":
            clip = self.seguir(archivo, d["m_audioClip"])
            if clip:
                self.clips[clip[1]["m_Name"]] = clip[2]
            c.update(clip=clip[1]["m_Name"] if clip else None, al_despertar=bool(d["m_PlayOnAwake"]),
                     volumen=r6(d["m_Volume"]), tono=r6(d["m_Pitch"]), bucle=bool(d["Loop"]), mudo=bool(d["Mute"]),
                     distancia_minima=r6(d["MinDistance"]), distancia_maxima=r6(d["MaxDistance"]),
                     paneo_2d=r6(d["Pan2D"]), atenuacion=d["rolloffMode"],
                     nivel_paneo=[[r6(k["time"]), r6(k["value"])] for k in d["panLevelCustomCurve"]["m_Curve"]])
        elif tipo == "GUITexture":
            r = d["m_PixelInset"]
            c.update(textura=self.textura(archivo, d["m_Texture"]), color=color(d["m_Color"]),
                     margen=[r6(r["x"]), r6(r["y"]), r6(r["width"]), r6(r["height"])],
                     bordes=[d["m_LeftBorder"], d["m_RightBorder"], d["m_TopBorder"], d["m_BottomBorder"]])
        elif tipo in ("EllipsoidParticleEmitter", "ParticleAnimator", "ParticleRenderer", "WorldParticleCollider"):
            c.update(self.particulas(tipo, d, archivo))
        elif tipo == "Animation":
            # Las 29 están vacías (sin clips): las dejan los modelos importados.
            assert d["m_Animation"]["m_PathID"] == 0 and not d["m_Animations"]
            c["clips"] = []
        elif tipo == "MonoBehaviour":
            if d["_clase"] == "Terrain":
                c.update(script="Terrain", campos=self.terreno(archivo, d["_crudo"]))
            else:
                c.update(script=d["_clase"], campos=self.campos_script(archivo, d))
        elif tipo == "TerrainCollider":
            c.update(arboles=bool(d["m_CreateTreeColliders"]))
        elif tipo in ("GUILayer", "FlareLayer", "AudioListener"):
            pass
        else:
            raise KeyError(tipo)
        return c

    def objetos(self, nombre):
        archivo = self.archivos[nombre]
        transformes = dict(self.de_tipo(nombre, "Transform"))
        go_de_t = {pid: d["m_GameObject"]["m_PathID"] for pid, d in transformes.items()}
        lista = []
        for pid, d in sorted(self.de_tipo(nombre, "GameObject")):
            comps = [c[1] if isinstance(c, (list, tuple)) else c["component"] for c in d["m_Component"]]
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

    def ajustes_escena(self, nombre):
        archivo = self.archivos[nombre]
        rs = self.unico(nombre, "RenderSettings")
        return {"ambiente": color(rs["m_AmbientLight"]), "niebla": bool(rs["m_Fog"]),
                "cielo": self.material(archivo, rs["m_SkyboxMaterial"]),
                "fuerza_destello": r6(rs["m_FlareStrength"]), "fuerza_halo": r6(rs["m_HaloStrength"])}

    # --- GUI -------------------------------------------------------------------------------------

    def estilo(self, l, archivo):
        e = {"nombre": l.s()}
        usado = e["nombre"] in ESTILOS_USADOS
        for k in ("normal", "hover", "active", "focused", "on_normal", "on_hover", "on_active", "on_focused"):
            ptr = l.pptr()
            fondo = self.textura(archivo, ptr) if usado else None
            e[k] = {"fondo": fondo, "color": [r6(l.f()) for _ in range(4)]}
        for k in ("borde", "margen", "relleno", "desborde"):
            e[k] = [l.i() for _ in range(4)]  # izquierda, derecha, arriba, abajo
        fuente = self.seguir(archivo, l.pptr())
        e["fuente"] = self.fuente(fuente) if fuente and usado else None
        e.update(posicion_imagen=l.i(), alineacion=l.i(), ajuste=l.booleano(), recorte=l.i(),
                 desplazamiento=[r6(l.f()), r6(l.f())], recorte_desplazado=[r6(l.f()), r6(l.f())],
                 ancho_fijo=r6(l.f()), alto_fijo=r6(l.f()), tamano=l.i(), estilo=l.i(),
                 estirar_ancho=l.booleano(), estirar_alto=l.booleano())
        return e

    def piel(self):
        """La GUISkin del juego (sharedassets0, «GUISkin»): 20 estilos fijos y 7 propios. El formato
        es el nativo de GUIStyle en Unity 3.4."""
        nombre, pid = next((a, p) for a in ("sharedassets0.assets",) for p, o in self.archivos[a].objects.items()
                           if o.type.name == "MonoBehaviour" and self.leer(a, p)[1]["_clase"] == "GUISkin")
        archivo = self.archivos[nombre]
        l = Lector(self.leer(nombre, pid)[1]["_crudo"])
        l.pptr(), l.i(), l.pptr()
        assert l.s() == "GUISkin"
        fuente = self.fuente(self.seguir(archivo, l.pptr()))
        estilos = [self.estilo(l, archivo) for _ in range(20)]
        estilos += [self.estilo(l, archivo) for _ in range(l.i())]
        estilos = [e for e in estilos if e["nombre"] in ESTILOS_USADOS]
        ajustes = {"doble_clic_palabra": l.booleano(), "triple_clic_linea": l.booleano(),
                   "cursor": [r6(l.f()) for _ in range(4)], "parpadeo": r6(l.f()),
                   "seleccion": [r6(l.f()) for _ in range(4)]}
        assert l.p == len(l.b)
        return {"fuente": fuente, "estilos": {e["nombre"]: e for e in estilos}, "ajustes": ajustes}

    def fuente(self, s):
        """Una fuente de mapa de bits de Unity 3.4: la textura y el rectángulo de cada carácter."""
        _, d, ref = s
        clave = self.clave("fuente", ref, d["m_Name"])
        if clave not in self.fuentes:
            archivo = self.archivos[ref[0]]
            mat = self.seguir(archivo, d["m_DefaultMaterial"])[1]
            textura = None
            for k, t in mat["m_SavedProperties"]["m_TexEnvs"]:
                if (k["name"] if isinstance(k, dict) else k) == "_MainTex":
                    _, td, tref = self.seguir(self.archivos[ref[0]], t["m_Texture"])
                    # La Arial integrada es dinámica: su textura se llena en marcha con la fuente
                    # del sistema y en el archivo está vacía.
                    if td["m_Width"]:
                        textura = self.clave("textura", tref, f"fuente {d['m_Name']}")
                        self.texturas[textura] = tref
            self.fuentes[clave] = {
                "textura": textura, "interlineado": r6(d["m_LineSpacing"]), "kerning": r6(d["m_Kerning"]),
                "mayusculas": d["m_ConvertCase"], "inicio_ascii": d["m_AsciiStartOffset"],
                "caracteres": {str(c["index"]): {"uv": [r6(c["uv"][k]) for k in ("x", "y", "width", "height")],
                                                 "vert": [r6(c["vert"][k]) for k in ("x", "y", "width", "height")],
                                                 "ancho": r6(c["width"])}
                               for c in d["m_CharacterRects"] if c["index"] in CARACTERES}}
        return clave

    def destello_json(self):
        _, d, ref = self.destello
        return {"nombre": d["m_Name"], "textura": self.textura(self.archivos[ref[0]], d["m_FlareTexture"]),
                "disposicion": d["m_TextureLayout"], "dentro_de_la_luz": d.get("m_UseFog"),
                "elementos": [{"imagen": e["m_ImageIndex"], "posicion": r6(e["m_Position"]), "tamano": r6(e["m_Size"]),
                               "color": color(e["m_Color"]), "color_luz": bool(e["m_UseLightColor"]),
                               "girar": bool(e["m_Rotate"]), "zoom": bool(e["m_Zoom"]), "fundido": bool(e["m_Fade"])}
                              for e in d["m_Elements"]]}

    # --- terreno ---------------------------------------------------------------------------------

    def escribir_terreno(self):
        _, d, ref = self.datos_terreno
        archivo = self.archivos[ref[0]]
        carpeta = os.path.join(RAIZ, "terreno")
        os.makedirs(carpeta, exist_ok=True)
        hm = d["m_Heightmap"]
        w, h = hm["m_Width"], hm["m_Height"]
        alturas = hm["m_Heights"]
        assert max(alturas) < 256, "el mapa de alturas cabe en 8 bits"
        # m_Heights va fila por fila en z (cada fila, todas las x).
        Image.frombytes("L", (w, h), bytes(alturas)).save(os.path.join(carpeta, "alturas.png"))
        sd = d["m_SplatDatabase"]
        capas = [{"textura": self.textura(archivo, s["texture"]), "tamano": v(s["tileSize"], 2),
                  "desplazamiento": v(s["tileOffset"], 2)} for s in sd["m_Splats"]]
        mezcla = [self.textura(archivo, a) for a in sd["m_AlphaTextures"]]
        dd = d["m_DetailDatabase"]
        n = dd["m_PatchCount"]
        parches = []
        for i, p in enumerate(dd["m_Patches"]):
            if p["numberOfObjects"]:
                parches.append({"x": i % n, "z": i // n, "capas": p["layerIndices"],
                                "cantidades": list(p["numberOfObjects"])})
        with open(os.path.join(carpeta, "parches.json"), "w", encoding="utf-8") as out:
            json.dump(parches, out, separators=(",", ":"))
        tinte = dd["WavingGrassTint"]
        return {"nombre": d["m_Name"], "alturas": [w, h], "escala": v(hm["m_Scale"]),
                "capas": capas, "mezcla": mezcla, "resolucion_mezcla": sd["m_AlphamapResolution"],
                "resolucion_mapa_base": sd["m_BaseMapResolution"],
                "detalles": {"parches": n, "muestras": dd["m_PatchSamples"],
                             "prototipos": [{"textura": self.textura(archivo, p["prototypeTexture"]),
                                             "ancho": [r6(p["minWidth"]), r6(p["maxWidth"])],
                                             "alto": [r6(p["minHeight"]), r6(p["maxHeight"])],
                                             "ruido": r6(p["noiseSpread"]), "doblez": r6(p["bendFactor"]),
                                             "sano": color(p["healthyColor"]), "seco": color(p["dryColor"]),
                                             "modo": p["renderMode"]} for p in dd["m_DetailPrototypes"]],
                             "tinte": color(tinte), "fuerza": r6(dd["m_WavingGrassStrength"]),
                             "cantidad": r6(dd["m_WavingGrassAmount"]), "rapidez": r6(dd["m_WavingGrassSpeed"])},
                "arboles": len(dd["m_TreeInstances"])}

    def escribir_recursos(self):
        """src/recursos.rs: los archivos de assets/ embebidos en el binario con include_bytes!."""
        def lista(nombre, carpeta, ext):
            archivos = sorted(f for f in os.listdir(os.path.join(RAIZ, carpeta)) if f.endswith(ext) or not ext)
            filas = [f'    ({json.dumps(os.path.splitext(f)[0] if ext else f)}, include_bytes!("../assets/{carpeta}/{f}")),'
                     for f in archivos]
            return ["#[rustfmt::skip]", f"pub static {nombre}: &[(&str, &[u8])] = &["] + filas + ["];", ""]
        lineas = ["// Generado por tools/extraer.py: los assets embebidos en el binario.", ""]
        lineas += lista("MALLAS", "mallas", ".bin")
        lineas += lista("TEXTURAS", "texturas", ".png")
        lineas += lista("AUDIO", "audio", "")
        with open(os.path.join(RAIZ, "..", "src", "recursos.rs"), "w", encoding="utf-8") as out:
            out.write("\n".join(lineas))

    # --- proyecto --------------------------------------------------------------------------------

    def ajustes(self):
        ps = self.unico("mainData", "PlayerSettings")
        t = self.unico("mainData", "TimeManager")
        f = self.unico("mainData", "PhysicsManager")
        q = self.unico("mainData", "QualitySettings")
        bs = self.unico("mainData", "BuildSettings")
        im = self.unico("mainData", "InputManager")
        nivel = ["Fastest", "Fast", "Simple", "Good", "Beautiful", "Fantastic"][q["m_DefaultStandaloneQuality"]]
        calidad = q[nivel]
        return {
            "version": self.archivos["mainData"].unity_version,
            "producto": ps["productName"], "compania": ps["companyName"],
            "pantalla": [ps["defaultScreenWidth"], ps["defaultScreenHeight"]],
            "pantalla_web": [ps["defaultScreenWidthWeb"], ps["defaultScreenHeightWeb"]],
            "pantalla_completa": bool(ps["defaultIsFullScreen"]), "camino_render": ps["m_RenderingPath"],
            "tiempo": {"paso_fijo": t["Fixed Timestep"], "paso_maximo": t["Maximum Allowed Timestep"],
                       "escala": t["m_TimeScale"]},
            "fisica": {"gravedad": v(f["m_Gravity"]), "umbral_rebote": r6(f["m_BounceThreshold"]),
                       "velocidad_dormir": r6(f["m_SleepVelocity"]),
                       "velocidad_angular_dormir": r6(f["m_SleepAngularVelocity"]),
                       "velocidad_angular_maxima": r6(f["m_MaxAngularVelocity"]),
                       "penetracion_minima": r6(f["m_MinPenetrationForPenalty"]),
                       "iteraciones": f["m_SolverIterationCount"],
                       "rayos_tocan_triggers": bool(f["m_RaycastsHitTriggers"])},
            "calidad": {"nivel": nivel, "luces_pixel": calidad["pixelLightCount"],
                        "antialiasing": calidad["antiAliasing"], "vegetacion_suave": calidad["softVegetation"],
                        "particulas_suaves": calidad["softParticles"], "anisotropico": calidad["anisotropicTextures"],
                        "sincronia": calidad["vSyncCount"]},
            # Un eje puede tener varias entradas con el mismo nombre (teclado y mando): GetAxis
            # devuelve la de mayor valor absoluto.
            "ejes": [{"nombre": a["m_Name"], "negativo": a["negativeButton"], "positivo": a["positiveButton"],
                      "negativo_alt": a["altNegativeButton"], "positivo_alt": a["altPositiveButton"],
                      "gravedad": r6(a["gravity"]), "zona_muerta": r6(a["dead"]),
                      "sensibilidad": r6(a["sensitivity"]), "salto": a["snap"], "invertir": a["invert"],
                      "tipo": a["type"], "eje": a["axis"], "mando": a["joyNum"]}
                     for a in im["m_Axes"] if a["m_Name"] in ("Horizontal", "Saltar")],
            "niveles": [os.path.splitext(os.path.basename(n))[0] for n in bs["levels"]],
            "desarrollo": bool(bs["isDebugBuild"]),
        }

    def escribir(self):
        escena = dict(self.ajustes())
        escenas = []
        for nombre, n in zip(ESCENAS, escena["niveles"]):
            escenas.append({"nombre": n, "ajustes": self.ajustes_escena(nombre), "objetos": self.objetos(nombre)})
        piel = self.piel()
        destello = self.destello_json()
        terreno = self.escribir_terreno()
        mallas = {clave: self.escribir_malla(clave, d) for clave, d in sorted(self.mallas.items())}
        escena.update(escenas=escenas, materiales=dict(sorted(self.materiales.items())), mallas=mallas,
                      piel=piel, fuentes=self.fuentes, destello=destello, terreno=terreno,
                      audio=self.escribir_audio(), texturas=self.escribir_texturas())
        with open(os.path.join(RAIZ, "escena.json"), "w", encoding="utf-8") as out:
            json.dump(escena, out, ensure_ascii=False, indent=1)
        self.escribir_recursos()
        print("escenas:", [len(e["objetos"]) for e in escenas], "mallas:", len(mallas),
              "materiales:", len(self.materiales), "texturas:", len(escena["texturas"]),
              "audio:", len(escena["audio"]), "fuentes:", len(self.fuentes))


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    Extractor(sys.argv[1]).escribir()
