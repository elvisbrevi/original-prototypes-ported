# Plan del museo: portar los prototipos originales a macroquad

Plan de trabajo para recrear en Rust + macroquad todos los juegos publicados en
[elvisbrevi.itch.io](https://elvisbrevi.itch.io) (menos IMÁN 2026, que solo fue una prueba de publicación).
Está pensado para avanzar en varias sesiones: marca cada casilla al completarla e incluye el cambio
del plan en el mismo commit.

**Siguiente paso:** 2.9 Go Ball (Unity 3.4): triaje, extracción y enfoque hechos (ver 2.9).
Port nativo y web hechos; falta el README del juego con su ficha del museo y los
pendientes.
Reutilizar lo que sirva de `crates/unity`. Seguir la plantilla con el paso 6 ampliado. La fase 4 (publicar las
versiones web en itch.io) espera la confirmación del dueño de la cuenta: 4.1 y 4.2 se pueden
preparar en cuanto la dé. El 0.7 (GitHub Pages) queda pendiente de decidir si se publica.

## Reglas

1. **Fidelidad.** La lógica se traduce del original (código decompilado o datos del motor), no se
   reinventa. Las rarezas del original se conservan y se documentan.
2. **Assets originales**, extraídos de los binarios. Si uno no se puede extraer, se usa un sustituto
   provisional y se anota como pendiente.
3. **Cada port tiene versión nativa (PC) y web (wasm)**, con controles táctiles que aparecen solos en
   móviles y tablets.
4. **itch.io, solo con confirmación.** Los ports viven en este repo y su versión web se publica en
   la página de itch.io de cada juego (fase 4), pero cada subida se confirma antes con el dueño de
   la cuenta.
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

# Android: JADX 1.5.6 y apktool 3.0.3 (Java 21) en ~/.local/opt; REA busca `apktool` en el PATH
sudo apt install openjdk-21-jre-headless
curl -LO https://github.com/skylot/jadx/releases/download/v1.5.6/jadx-1.5.6.zip   # unzip -d ~/.local/opt/jadx
curl -L -o ~/.local/opt/apktool/apktool_3.0.3.jar \
  https://github.com/iBotPeaches/Apktool/releases/download/v3.0.3/apktool_3.0.3.jar
printf '#!/bin/sh\nexec java -jar "$HOME/.local/opt/apktool/apktool_3.0.3.jar" "$@"\n' > ~/.local/bin/apktool
chmod +x ~/.local/bin/apktool

# Audio: ffmpeg para pasar MP3 a Ogg Vorbis (macroquad no lee MP3)
sudo apt install ffmpeg

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
  campos de los MonoBehaviour (en los builds de Unity 5 sin type trees, con el MonoScript
  resuelto a mano). AssetRipper no hizo falta. Para un port, el `tools/extraer.py` del juego
  escribe el `escena.json` que lee `crates/unity`: el de Super Happy Jump es el modelo para un
  bundle de Unity 4 y el de One Button Prototype para un build de Unity 5 con varias escenas.
- Construct 2: `python3 -I tools/c2_eventos.py resumen|eventos|fuente <carpeta html5>` (proyecto,
  tipos, layouts, tabla de referencias del runtime minificado y hojas de eventos en pseudocódigo).
- Android: `~/.local/opt/jadx/bin/jadx -d <salida> <apk>` y
  `java -jar ~/.local/opt/apktool/apktool_3.0.3.jar d -o <salida> <apk>`. REA usa apktool
  (`decode-android-resources`) con el lanzador `~/.local/bin/apktool`; sus herramientas de JADX
  piden un JAR de JADX MCP en `REA_JADX_MCP_JAR`, que no está. Para un port, el
  `tools/extraer.py` de The Meaning of Christmas lee el APK como zip y pasa el MP3 a Ogg.

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
6. README del juego y su ficha en el README del museo. Cuentan qué juego es, con qué y cuándo se
   hizo y que ahora está portado, sin separar «versión original» y «port». En «Cómo se hizo» el
   README documenta el proceso técnico del port, al terminarlo: triaje y Evidence de REA,
   extracción, qué se tradujo y cómo se resolvió cada pieza del motor (lo que no se ve leyendo el
   código), y cómo se comprobó.
7. Pendientes anotados.
8. Publicar la versión web en la página de itch.io del juego (fase 4), con confirmación.

## Fase 1 — Triaje

Hecho un primer triaje al descargar (2026-10-18). Falta confirmarlo con REA juego a juego (paso 1 de
cada juego en la fase 2).

| Juego | Original descargado | Stack detectado | Ruta de RE | Riesgos conocidos |
|---|---|---|---|---|
| Proyecto Nutria | `Proyecto_Nutria.rar` (1,5 MB) | XNA 3.1, XNB v4 | ✅ hecho | sin audio en el original; ver pendientes en 2.1 |
| IMAN | `IMAN_(Prototipo).rar` | XNA 3.1 | ✅ hecho | ver pendientes de IMAN |
| The Electric Eel Attack | `The_Electric_Eel_Attack.zip` (7 MB) | XNA 3.1 + `Engine.dll` propio | ✅ hecho | ver pendientes en 2.2 |
| Go Ball | 3 zips (Win32, Win64, macOS) | Unity 3.4.0f5, Mono, con `.mdb` | triaje y extracción hechos (2.9, `games/go-ball/tools/extraer.py`) | 3D + física PhysX (una esfera contra `MeshCollider`); UnityPy lee texturas, mallas, audio y campos de MonoBehaviour (los PPtr salen con los bytes invertidos) |
| The Meaning of Christmas | APK (7,8 MB) | Java + libGDX 0.9.2 | ✅ hecho | ver pendientes en 2.8 |
| Fast Car Racer | — | Android (`cl.gioco.fcr`) | — | **sin binario**: itch.io solo enlaza a Google Play |
| Super Happy Jump | `web.unity3d` (385 KB) | Unity 4.6.0f3, Web Player | ✅ hecho | ver pendientes en 2.6 |
| Que No Caiga | HTML5 (39 archivos) | Construct 2 | ✅ hecho | ver pendientes en 2.3 |
| Retro Penal | HTML5 + NW.js (win/osx/linux) | Construct 2 | ✅ hecho | ver pendientes en 2.4 |
| One Button Prototype | Mac, Linux, APK y WebGL | Unity 5.1.2f1, Mono | ✅ hecho | ver pendientes en 2.7 |
| Neuro Battle | `Neuro Battle.zip` (WebGL) | Unity 5.1.1f1, IL2CPP → asm.js | **sin DLL de C#**: metadatos IL2CPP + escenas + observar el juego (REA browser) | el más difícil; el WebGL de 2015 puede no correr en navegadores actuales. El `.data` ya se desempaqueta (`unity_assets.py emscripten`): escenas, sprites y nombres de scripts legibles |
| Da Car | HTML5 (32 archivos) | Construct 2 + Box2D (asm.js) | ✅ hecho | ver pendientes en 2.5 |

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

### 2.4 Retro Penal (Construct 2) ✅
- [x] Triaje (comparar `package.nw` con la versión HTML5): REA `analyze-javascript-application`
  sobre las dos. Los paquetes NW.js (22-jul-2015) son una exportación del día anterior a la HTML5
  (23-jul-2015): mismas imágenes y OGG, pero todavía con el plugin Browser y el botón «Compartir»,
  cuyo evento sin condiciones es el clic en cada tick del final de la HTML5.
- [x] Eventos con `tools/c2_eventos.py` y `games/retro-penal/reference/nombres.json`.
- [x] Runtime compartido `crates/construct2`, sacado de Que No Caiga (que pasa a usarlo, con las
  mismas capturas de humo): Bullet con rebote en sólidos, Turret, Custom Movement, Solid y Wrap,
  colisión con giro y espejo, On collision, Wait, bloques O y Sino, Audio con bucle, Stop, volumen
  y On ended, Mouse.SetCursor.
- [x] Port nativo jugable: las tres hojas de eventos.
- [x] Web + táctil (se juega tocando la pantalla, sin botones).
- [x] README del juego + ficha del museo.
- [x] Pendientes anotados: las fuentes son Liberation Sans en lugar de Arial; el sonido no se
  probó en hardware real; Stop y SetVolume actúan sobre todo el archivo (aquí no cambia nada); la
  ventana no escala por múltiplos enteros como el original; de Turret no se portaron la puntería
  predictiva ni los disparadores de disparo (no se usan).

### 2.5 Da Car (Construct 2) ✅
- [x] Triaje: exportado el 23-nov-2015; plugins Sprite, Text, Particles, Audio, Touch, Mouse y
  Keyboard, y los comportamientos Physics, Wrap y ScrollTo. REA `analyze-javascript-application`
  pasó del límite de 5 min con la carpeta entera (Box2D en asm.js dentro de `c2runtime.js`): se
  repitió sin el runtime y el runtime se leyó a mano.
- [x] Eventos con `tools/c2_eventos.py` y `games/da-car/reference/nombres.json`.
- [x] Box2D 2.3.1 traducido a Rust en `crates/box2d` (el `box2d.js` de kripken que trae el
  runtime, con `b2_maxPolygonVertices` = 16), idéntico bit a bit al C++ en cuatro escenas
  (`crates/box2d/comparar/comparar.sh`).
- [x] Port nativo jugable: el comportamiento Physics (formas, separador de polígonos cóncavos,
  articulaciones, colisiones registradas), ScrollTo, Wrap por la vista, capas con parallax,
  Particles y la hoja «InGame».
- [x] Web + táctil: se acelera tocando y se gira inclinando el dispositivo (`museo::sensores::beta`,
  desde `deviceorientation`; en iOS se pide permiso al primer toque).
- [x] README del juego + ficha del museo.
- [x] Pendientes anotados: las fuentes son Liberation Sans en lugar de Arial; el sonido no se
  probó en hardware real; la inclinación no se probó en un móvil de verdad (solo la orientación
  emulada por CDP en Chromium, que llega a la página, y una inclinación fija forzada en nativo); no
  se jugó hasta la meta sin ayuda (la meta se probó con un cambio temporal).

### 2.6 Super Happy Jump (Unity 4.6 Web Player) ✅
- [x] Abrir `web.unity3d` con UnityPy: `Assembly-CSharp.dll` (compilado el 9-dic-2014, cabecera
  PE), la escena, los prefabs, sprites, clips de Mecanim, partículas, la `GUISkin` y el audio, con
  `games/super-happy-jump/tools/extraer.py` → `assets/escena.json`.
- [x] Triaje REA del ensamblado (`inspect-managed-artifact`, `inspect-managed-members`, 132 métodos
  importados con `import-managed-reconstruction`; REA no abre el formato `UnityWeb`) + ILSpy, con
  el `UnityEngine.dll` de Unity 5.1.2 de One Button Prototype (el de 4.6 no viene en el bundle).
- [x] Crate compartido `crates/unity`: Time, GameObjects, física 2D sobre `crates/box2d` (que ganó
  sensores, eventos por fixture, `QueryAABB`, `DestroyFixture`, `ApplyForceToCenter` y
  `SetMassData`, comparados bit a bit con el C++ en la escena «corredor»), Animator, partículas
  (Shuriken), cámara y orden de dibujo, IMGUI y AudioSource. `museo::Preferencias` para los
  `PlayerPrefs` (archivo en PC, `localStorage` en la web).
- [x] Port nativo jugable: los scripts del juego y el frame de Unity en `src/juego.rs`.
- [x] Web + táctil (se juega tocando la pantalla, sin botones).
- [x] README del juego + ficha del museo.
- [x] Pendientes anotados: las fuentes son Liberation Sans en lugar de Arial y Nunito en lugar de
  Arial Rounded MT Bold (incrustada en el original, de Monotype, no redistribuible); el Box2D del
  Web Player es de versión desconocida (se usa 2.3.1); lo que Unity hace por dentro (despertar al
  cuerpo movido por código, estados de estilo sin fondo, sprites en las transiciones de Mecanim,
  partículas estiradas) se dedujo y no se pudo comparar con el original en marcha, porque el
  Unity Web Player ya no corre; el sonido no se probó en hardware real; los controles táctiles se
  probaron solo en emulación de Chromium; las monedas y el récord se probaron con un cambio
  temporal; las versiones de iOS y Android no están en itch.io (solo enlaces a las tiendas).

### 2.7 One Button Prototype (Unity 5.1, Mono) ✅
- [x] Triaje REA de `Assembly-CSharp.dll` y `-firstpass` (`inspect-managed-artifact` e
  `inspect-managed-members` de los seis tipos del juego y de `FPSCounter`): compilados el
  22-jul-2015 (cabecera PE). Los builds de escritorio y Android son de ese día; el WebGL del
  17-ago-2015 tiene las mismas escenas (comparadas con UnityPy) y los mismos literales en IL2CPP.
- [x] Escenas y sprites con UnityPy (sin type trees: TypeTreeGeneratorAPI para los MonoBehaviour)
  en `games/one-button-prototype/tools/extraer.py`: las tres escenas, sprites de malla ajustada
  con su `uvTransform`, texturas recortadas, clips (también las curvas constantes del
  `BoxCollider2D`), controladores con blend tree, la `GameSkin` integrada y OpenSans.
- [x] `crates/unity` ampliado para Unity 5.1: varias escenas, `PolygonCollider2D` (piezas
  convexas), contactos por par de colisionadores con `OnCollisionEnter2D`, `OverlapCircleAll`,
  `bounds`, interpolación, Animator con blend trees 1D, *AnimatePhysics*, curvas sobre el
  colisionador con *Write Defaults*, `GUI.Window` con título y foco, `GUIText`, `SmoothDamp`.
  `crates/box2d` ganó el `Set` con soldadura de Box2D 2.3.1, comparado bit a bit con un C++
  compilado como el de Unity en la escena nueva «rebote».
- [x] Port nativo jugable.
- [x] Web + táctil (un solo botón: el toque).
- [x] README del juego + ficha del museo.
- [x] Pendientes anotados: `import-managed-reconstruction` no se hizo (REA pide la Evidence de
  miembros entera en la llamada, unos 250 KB; el payload se regenera con
  `tools/rea_reconstruction.py`); las fuentes de la GUI son Liberation Sans en lugar de Arial; lo
  que Unity hace por dentro (interpolación, *AnimatePhysics*, foco de las ventanas, `bounds`, la
  curva de `m_Center` sin aplicar, las piezas de los polígonos) se dedujo y no se pudo comparar
  con el original en marcha; los niveles no se jugaron de principio a fin sin ayuda (la estrella
  se probó con un cambio temporal) y no se probó pisar un dragón; los controles táctiles se
  probaron solo en emulación de Chromium.

### 2.8 The Meaning of Christmas (libGDX) ✅
- [x] Triaje: REA `decode-android-resources` (con apktool en el PATH) para el manifiesto y los
  textos; JADX directamente para el código (las herramientas de REA que lo recuperan piden un JAR
  de JADX MCP). libGDX 0.9.2 sobre su «hello world», APK armado el 1-ene-2012 y firmado con el
  certificado de depuración.
- [x] `classes.dex` decompilado con JADX 1.5.6: las siete clases del juego en `reference/java`, y
  lo que usa de libGDX (`BitmapFont`, `Animation`, `WindowedMean`, `AndroidGraphics`) leído del
  mismo APK.
- [x] Assets de `assets/data` y la fuente `arial-15` de libGDX con
  `games/the-meaning-of-christmas/tools/extraer.py`; MP3 → Ogg Vorbis con ffmpeg.
- [x] Port nativo jugable (320×480, en vertical).
- [x] Web + táctil (se juega tocando).
- [x] README del juego + ficha del museo.
- [x] Pendientes anotados: el sonido no se probó en hardware real; en la web el título queda en
  silencio hasta el primer toque (que ya empieza la partida); los controles táctiles se probaron
  solo en emulación de Chromium; la quema de regalos y el final de la partida se probaron con
  cambios temporales (un jugador automático y el dinero cerca del tope); faltan las herramientas
  JADX de REA (`REA_JADX_MCP_JAR`).

### 2.9 Go Ball (Unity 3.4)
- [x] Triaje REA + ILSpy de `Assembly-CSharp.dll` y `Assembly-UnityScript.dll` (con sus `.mdb`),
  del build de Windows de 32 bits extraído en `originales/go-ball/win32` (sin el `.exe` ni el
  `.pdb`):
  - Compilados el 22-sep-2011 a las 04:15 UTC (cabecera PE), Unity 3.4.0f5. REA
    `inspect-managed-artifact` de `Assembly-CSharp.dll`
    (`ev_f3b10410c604e1409754d98b8d98c4beb49b09d7eb0ab19ef296839fedf7eb78`) e
    `inspect-managed-members` de los seis tipos del juego: 35 métodos y 30 literales
    (`ev_8022ac3d81b5715672524d9edfad7041f0da03cf1ef2468f9c8b827d41080d8f`).
  - Seis escenas en `BuildSettings`: `menuPrincipal` (en `mainData`, con un terreno con árboles
    y la bola de adorno) y `nivel1` a `nivel5` (`level0` a `level4`). `PlayerSettings`: «Go
    Ball», 1024×600 (640×480 en web). Sin lightmaps.
  - Scripts (unas 600 líneas propias): `Bola` (la bola, un `Rigidbody` con `SphereCollider`:
    `AddForce` horizontal en modo `Acceleration` con el eje «Horizontal» y salto con «Saltar»
    fijando la velocidad vertical; muere con la «Sierra», con `OnParticleCollision` o al salir
    de `limiteMin`/`limiteMax`; «Explosivo1» la empuja), `MoverObjeto` (objetos que se arrastran
    con el ratón en horizontal o vertical, de 0,05 en 0,05, o que van y vienen solos; la rama
    de rotación compara la y con los límites horizontales), `Objetivo` (la meta, que gira con
    iTween `RotateBy` en *ping-pong*), `Camara` (sigue a la bola; botones de salir, maximizar,
    opciones, que no hace nada, y reiniciar, con ventanas que pausan con `timeScale`),
    `Transicion` (fundidos con un `GUITexture` y la carga de niveles), `EmpezarJuego` (el menú) y
    `TimedObjectDestructor` (UnityScript). Además, `iTween` (4640 líneas, de terceros) y
    `WaterSimple` (Standard Assets) en `-firstpass`.
  - Lo que pide del motor: render 3D con los shaders integrados de Unity 3.4 y sus luces, un
    terreno con árboles (solo en el menú), el agua de los Standard Assets, animaciones heredadas
    (`Animation`, 29), partículas heredadas (`EllipsoidParticleEmitter`, `ParticleAnimator`,
    `ParticleRenderer` y `WorldParticleCollider`), `GUITexture`, IMGUI con `GUISkin`,
    `OnMouseDown`/`OnMouseDrag` (rayos contra los colisionadores), `AudioSource` y PhysX 2.8:
    la única cosa dinámica es la bola (y unos sistemas de partículas «OuterCore» con
    `Rigidbody` en `nivel2` y `nivel3`), contra 148 `MeshCollider` que el código y las
    animaciones mueven. `PhysicsManager`: *bounce threshold* 2, *sleep velocity* 0,15, velocidad
    angular máxima 7 (limita lo que rueda la bola) y 6 iteraciones.
  - Propuesta de enfoque, a decidir: el render 3D con macroquad y shaders GLSL que imiten los
    integrados que usen los materiales, y para la física, un solver propio de una esfera contra
    triángulos (con el roce, el rebote y el límite de velocidad angular de PhysX documentados)
    en vez de un motor 3D completo, porque PhysX no se puede traducir.
- [x] Escenas y assets con UnityPy en `games/go-ball/tools/extraer.py` → `assets/`:
  - UnityPy lee Unity 3.4: las clases del motor traen type trees; los MonoBehaviour comparten
    uno sin los campos del script, que se generan con TypeTreeGeneratorAPI desde `Managed/`. La
    `GUISkin` y el `Terrain` son MonoBehaviour de `UnityEngine.dll` con el formato nativo de
    3.4 y se leen a mano (20 estilos fijos y 7 propios; los campos del terreno).
  - `escena.json`: las seis escenas (8, 36, 36, 62, 64 y 66 objetos) con su jerarquía, los
    componentes 3D (MeshRenderer, MeshCollider, BoxCollider, SphereCollider, Rigidbody, Light,
    Camera, AudioSource, GUITexture, las partículas heredadas y los campos de los scripts), los
    `RenderSettings` de cada escena, los 44 materiales con su shader, la piel (los estilos que
    usan los scripts), tres fuentes de mapa de bits (solo ASCII y Latin-1), el destello «85mm
    Lens», el terreno del menú (alturas en 8 bits, mezcla y los 45 parches de pasto; sin
    árboles) y los ajustes (tiempo, física, calidad «Fantastic», ejes del InputManager).
  - 25 mallas en `mallas/*.bin` (Unity 3.4 guarda las submallas como tiras: se separan en
    triángulos, con los de colisión aparte), 58 texturas (DXT1/DXT5 decodificadas, Alpha8 de
    las fuentes a blanco con alfa) y las seis caras de los dos cubemaps del diamante, y los seis
    clips (OGG y un WAV) con su duración. `src/recursos.rs` los embebe.
  - Las 29 `Animation` están vacías (sin clips: las dejan los modelos importados), así que no
    hay animaciones que portar; lo que se mueve lo mueven los scripts e iTween.
  - Las capturas de la página de itch.io (1440×900, «Development Build») sirven de referencia
    visual; no van al repo.
- [x] Enfoque decidido y probado con los cinco niveles: el de la propuesta, en `crates/unity`.
  - Render 3D propio con miniquad directamente (`render3d.rs`): búferes estáticos por malla,
    texturas con mipmaps (Unity guarda las filas de abajo hacia arriba), cubemaps, la vista de
    Unity (z hacia adelante, invertida como su `worldToCameraMatrix`), las colas 2000, 2450,
    2901 y 3000, partículas como billboards y el destello de la luz en la pantalla.
  - Los shaders de Unity 3.4 reescritos en GLSL desde su código ARB del build
    (`shaders3d.rs`): Diffuse y Bumped Specular (ambiente + luz puntual por píxel con la
    atenuación de Unity), Tree Soft Occlusion (luz por vértice y oclusión en las tangentes),
    Cutout Soft Edge Unlit (dos pasadas), la nube (función fija), FX/Diamond (dos pasadas con
    cubemaps), el skybox y Particles/Alpha Blended.
  - miniquad 0.4 apaga el depth test cuando un pipeline no escribe profundidad: los
    transparentes se dibujan con `glDepthMask(false)`, que en la web agrega el `index.html` de
    `crates/museo` (el `mq_js_bundle.js` no lo trae).
  - Física propia en lugar de PhysX (`fisica3d.rs`): una esfera contra triángulos de
    MeshCollider, cajas y esferas, con impulsos secuenciales (6 iteraciones), roce 0,6 del
    material por defecto, el «skin» de 0,01, la velocidad angular máxima de 7 (la bola no rueda
    a más de 1,68 m/s), las restricciones y las fuerzas de `AddForce` que esperan al próximo
    paso (desde `Update`, la aceleración depende de los fps, como en el original); rayos para
    `OnMouseDown` y para las partículas.
  - Partículas heredadas (`particulas3d.rs`), InputManager con el suavizado de los ejes
    (`entrada.rs`), AudioSource con atenuación por distancia (`audio3d.rs`) y la GUI de Unity 3
    (fuentes de mapa de bits, `GUI.color`, `DrawTexture`, tooltips, estados sin fondo) en
    `gui.rs`, decompilando de `UnityEngine.dll` 3.4 `SendMouseEvents`, `GUI` y `GUIUtility`.
  - El juego (`games/go-ball/src`): los siete scripts, iTween `RotateBy` y el frame de Unity
    3.4. Los niveles se ven como las capturas de itch.io y se juegan (arrastrar cajas, saltar,
    la meta, el fuego). Falta el terreno del menú.
- [x] Port nativo jugable: el menú y los cinco niveles, de principio a fin (la meta del nivel 5
  vuelve al menú).
  - El terreno del menú (`crates/unity/src/terreno.rs`): la malla del mapa de alturas en la zona
    que ve la cámara (filas en z, alturas sobre 32766), las dos capas mezcladas con el mapa de
    control y unas 3400 matas de pasto en cuatro capas, como billboards que se balancean con el
    viento (el programa de vértices de BillboardWavingDoublePass traducido). El reparto al azar de
    las matas dentro de cada muestra no es el de Unity.
  - El destello calibrado con las capturas de itch.io: la caída del rojo en la fila de la luz da
    un resplandor (tamaño 50) de 1,25 altos de pantalla, o sea 2,5 % del alto por unidad de
    tamaño; con eso coincide punto a punto en el nivel 1. Las sierras lo tapan y lo apagan, como
    el rayo de oclusión de Unity contra sus MeshCollider.
  - `src/recursos.rs` con `static` (con `const` el wasm repetía las texturas: 24 MB en vez de
    14).
  - Probado en Xvfb: el menú, «Jugar», arrastrar las cajas, rodar y saltar, la ventana de salir
    (pausa y oscurece) y la vuelta al menú; con un cambio temporal (la bola puesta junto a la
    meta o al peligro): la meta y el paso al nivel siguiente, la muerte en el fuego y en las
    sierras con el reinicio, y el nivel 5 que vuelve al menú.
- [x] Web + táctil: una cruceta horizontal y el botón «A» para saltar (las flechas y la de
  arriba); fuera de los controles, el dedo hace de ratón (los botones de la GUI y arrastrar las
  cajas; el primer dedo que empieza fuera de los controles). `humo.json`: el menú, «Jugar», rodar
  y saltar, la ventana de salir y la vuelta al menú, en nativo y en la web (escritorio, móvil
  horizontal y vertical). Arrastrar una caja con el dedo se probó aparte con Playwright (toques
  de CDP en móvil horizontal). El wasm pesa 14 MB.
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
- [ ] Revisar que cada ficha tenga fecha confirmada, stack, descripción, estado del port y enlace a
  itch.io.
- [x] README de los juegos y fichas del museo con un mismo enfoque: qué juego es, con qué y cuándo
  se hizo y que ahora está portado, sin separar «original» y «port». «Cómo se hizo» ampliado con
  los detalles técnicos de cada port: IMAN, Proyecto Nutria, The Electric Eel Attack, Que No Caiga,
  Retro Penal, Da Car, Super Happy Jump, One Button Prototype y The Meaning of Christmas (paso 6
  de la plantilla para los siguientes).

## Fase 4 — Publicación en itch.io

Cada página de itch.io pasa a ofrecer el port para jugar en el navegador. Los juegos que ya tenían
versión web (Que No Caiga, Retro Penal y Da Car en HTML5; One Button Prototype y Neuro Battle en
WebGL; Super Happy Jump en el Unity Web Player, que ya no corre) la reemplazan por la del port, y
los demás la estrenan. Nada se sube sin la confirmación del dueño de la cuenta (regla 4).

- [ ] **4.1** Preparar la publicación: butler (la herramienta de itch.io) con una API key que no
  entra al repo, y `tools/publicar_itch.sh <slug>`, que compila la web con `tools/build_web.sh` y
  la sube con `butler push dist/<slug> elvisbrevi/<proyecto>:html5`. Hace falta la tabla de slugs
  del repo y proyectos de itch.io: casi todos coinciden, pero One Button Prototype es
  `prototipo-2000`. La prueba de IMÁN 2026 (`elvisbrevi/imn-2026:html5`) ya probó el flujo con
  IMAN.
- [ ] **4.2** Decidir con el dueño qué pasa con lo que ya hay en cada página. Las descargas
  originales (rar, zip, apk), el HTML5 de Construct 2 y los WebGL de Unity: ¿se reemplazan, quedan
  como descarga o se ocultan? Y el texto de la página: decir que lo que se juega es el port y
  enlazar a este repo.
- [ ] **4.3** Configurar cada página para jugar en el navegador, desde el panel de itch.io (butler
  no lo hace): tipo HTML, marco del tamaño del juego (640×480 IMAN y The Electric Eel Attack,
  1280×720 Proyecto Nutria, 400×500 Que No Caiga y Retro Penal, 800×450 Da Car, 960×600 Super
  Happy Jump y One Button Prototype, 320×480 en vertical The Meaning of Christmas), «Mobile
  friendly» con su orientación y botón de pantalla completa.
- [ ] **4.4** Publicar uno a uno, con confirmación, los ports ya hechos: Proyecto Nutria, IMAN, The
  Electric Eel Attack, The Meaning of Christmas, Que No Caiga, Retro Penal, Da Car, Super Happy
  Jump y One Button Prototype. Probar cada página
  publicada en escritorio y en móvil (`tools/humo/web.mjs` adaptado a una URL, o a mano).
- [ ] **4.5** Desde entonces, cada port nuevo se publica al cerrar su checkpoint (paso 8 de la
  plantilla), también con confirmación.
