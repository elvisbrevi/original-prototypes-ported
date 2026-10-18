"""Inventario y extracción de juegos Unity (3.x, 4.x Web Player, 5.x y WebGL) con UnityPy.

Uso (con el Python del venv de UnityPy, ver PLAN.md):
    python -I tools/unity_assets.py inventario <ruta>
    python -I tools/unity_assets.py exportar <ruta> <salida> [--managed <carpeta Managed>]
    python -I tools/unity_assets.py emscripten <fileloader.js> <archivo .data> <salida>

<ruta> puede ser un archivo (web.unity3d, level0, *.assets) o la carpeta *_Data de un build.
`exportar` escribe en <salida>:
    texturas/<nombre>.png       Texture2D y Sprite
    sprites.json                rectángulo, pivote, píxeles por unidad y textura de cada Sprite
    audio/<nombre>.<ext>        AudioClip (los datos tal como los guardó Unity: wav/ogg/mp3...)
    textos/<nombre>             TextAsset
    archivos/<nombre>           archivos sueltos dentro de un bundle (por ejemplo Assembly-CSharp.dll)
    escenas/<archivo>.json      jerarquía de GameObjects con su Transform y sus componentes; los
                                MonoBehaviour llevan sus campos si se da --managed (type trees
                                generados con TypeTreeGeneratorAPI a partir de los ensamblados)
`emscripten` desempaqueta el .data de un build WebGL de Unity 5 según las DataRequest de su
fileloader.js.

Los originales son datos no confiables: solo se leen, nunca se ejecutan.
"""
import collections
import json
import os
import re
import sys

import UnityPy
from UnityPy.files import BundleFile, SerializedFile


def seguro(nombre):
    """Nombre de archivo sin separadores ni rutas relativas."""
    limpio = re.sub(r"[^\w.\- ()]+", "_", nombre).strip(". ") or "_"
    return limpio[:120]


def unico(carpeta, nombre, usados):
    base, ext = os.path.splitext(seguro(nombre))
    candidato, n = base + ext, 1
    while (carpeta, candidato) in usados:
        n += 1
        candidato = f"{base}_{n}{ext}"
    usados.add((carpeta, candidato))
    return os.path.join(carpeta, candidato)


def recorrer(nombre, archivo, nivel=0):
    yield nivel, nombre, archivo
    if isinstance(archivo, BundleFile):
        for k, v in archivo.files.items():
            yield from recorrer(k, v, nivel + 1)


def version(env):
    for _, _, f in (x for k, f in env.files.items() for x in recorrer(k, f)):
        if isinstance(f, SerializedFile) and f.unity_version:
            return f.unity_version
    return None


def inventario(ruta):
    env = UnityPy.load(ruta)
    print(f"Unity {version(env)}")
    for k, f in env.files.items():
        for nivel, nombre, archivo in recorrer(k, f):
            extra = f" ({len(archivo.objects)} objetos)" if isinstance(archivo, SerializedFile) else ""
            print(f"{'  ' * nivel}{type(archivo).__name__:12} {os.path.relpath(nombre, ruta) if os.path.isabs(nombre) else nombre}{extra}")
    tipos = collections.Counter()
    nombres = collections.defaultdict(list)
    for obj in env.objects:
        tipos[obj.type.name] += 1
        if obj.type.name in ("Texture2D", "Sprite", "AudioClip", "TextAsset", "Mesh", "MonoScript", "Font"):
            try:
                nombres[obj.type.name].append(obj.peek_name() if hasattr(obj, "peek_name") else obj.read().m_Name)
            except Exception as e:  # se informa y se sigue
                nombres[obj.type.name].append(f"<error {type(e).__name__}>")
    print("\nObjetos:", ", ".join(f"{t} {n}" for t, n in tipos.most_common()))
    for t, lista in sorted(nombres.items()):
        print(f"\n{t} ({len(lista)}): {', '.join(sorted(lista))}")


def generador(env, managed):
    from UnityPy.helpers.TypeTreeGenerator import TypeTreeGenerator
    g = TypeTreeGenerator(version(env))
    g.load_local_dll_folder(managed)
    return g


def leer_campos(obj):
    try:
        return obj.read_typetree()
    except Exception as e:
        return {"_error": f"{type(e).__name__}: {e}"[:200]}


def vec(v):
    return [round(getattr(v, c), 6) for c in ("x", "y", "z", "w") if hasattr(v, c)]


def puntero(p):
    return [p.m_FileID, p.m_PathID] if p is not None else None


def escena(archivo, con_campos):
    """GameObjects de un SerializedFile con su Transform y sus componentes."""
    objetos = []
    for obj in archivo.objects.values():
        if obj.type.name != "GameObject":
            continue
        try:
            go = obj.read()
        except Exception as e:
            objetos.append({"path_id": obj.path_id, "_error": str(e)[:200]})
            continue
        datos = {"path_id": obj.path_id, "nombre": go.m_Name, "activo": getattr(go, "m_IsActive", None),
                 "capa": getattr(go, "m_Layer", None), "tag": getattr(go, "m_Tag", None), "componentes": []}
        componentes = getattr(go, "m_Components", None) or getattr(go, "m_Component", [])
        for c in componentes:
            ptr = c.component if hasattr(c, "component") else (c[1] if isinstance(c, (tuple, list)) else c)
            try:
                comp_obj = ptr.deref()
                tipo = comp_obj.type.name
                comp = {"tipo": tipo, "path_id": comp_obj.path_id}
                if tipo in ("Transform", "RectTransform"):
                    t = comp_obj.read()
                    comp.update(posicion=vec(t.m_LocalPosition), rotacion=vec(t.m_LocalRotation),
                                escala=vec(t.m_LocalScale), padre=puntero(getattr(t, "m_Father", None)))
                elif tipo == "MonoBehaviour":
                    mb = comp_obj.read()
                    script = mb.m_Script.deref_parse_as_object() if mb.m_Script else None
                    comp["script"] = getattr(script, "m_ClassName", None)
                    if con_campos:
                        comp["campos"] = leer_campos(comp_obj)
                else:
                    comp["campos"] = leer_campos(comp_obj)
            except Exception as e:
                comp = {"_error": f"{type(e).__name__}: {e}"[:200]}
            datos["componentes"].append(comp)
        objetos.append(datos)
    return objetos


def a_json(v):
    if isinstance(v, (bytes, bytearray, memoryview)):
        return f"<{len(v)} bytes>"
    if hasattr(v, "__dict__") and not isinstance(v, type):
        return {k: x for k, x in vars(v).items() if not k.startswith("_")}
    return str(v)


def imagen_sprite(sprite):
    """Imagen de un Sprite. Si UnityPy no puede usar su malla (Unity 4.x), recorta su rectángulo
    de la textura (el origen de los rectángulos de Unity está abajo a la izquierda)."""
    try:
        return sprite.image
    except Exception:
        textura = sprite.m_RD.texture.read().image
        r = sprite.m_RD.textureRect
        alto = textura.size[1]
        return textura.crop((round(r.x), round(alto - r.y - r.height), round(r.x + r.width), round(alto - r.y)))


def datos_sprite(sprite):
    r, t = sprite.m_Rect, sprite.m_RD.textureRect
    pivote = getattr(sprite, "m_Pivot", None)
    return {"nombre": sprite.m_Name, "rect": [r.x, r.y, r.width, r.height],
            "rect_textura": [t.x, t.y, t.width, t.height],
            "pivote": vec(pivote) if pivote is not None else None,
            "pixeles_por_unidad": sprite.m_PixelsToUnits,
            "textura": sprite.m_RD.texture.read().m_Name if sprite.m_RD.texture else None}


def exportar(ruta, salida, managed=None):
    env = UnityPy.load(ruta)
    if managed:
        env.typetree_generator = generador(env, managed)
    usados = set()
    cuenta = collections.Counter()
    errores = []
    sprites = []
    for sub in ("texturas", "audio", "textos", "archivos", "escenas"):
        os.makedirs(os.path.join(salida, sub), exist_ok=True)

    for k, f in env.files.items():
        for _, nombre, archivo in recorrer(k, f):
            if isinstance(archivo, SerializedFile):
                objetos = escena(archivo, bool(managed))
                if objetos:
                    destino = unico(os.path.join(salida, "escenas"), os.path.basename(nombre) + ".json", usados)
                    with open(destino, "w", encoding="utf-8") as out:
                        json.dump(objetos, out, ensure_ascii=False, indent=1, default=a_json)
                    cuenta["escenas"] += 1
            elif not isinstance(archivo, BundleFile) and not os.path.isabs(nombre):
                # Archivo suelto dentro de un bundle (ensamblados en el Web Player).
                destino = unico(os.path.join(salida, "archivos"), os.path.basename(nombre), usados)
                with open(destino, "wb") as out:
                    archivo.Position = 0
                    out.write(archivo.read_bytes(archivo.Length))
                cuenta["archivos"] += 1

    for obj in env.objects:
        tipo = obj.type.name
        try:
            if tipo in ("Texture2D", "Sprite"):
                d = obj.read()
                if tipo == "Sprite":
                    sprites.append(datos_sprite(d))
                img = imagen_sprite(d) if tipo == "Sprite" else d.image
                if img is None or img.size[0] == 0:
                    continue
                img.save(unico(os.path.join(salida, "texturas"), f"{d.m_Name or obj.path_id}.png", usados))
                cuenta[tipo] += 1
            elif tipo == "AudioClip":
                d = obj.read()
                for nombre, datos in d.samples.items():
                    with open(unico(os.path.join(salida, "audio"), nombre, usados), "wb") as out:
                        out.write(datos)
                    cuenta[tipo] += 1
            elif tipo == "TextAsset":
                d = obj.read()
                datos = d.m_Script
                datos = datos.encode("utf-8", "surrogateescape") if isinstance(datos, str) else bytes(datos)
                with open(unico(os.path.join(salida, "textos"), d.m_Name or str(obj.path_id), usados), "wb") as out:
                    out.write(datos)
                cuenta[tipo] += 1
        except Exception as e:  # se anota y se sigue con el resto
            errores.append(f"{tipo} {obj.path_id}: {type(e).__name__}: {e}"[:200])
    if sprites:
        with open(os.path.join(salida, "sprites.json"), "w", encoding="utf-8") as out:
            json.dump(sorted(sprites, key=lambda x: x["nombre"]), out, ensure_ascii=False, indent=1)
    print("Exportado:", dict(cuenta))
    for e in errores:
        print("  error:", e)


def emscripten(fileloader, datos, salida):
    texto = open(fileloader, encoding="utf-8", errors="replace").read()
    entradas = re.findall(r"new DataRequest\((\d+), (\d+), \d+, \d+\)\.open\('GET', '([^']+)'\)", texto)
    if not entradas:
        sys.exit("no encuentro DataRequest en el fileloader")
    with open(datos, "rb") as f:
        paquete = f.read()
    for inicio, fin, nombre in entradas:
        partes = [seguro(p) for p in nombre.strip("/").split("/") if p not in ("", ".", "..")]
        destino = os.path.join(salida, *partes)
        os.makedirs(os.path.dirname(destino), exist_ok=True)
        with open(destino, "wb") as out:
            out.write(paquete[int(inicio):int(fin)])
        print(f"{nombre:45} {int(fin) - int(inicio):>10} bytes")


if __name__ == "__main__":
    argumentos = sys.argv[1:]
    managed = None
    if "--managed" in argumentos:
        i = argumentos.index("--managed")
        managed = argumentos[i + 1]
        del argumentos[i:i + 2]
    orden = argumentos[0] if argumentos else ""
    if orden == "inventario" and len(argumentos) == 2:
        inventario(argumentos[1])
    elif orden == "exportar" and len(argumentos) == 3:
        exportar(argumentos[1], argumentos[2], managed)
    elif orden == "emscripten" and len(argumentos) == 4:
        emscripten(*argumentos[1:])
    else:
        sys.exit(__doc__)
