"""Inventario y descarga de los juegos originales publicados en https://elvisbrevi.itch.io.

Uso:
    python3 -I tools/originales.py inventario            # lee las páginas -> originales/catalogo.json
    python3 -I tools/originales.py descargar [slug ...]  # baja los archivos a originales/<slug>/

Los juegos antiguos se subieron por la web de itch.io, sin canales de butler, así que `butler fetch`
no puede bajarlos. Se usa el mismo flujo público del botón "Download" (CSRF + /file/<upload_id>).
Los juegos HTML5 se bajan rastreando los archivos del iframe de html-classic.itch.zone, y los de
Unity Web Player desde la URL firmada de su web.unity3d. Los enlaces externos (Google Play) se anotan.
Los archivos descargados son datos no confiables: no se ejecutan, solo se leen.
"""
import hashlib
import html
import http.cookiejar
import json
import os
import re
import sys
import time
import urllib.parse
import urllib.request

PERFIL = "https://elvisbrevi.itch.io"
RAIZ = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "originales")
CATALOGO = os.path.join(RAIZ, "catalogo.json")
EXCLUIDOS = {"imn-2026"}  # IMÁN 2026 es la prueba de publicación del port, no un original.
PAUSA = 2.0  # segundos entre peticiones (itch.io responde 429 si se va muy rápido)
EXTENSIONES = ("js", "json", "html", "css", "png", "jpg", "jpeg", "gif", "webp", "svg", "ogg", "m4a",
               "mp3", "wav", "webm", "mp4", "ttf", "woff", "woff2", "xml", "txt", "csv", "wasm", "data",
               "swf", "unityweb", "mem", "appcache", "ico", "fnt", "atlas", "pck", "zip")

cookies = http.cookiejar.CookieJar()
opener = urllib.request.build_opener(urllib.request.HTTPCookieProcessor(cookies))
opener.addheaders = [("User-Agent", "Mozilla/5.0 (original-prototypes-ported)")]


def pedir(url, datos=None, intentos=5):
    for i in range(intentos):
        time.sleep(PAUSA)
        try:
            with opener.open(url, data=datos, timeout=120) as r:
                return r.read()
        except urllib.error.HTTPError as e:
            if e.code == 429 and i + 1 < intentos:
                time.sleep(15 * (i + 1))
                continue
            raise


def texto(fragmento):
    return html.unescape(re.sub(r"<[^>]+>", " ", fragmento)).strip()


def juegos_del_perfil():
    d = pedir(PERFIL).decode("utf-8")
    juegos = {}
    for url, gid, titulo in re.findall(
            r'class="title game_link" href="([^"]+)"[^>]*data-label="game:(\d+):title">([^<]+)</a>', d):
        slug = url.rstrip("/").rsplit("/", 1)[1]
        if slug not in EXCLUIDOS:
            juegos.setdefault(slug, {"slug": slug, "titulo": html.unescape(titulo), "id": int(gid), "url": url})
    return list(juegos.values())


def leer_pagina(juego):
    d = pedir(juego["url"]).decode("utf-8")
    info = {}
    i = d.rfind("<table", 0, d.find("<td>Status"))
    if i >= 0:
        tabla = d[i:d.find("</table>", i)]
        for clave, valor in re.findall(r"<tr><td>(.*?)</td><td>(.*?)</td></tr>", tabla):
            fecha = re.search(r'<abbr title="([^"]+)"', valor)
            info[texto(clave)] = fecha.group(1) if fecha else re.sub(r"\s+", " ", texto(valor))
    juego["info"] = info
    descripcion = re.search(r'<div class="formatted_description user_formatted">(.*?)</div>\s*<div', d, re.S)
    juego["descripcion"] = re.sub(r"\s+", " ", texto(descripcion.group(1))) if descripcion else ""
    juego["archivos"] = []
    for bloque in re.findall(r'<div class="upload">(.*?)</div></div>', d, re.S):
        uid = re.search(r'data-upload_id="(\d+)"', bloque)
        nombre = re.search(r'<strong[^>]*title="([^"]+)"', bloque)
        tamano = re.search(r'class="file_size"><span>([^<]+)</span>', bloque)
        if uid:
            juego["archivos"].append({
                "upload_id": int(uid.group(1)),
                "nombre": html.unescape(nombre.group(1)) if nombre else None,
                "tamano_itch": tamano.group(1) if tamano else None,
                "plataformas": re.findall(r'title="Download for ([^"]+)"', bloque),
            })
    iframe = re.search(r'src="(https://html[^"]*\.itch\.zone/html/[^"]+?)(?:\?[^"]*)?"', d)
    if not iframe:
        iframe = re.search(r'data-iframe="[^"]*?src=&quot;(https://html[^&]*\.itch\.zone/html/[^&]+?)(?:\?[^&]*)?&quot;', d)
    juego["html5"] = iframe.group(1) if iframe else None
    unity = re.search(r'(https:\\/\\/[^"\']+?\.unity3d\?[^"\']+)', d)
    juego["_unity_web"] = unity.group(1).replace("\\/", "/") if unity else None
    juego["unity_web_player"] = bool(unity)
    juego["externos"] = sorted(set(re.findall(r'https://play\.google\.com/store/apps/details\?id=[\w.]+', d)))
    juego["csrf"] = re.search(r'<meta name="csrf_token" value="([^"]+)"', d).group(1)


def guardar_catalogo(juegos):
    os.makedirs(RAIZ, exist_ok=True)
    limpio = [{k: v for k, v in j.items() if k != "csrf" and not k.startswith("_")} for j in juegos]
    with open(CATALOGO, "w", encoding="utf-8") as f:
        json.dump(limpio, f, ensure_ascii=False, indent=2)
        f.write("\n")


def inventario():
    juegos = juegos_del_perfil()
    for j in juegos:
        leer_pagina(j)
        print(f"{j['slug']:28} {j['info'].get('Made with', '?'):12} archivos={len(j['archivos'])} "
              f"html5={bool(j['html5'])} unity_web={j['unity_web_player']} externos={j['externos']}")
    guardar_catalogo(juegos)


def sha256(ruta):
    h = hashlib.sha256()
    with open(ruta, "rb") as f:
        for bloque in iter(lambda: f.read(1 << 20), b""):
            h.update(bloque)
    return h.hexdigest()


def bajar_archivo(juego, archivo, destino):
    ruta = os.path.join(destino, os.path.basename(archivo["nombre"] or f"upload_{archivo['upload_id']}"))
    if os.path.exists(ruta) and os.path.getsize(ruta) > 0:
        archivo["sha256"], archivo["bytes"] = sha256(ruta), os.path.getsize(ruta)
        return
    datos = urllib.parse.urlencode({"csrf_token": juego["csrf"]}).encode()
    url = f"{juego['url']}/file/{archivo['upload_id']}?source=view_game&as_props=1&after_download_lightbox=true"
    respuesta = json.loads(pedir(url, datos))
    if "url" not in respuesta:
        raise RuntimeError(f"itch.io no dio URL de descarga: {list(respuesta)}")
    with open(ruta, "wb") as f:
        f.write(pedir(respuesta["url"]))
    archivo["sha256"] = sha256(ruta)
    archivo["bytes"] = os.path.getsize(ruta)


def bajar_html5(juego, destino):
    """Rastrea index.html y todo lo que referencian sus archivos de texto (rutas relativas)."""
    base = juego["html5"].rsplit("/", 1)[0] + "/"
    raiz = os.path.join(destino, "html5")
    pendientes, vistos, bajados = ["index.html"], set(), []
    patron = re.compile(r"""["'(]([A-Za-z0-9_\-./ %]+\.(?:""" + "|".join(EXTENSIONES) + r"""))["')?#]""", re.I)
    while pendientes:
        rel = urllib.parse.unquote(pendientes.pop(0)).lstrip("./")
        if rel in vistos or ".." in rel or rel.startswith(("http", "/")):
            continue
        vistos.add(rel)
        ruta = os.path.join(raiz, rel)
        if os.path.exists(ruta):  # ya bajado en una ejecución anterior: solo se vuelve a leer
            with open(ruta, "rb") as f:
                datos = f.read()
        else:
            try:
                datos = pedir(base + urllib.parse.quote(rel))
            except urllib.error.HTTPError:
                continue
            os.makedirs(os.path.dirname(ruta), exist_ok=True)
            with open(ruta, "wb") as f:
                f.write(datos)
        bajados.append(rel)
        extension = rel.rsplit(".", 1)[-1].lower()
        if extension == "appcache":  # manifiesto de Construct: una ruta por línea, sin comillas
            pendientes += [l.strip() for l in datos.decode("utf-8-sig", "replace").splitlines()
                           if l.strip() and not l.startswith(("#", "CACHE", "NETWORK", "FALLBACK", "*"))]
        elif extension in ("html", "js", "json", "css", "xml", "txt", "fnt", "atlas"):
            carpeta = os.path.dirname(rel)
            for ref in patron.findall(datos.decode("utf-8", "replace")):
                pendientes.append(ref)
                if carpeta:
                    pendientes.append(f"{carpeta}/{ref}")
    juego["html5_archivos"] = len(bajados)
    juego["html5_bytes"] = sum(os.path.getsize(os.path.join(raiz, r)) for r in bajados)


def descargar(slugs):
    juegos = json.load(open(CATALOGO, encoding="utf-8"))
    for j in juegos:
        if slugs and j["slug"] not in slugs:
            continue
        leer_pagina(j)  # renueva cookies y token CSRF
        destino = os.path.join(RAIZ, j["slug"])
        os.makedirs(destino, exist_ok=True)
        for a in j["archivos"]:
            try:
                bajar_archivo(j, a, destino)
                print(f"{j['slug']}: {a['nombre']} {a['bytes']} bytes")
            except Exception as e:  # se documenta y se sigue con el resto
                a["error"] = str(e)
                print(f"{j['slug']}: ERROR {a['nombre']}: {e}")
        if j["_unity_web"]:
            ruta = os.path.join(destino, "web.unity3d")
            with open(ruta, "wb") as f:
                f.write(pedir(j["_unity_web"]))
            j["unity_web_sha256"], j["unity_web_bytes"] = sha256(ruta), os.path.getsize(ruta)
            print(f"{j['slug']}: web.unity3d {j['unity_web_bytes']} bytes")
        if j["html5"]:
            bajar_html5(j, destino)
            print(f"{j['slug']}: html5 {j['html5_archivos']} archivos, {j['html5_bytes']} bytes")
        guardar_catalogo(juegos)


if __name__ == "__main__":
    orden = sys.argv[1] if len(sys.argv) > 1 else ""
    if orden == "inventario":
        inventario()
    elif orden == "descargar":
        descargar(set(sys.argv[2:]))
    else:
        sys.exit(__doc__)
