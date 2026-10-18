# Plan del museo: portar los prototipos originales a macroquad

Plan de trabajo para recrear en Rust + macroquad todos los juegos publicados en
[elvisbrevi.itch.io](https://elvisbrevi.itch.io) (menos IMÁN 2026, que solo fue una prueba de publicación).
Está pensado para avanzar en varias sesiones: marca cada casilla al completarla e incluye el cambio
del plan en el mismo commit.

**Siguiente paso:** 2.4 Retro Penal (Construct 2): reutilizar la estructura de
`games/que-no-caiga` (lector de `data.js`, runtime parcial de Construct 2) y anotar los nombres de
su tabla de referencias; usa Turret, CustomMovement, Bullet, Solid y Wrap. El 0.7 (GitHub Pages)
queda pendiente de decidir si se publica.

## Reglas

1. **Fidelidad.** La lógica se traduce del original (código decompilado o datos del motor), no se
   reinventa. Las rarezas del original se conservan y se documentan.
2. **Assets originales**, extraídos de los binarios. Si uno no se puede extraer, se usa un sustituto
   provisional y se anota como pendiente.
3. **Cada port tiene versión nativa (PC) y web (wasm)**, con controles táctiles que aparecen solos en
   móviles y tablets.
4. **No se publica en itch.io.** Los ports viven en este repo.
5. **Los impedimentos no bloquean.** Audio que no se puede convertir, formatos propietarios o binarios
   que faltan se anotan como pendientes (en el README del juego y en la tabla de la fase 1) y se sigue
   con lo demás.
6. **Evidencia con REA.** El triaje y la reconstrucción se hacen con [REA](https://rea.tools/); los IDs
   de Evidence se anotan en el README de cada juego.

## Retomar en otra sesión

```sh
# Rust y web
rustup target add wasm32-unknown-unknown
sudo apt install libasound2-dev unar p7zip-full      # ALSA para macroquad; RAR/7z para los originales

# REA (Claude Code: reiniciar después del setup para tener sus herramientas MCP)
npm install --global rea-agents
rea setup --client claude_code --dry-run              # revisar el plan y luego aplicarlo con --yes

# .NET (XNA y Unity con Mono): decompilador con referencias resueltas
sudo apt install dotnet-sdk-8.0
dotnet tool install -g ilspycmd --version 9.1.0.7988  # la última versión exige .NET 10

# Originales (no están en git): bajarlos y comprobar sha256 contra originales/catalogo.json
python3 -I tools/originales.py descargar

# Unity (3.4 a 5.1, Web Player y WebGL): UnityPy y type trees de MonoBehaviour desde los ensamblados
sudo apt install python3.12-venv
python3 -m venv ~/.venvs/museo && ~/.venvs/museo/bin/pip install -r tools/requirements-unity.txt

# Android: JADX 1.5.6 y apktool 3.0.3 (Java 21) en ~/.local/opt
sudo apt install openjdk-21-jre-headless
curl -LO https://github.com/skylot/jadx/releases/download/v1.5.6/jadx-1.5.6.zip   # unzip -d ~/.local/opt/jadx
curl -L -o ~/.local/opt/apktool/apktool_3.0.3.jar \
  https://github.com/iBotPeaches/Apktool/releases/download/v3.0.3/apktool_3.0.3.jar

# Tests de humo (checkpoint 0.9): Playwright con el Chromium de la caché de Playwright o $CHROMIUM
sudo apt install xvfb xdotool imagemagick
npm install --prefix tools/humo
node tools/humo/nativo.mjs <juego>   # tras cargo build --release -p <juego>
node tools/humo/web.mjs <juego>      # tras tools/build_web.sh <juego>
```

Herramientas de extracción por stack (checkpoint 0.8; las salidas van a `originales/<slug>/`):

- XNA 3.1: `games/iman/tools/xnb_extract.py` (XNB v4 sin comprimir, `SpriteFont`, bancos XACT).
- Unity: `~/.venvs/museo/bin/python -I tools/unity_assets.py inventario|exportar|emscripten ...`.
  Lee Unity 3.4, 4.6 (`web.unity3d`) y 5.1; con `--managed <carpeta Managed>` vuelca también los
  campos de los MonoBehaviour. AssetRipper no hizo falta.
- Construct 2: `python3 -I tools/c2_eventos.py resumen|eventos|fuente <carpeta html5>` (proyecto,
  tipos, layouts, tabla de referencias del runtime minificado y hojas de eventos en pseudocódigo).
- Android: `~/.local/opt/jadx/bin/jadx -d <salida> <apk>` y
  `java -jar ~/.local/opt/apktool/apktool_3.0.3.jar d -o <salida> <apk>`. Para las herramientas
  Android de REA falta un JAR de JADX MCP en `REA_JADX_MCP_JAR` (checkpoint 2.8).

## Fase 0 — Infraestructura

- [x] **0.1** Renombrar el repo (`iman-macroquad` → `original-prototypes-ported`) y convertirlo en un
  workspace de Cargo con un crate por juego en `games/<slug>/`.
- [x] **0.2** Inventario y descarga de originales: `tools/originales.py` →
  `originales/catalogo.json` (metadatos, archivos, sha256). Los binarios quedan fuera de git.
- [x] **0.3** Repo público en GitHub.
- [x] **0.4** Crate compartido `crates/museo` sacado de IMAN: pantalla virtual escalada con bandas,
  paso fijo, detección táctil (`(pointer: coarse)` + primer toque) y controles táctiles
  **configurables por juego** (cruceta, 1–4 botones, zonas de toque). Migrar IMAN a usarlo.
- [x] **0.5** Plantilla web común: que `tools/build_web.sh` genere `index.html` (título, nombre del
  wasm, plugin táctil) en vez de copiar `games/<slug>/web/`.
- [x] **0.6** GitHub Actions: build nativo + wasm de todo el workspace en cada push.
- [ ] **0.7** (Opcional) GitHub Pages con todas las versiones web, enlazadas desde el README del museo.
- [x] **0.8** Herramientas de extracción por stack: UnityPy/AssetRipper (Unity 4/5 y Unity Web
  Player), lector de `data.js` de Construct 2, apktool/JADX (Android).
- [x] **0.9** Test de humo web reutilizable (Playwright: móvil horizontal/vertical y escritorio) en
  `tools/`, como el que se usó a mano con IMAN.

## Plantilla de checkpoints por juego

Cada juego de la fase 2 sigue estos pasos:

1. Triaje con REA: identidad del binario, stack y versión, fecha original, controles.
2. Extracción de assets con un script en `games/<slug>/tools/` (regenerable).
3. Reconstrucción de la lógica: C# decompilado, eventos de Construct o pseudocódigo, guardado en
   `games/<slug>/reference/`.
4. Port nativo jugable de principio a fin.
5. Versión web + controles táctiles, probada en móvil (horizontal y vertical) y escritorio con
   `tools/humo` (pasos del juego en `games/<slug>/humo.json`, también para el test nativo).
6. README del juego y su ficha en el README del museo.
7. Pendientes anotados.

## Fase 1 — Triaje

Hecho un primer triaje al descargar (2026-10-18). Falta confirmarlo con REA juego a juego (paso 1 de
cada juego en la fase 2).

| Juego | Original descargado | Stack detectado | Ruta de RE | Riesgos conocidos |
|---|---|---|---|---|
| Proyecto Nutria | `Proyecto_Nutria.rar` (1,5 MB) | XNA 3.1, XNB v4 | ✅ hecho | sin audio en el original; ver pendientes en 2.1 |
| IMAN | `IMAN_(Prototipo).rar` | XNA 3.1 | ✅ hecho | ver pendientes de IMAN |
| The Electric Eel Attack | `The_Electric_Eel_Attack.zip` (7 MB) | XNA 3.1 + `Engine.dll` propio | ✅ hecho | ver pendientes en 2.2 |
| Go Ball | 3 zips (Win32, Win64, macOS) | Unity 3.4.0f5, Mono, con `.mdb` | ILSpy de `Assembly-CSharp` y `Assembly-UnityScript`; escenas `level0..4` con `unity_assets.py` | 3D + física PhysX; UnityPy lee texturas, mallas, audio y campos de MonoBehaviour (los PPtr salen con los bytes invertidos) |
| The Meaning of Christmas | APK (7,8 MB) | Java + libGDX | REA Android + JADX; assets en `assets/data` | música en MP3 (macroquad no lo lee: convertir a OGG); el juego está en `com.badlogic.gdx.helloworld` (plantilla de libGDX) |
| Fast Car Racer | — | Android (`cl.gioco.fcr`) | — | **sin binario**: itch.io solo enlaza a Google Play |
| Super Happy Jump | `web.unity3d` (385 KB) | Unity 4.6.0f3, Web Player | UnityPy para abrir el bundle `UnityWeb` → `Assembly-CSharp.dll` + assets | formato UnityWeb comprimido con LZMA |
| Que No Caiga | HTML5 (39 archivos) | Construct 2 | ✅ hecho | ver pendientes en 2.3 |
| Retro Penal | HTML5 + NW.js (win/osx/linux) | Construct 2 | igual que Que No Caiga | ídem |
| One Button Prototype | Mac, Linux, APK y WebGL | Unity 5.1.2f1, Mono | ILSpy (Standard Assets 2D + clases propias); escenas con UnityPy/AssetRipper | física 2D (Box2D) |
| Neuro Battle | `Neuro Battle.zip` (WebGL) | Unity 5.1.1f1, IL2CPP → asm.js | **sin DLL de C#**: metadatos IL2CPP + escenas + observar el juego (REA browser) | el más difícil; el WebGL de 2015 puede no correr en navegadores actuales. El `.data` ya se desempaqueta (`unity_assets.py emscripten`): escenas, sprites y nombres de scripts legibles |
| Da Car | HTML5 (32 archivos) | Construct 2 | igual que Que No Caiga | ídem |

## Fase 2 — Ports

Orden sugerido: primero lo que reutiliza herramientas ya hechas y luego lo más difícil.

### 2.0 IMAN ✅
- [x] Triaje REA, assets, lógica, port nativo, web + táctil, README.
- [x] Migrar a `crates/museo` (checkpoint 0.4).
- Pendientes: mando DirectInput no portado; sonido sin probar en hardware real (el servidor no tiene
  tarjeta de sonido); controles táctiles probados solo en emulación de Chromium.

### 2.1 Proyecto Nutria (XNA 3.1) ✅
- [x] Triaje REA (`inspect-managed-artifact`, `inspect-managed-members`) + fecha de compilación
  (11-nov-2009 12:52 UTC, cabecera PE).
- [x] Assets: `tools/xnb_extract.py` (generalización del de IMAN) con la lista de contenido usado.
  No hay audio en el original.
- [x] Lógica: `ilspycmd -r <XNA 3.1 ref>` + `import-managed-reconstruction` en REA (104 métodos).
  El efecto de pausa se desensambló (ps_2_0) y se reescribió en GLSL.
- [x] Port nativo jugable.
- [x] Web + táctil.
- [x] README del juego + ficha del museo.
- [x] Pendientes anotados: mando DirectInput no portado (el pollo solo era jugable con mando);
  controles táctiles probados solo en emulación de Chromium.

### 2.2 The Electric Eel Attack (XNA 3.1 + Engine.dll) ✅
- [x] Triaje REA de `The Electric Eel Attack.exe` y `Engine.dll` (9 y 8-mar-2011, cabeceras PE).
- [x] Assets: XNB (texturas, fuentes, 17 `SoundEffect` PCM → WAV) y los mapas XML del editor del
  motor. Los efectos se desensamblaron y los tres que se ven se reescribieron en GLSL.
- [x] Lógica: decompilar los dos ensamblados (33 + 129 métodos importados en REA).
- [x] Port nativo jugable.
- [x] Web + táctil.
- [x] README del juego + ficha del museo.
- [x] Pendientes anotados: la música se carga al llegar a su nivel (pausa breve en web); el
  sonido no se probó en hardware real (el servidor no tiene tarjeta de sonido); los controles
  táctiles se probaron solo en emulación de Chromium. Los niveles se probaron hasta la cadena
  1 → 4 → título con ayudas temporales, pero ninguno se jugó de principio a fin en el port.

### 2.3 Que No Caiga (Construct 2) ✅
- [x] Triaje: Construct 2 de 2015 (r20x, sin número en la exportación), plugins Sprite, Text,
  Touch, Audio, Browser (y Button, TextBox y Mouse sin uso) y el comportamiento Platform (REA
  `analyze-javascript-application` sobre `originales/que-no-caiga/html5`).
- [x] Herramienta `tools/c2_eventos.py` (checkpoint 0.8) y `games/que-no-caiga/reference/nombres.json`
  con los nombres de las condiciones y acciones minificadas.
- [x] Assets: `images/*.png` y `media/*.ogg` tal cual, y `data.js` se lee al arrancar.
- [x] Port nativo jugable: las dos hojas de eventos y lo que usan del runtime (Platform sin
  sólidos, animaciones de Sprite, polígonos de colisión, Text, Audio con etiquetas, Touch con el
  ratón).
- [x] Web + táctil (se juega tocando la pantalla, sin botones).
- [x] README del juego + ficha del museo.
- [x] Pendientes anotados: las fuentes son Liberation Sans en lugar de Arial y Arial Black; el
  sonido no se probó en hardware real; no se jugó hasta el segundo 50 sin ayuda (la bomba se probó
  con un cambio temporal).

### 2.4 Retro Penal (Construct 2)
- [ ] Triaje (comparar `package.nw` con la versión HTML5).
- [ ] Eventos con `tools/c2_eventos.py`.
- [ ] Port nativo jugable.
- [ ] Web + táctil.
- [ ] README del juego + ficha del museo.
- [ ] Pendientes anotados.

### 2.5 Da Car (Construct 2)
- [ ] Triaje.
- [ ] Eventos con `tools/c2_eventos.py`.
- [ ] Port nativo jugable (física de la moto).
- [ ] Web + táctil.
- [ ] README del juego + ficha del museo.
- [ ] Pendientes anotados.

### 2.6 Super Happy Jump (Unity 4.6 Web Player)
- [ ] Abrir `web.unity3d` con UnityPy: `Assembly-CSharp.dll`, escena, sprites y audio.
- [ ] Triaje REA del ensamblado + ILSpy (referencias de UnityEngine 4.6).
- [ ] Port nativo jugable (toque = salto, doble toque = doble salto: encaja con móvil).
- [ ] Web + táctil.
- [ ] README del juego + ficha del museo.
- [ ] Pendientes anotados.

### 2.7 One Button Prototype (Unity 5.1, Mono)
- [ ] Triaje REA de `Assembly-CSharp.dll` y `-firstpass` (ya identificado: Standard Assets 2D +
  `GUI_Prototipo`, `PlatformerDragon2D`, `Platformer2DDragonControl`).
- [ ] Escenas y sprites con UnityPy/AssetRipper (posiciones, colisionadores, capas).
- [ ] Física 2D: decidir entre reimplementar lo necesario o usar un motor (p. ej. rapier2d).
- [ ] Port nativo jugable.
- [ ] Web + táctil (un solo botón).
- [ ] README del juego + ficha del museo.
- [ ] Pendientes anotados.

### 2.8 The Meaning of Christmas (libGDX)
- [ ] Configurar JADX para REA (`REA_JADX_MCP_JAR`) o usar JADX directamente; triaje del APK.
- [ ] Decompilar `classes.dex` a Java (libGDX se traduce casi 1:1 a macroquad).
- [ ] Assets de `assets/data`; MP3 → OGG.
- [ ] Port nativo jugable.
- [ ] Web + táctil.
- [ ] README del juego + ficha del museo.
- [ ] Pendientes anotados.

### 2.9 Go Ball (Unity 3.4)
- [ ] Triaje REA + ILSpy de `Assembly-CSharp.dll` y `Assembly-UnityScript.dll` (con sus `.mdb`).
- [ ] Escenas `level0..4`: comprobar qué herramienta lee Unity 3.4; extraer mallas y texturas.
- [ ] Decidir el enfoque si es 3D (cámara y mallas de macroquad) y la física.
- [ ] Port nativo jugable.
- [ ] Web + táctil.
- [ ] README del juego + ficha del museo.
- [ ] Pendientes anotados.

### 2.10 Neuro Battle (Unity 5.1 WebGL / IL2CPP)
- [ ] Desempaquetar `Neuro Battle.data` (paquete de archivos de Emscripten) y buscar los metadatos
  IL2CPP (`global-metadata.dat`).
- [ ] Recuperar tipos, métodos y textos (Il2CppDumper o similar) y extraer escenas y assets.
- [ ] Probar si el WebGL original corre en un navegador actual; si corre, observarlo con REA
  (`capture-browser-scenario`) para cubrir la lógica que no se pueda leer.
- [ ] Port nativo jugable.
- [ ] Web + táctil.
- [ ] README del juego + ficha del museo.
- [ ] Pendientes anotados.

### 2.11 Fast Car Racer (bloqueado)
- [ ] Conseguir el binario: el APK/AAB subido a Google Play (se puede bajar desde Play Console →
  *App bundle explorer*) o una copia de los archivos del proyecto. Sin binario no hay RE fiel.
- [ ] Si no aparece: decidir si se recrea a partir del video de gameplay (y marcarlo como
  reconstrucción, no port).
- [ ] Resto de pasos de la plantilla.

## Fase 3 — Museo

- [ ] Capturas de cada port en `games/<slug>/docs/` y en el README del museo.
- [ ] Versiones web jugables enlazadas desde el README (si se hace el checkpoint 0.7).
- [ ] Revisar que cada ficha tenga fecha confirmada, stack, descripción, port y original.
