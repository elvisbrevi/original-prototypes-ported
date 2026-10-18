# Original Prototypes Ported — museo de prototipos

Los juegos y prototipos que publiqué en [itch.io](https://elvisbrevi.itch.io) entre 2009 y 2015,
recreados en **Rust + [macroquad](https://macroquad.rs)** para jugarse hoy en PC y en el navegador
(con controles táctiles en móvil). Cada port se reconstruye por ingeniería inversa del binario
original con [REA](https://rea.tools/) y usa los gráficos y sonidos originales extraídos de él.

El avance está en [PLAN.md](PLAN.md). Cada push compila todo en nativo y en wasm
([GitHub Actions](.github/workflows/build.yml)):
![build](https://github.com/elvisbrevi/original-prototypes-ported/actions/workflows/build.yml/badge.svg)

## Sala de exposición

| Año | Juego | Stack original | Port |
|---|---|---|---|
| 2009 | [Proyecto Nutria](#proyecto-nutria) | C# + XNA 3.1 (Windows) | ✅ [games/proyecto-nutria](games/proyecto-nutria) |
| 2009 | [IMAN](#iman) | C# + XNA 3.1 (Windows) | ✅ [games/iman](games/iman) |
| 2011 | [The Electric Eel Attack](#the-electric-eel-attack) | C# + XNA 3.1 + motor propio (Windows) | ✅ [games/the-electric-eel-attack](games/the-electric-eel-attack) |
| 2011 | [Go Ball](#go-ball) | Unity 3.4 (C# + UnityScript) | pendiente |
| 2011 | [The Meaning of Christmas](#the-meaning-of-christmas) | Java + libGDX (Android) | pendiente |
| 2013 | [Fast Car Racer](#fast-car-racer) | Android (Google Play) | bloqueado: falta el binario |
| 2014 | [Super Happy Jump](#super-happy-jump) | Unity 4.6 (Web Player, iOS, Android) | pendiente |
| 2014–15 | [Que No Caiga](#que-no-caiga) | Construct 2 (HTML5) | ✅ [games/que-no-caiga](games/que-no-caiga) |
| 2014–15 | [Retro Penal](#retro-penal) | Construct 2 (HTML5 + NW.js) | pendiente |
| 2015 | [One Button Prototype](#one-button-prototype) | Unity 5.1 (Mac, Linux, Android, WebGL) | pendiente |
| 2015 | [Neuro Battle](#neuro-battle) | Unity 5.1 (WebGL) | pendiente |
| 2015 | [Da Car](#da-car) | Construct 2 (HTML5) | pendiente |

Las fechas salen de los propios binarios (fecha de compilación, de los archivos o de la exportación)
y de los videos de gameplay; los datos que todavía son deducciones están marcados como «por
confirmar» y se cierran en el triaje de cada juego.

---

### Proyecto Nutria

![Proyecto Nutria](games/proyecto-nutria/docs/nivel1.png)

- **Fecha:** compilado el 11-nov-2009, según la cabecera del ejecutable (video del prototipo:
  3-oct-2009).
- **Stack:** C# + XNA 3.1, instalador ClickOnce para Windows.
- **De qué trata:** plataformas de acción para dos. Un coipo (la «nutria» chilena) y un pollo
  recorren un nivel de plataformas de roca y recogen cajas con pistolas láser para dispararse. El
  pollo solo era jugable con un mando conectado. Tiene un nivel y un menú de pausa con el fondo
  desenfocado en gris.
- **Port:** ✅ [games/proyecto-nutria](games/proyecto-nutria): lógica traducida del C# decompilado,
  gráficos, fuente y efecto de pausa originales; versión web con controles táctiles. El original
  no tiene sonido.
- **Original:** [elvisbrevi.itch.io/proyecto-nutria](https://elvisbrevi.itch.io/proyecto-nutria)
  (`Proyecto_Nutria.rar`) · [video](https://vimeo.com/6882393)

### IMAN

![IMAN](games/iman/docs/nivel1.png)

- **Fecha:** compilado el 13-nov-2009 (video: 15-dic-2009).
- **Stack:** C# + XNA 3.1, instalador ClickOnce para Windows.
- **De qué trata:** plataformas en el que un niño usa el imán de su mochila para atraer cajas y para
  elevarse hacia plataformas móviles. Dos niveles y un final.
- **Port:** ✅ [games/iman](games/iman): lógica traducida del C# decompilado, gráficos, fuente, shader
  de lápiz y sonido originales; versión web con controles táctiles.
- **Original:** [elvisbrevi.itch.io/iman](https://elvisbrevi.itch.io/iman) (`IMAN_(Prototipo).rar`) ·
  [video](https://vimeo.com/8196488)

### The Electric Eel Attack

![The Electric Eel Attack](games/the-electric-eel-attack/docs/nivel1.png)

- **Fecha:** compilado el 9-mar-2011, versión 1.0.0.13 (el motor, el 8-mar-2011), según las
  cabeceras de los ejecutables (video: 24-feb-2011).
- **Stack:** C# + XNA 3.1 con un motor propio (`Engine.dll`): niveles en XML hechos con su editor,
  física, partículas y efectos. ClickOnce para Windows.
- **De qué trata:** plataformas de acción. Yerko reúne los tres mapas de cada nivel para abrir el
  vórtice de salida, mientras lo atacan anguilas eléctricas: unas patrullan y se electrifican, las
  *kamikazes* saltan. Se las pisa o se las dispara con la energía de las pilas. Tiene cuatro
  niveles: campo, caverna, ciudad y una fábrica con un cañón que apunta.
- **Port:** ✅ [games/the-electric-eel-attack](games/the-electric-eel-attack): motor y juego
  traducidos del C# decompilado, mapas, gráficos, fuentes, sonidos y música originales, y los
  efectos reescritos en GLSL; versión web con controles táctiles.
- **Original:** [elvisbrevi.itch.io/the-electric-eel-attack](https://elvisbrevi.itch.io/the-electric-eel-attack)
  (`The_Electric_Eel_Attack.zip`) · [video](https://vimeo.com/20317603)

### Go Ball

- **Fecha:** build del 22-sep-2011 (teaser: 23-sep-2011).
- **Stack:** Unity 3.4.0f5 con scripts en C# y UnityScript; demo para Windows (32 y 64 bits) y macOS.
- **De qué trata:** puzle con una bola, en 5 niveles. Por confirmar en el triaje.
- **Port:** pendiente.
- **Original:** [elvisbrevi.itch.io/go-ball](https://elvisbrevi.itch.io/go-ball) · [teaser](https://vimeo.com/29466492)

### The Meaning of Christmas

- **Fecha:** diciembre de 2011; APK final del 1-ene-2012 (video: 1-ene-2012).
- **Stack:** Java + libGDX, Android.
- **De qué trata:** puzle navideño, con «Noche de paz» como música. Por confirmar en el triaje.
- **Port:** pendiente.
- **Original:** [elvisbrevi.itch.io/the-meaning-of-christmas](https://elvisbrevi.itch.io/the-meaning-of-christmas)
  (`The Meaning Of Christmas.apk`) · [video](https://vimeo.com/34446793)

### Fast Car Racer

- **Fecha:** video del 21-ene-2013.
- **Stack:** Android, publicado en Google Play (`cl.gioco.fcr`); motor por confirmar.
- **De qué trata:** carreras de autos.
- **Port:** bloqueado: itch.io solo enlaza a Google Play y no hay binario descargable (ver PLAN.md).
- **Original:** [elvisbrevi.itch.io/fast-car-racer](https://elvisbrevi.itch.io/fast-car-racer) ·
  [video](https://vimeo.com/57856159)

### Super Happy Jump

- **Fecha:** hecho con Unity 4.6.0f3, publicado desde noviembre de 2014 (fecha exacta por confirmar).
- **Stack:** Unity 4.6 (Unity Web Player; también iOS y Android).
- **De qué trata:** juego de saltos infinito: toca para saltar, toca dos veces para el doble salto,
  recoge monedas y no toques los cactus.
- **Port:** pendiente.
- **Original:** [elvisbrevi.itch.io/super-happy-jump](https://elvisbrevi.itch.io/super-happy-jump) (`web.unity3d`)

### Que No Caiga

![Que No Caiga](games/que-no-caiga/docs/patada.png)

- **Fecha:** exportado el 23-jul-2015, según su `offline.appcache`. Tiene la temática del Mundial
  de Brasil 2014; la fecha de creación sigue por confirmar.
- **Stack:** Construct 2 (una versión de 2015, r20x), HTML5.
- **De qué trata:** dominar el balón para que no caiga al suelo: cada toque hace patear al jugador.
  Cuenta los segundos, y con el tiempo llegan más pelotas y bombas que explotan en los pies.
- **Port:** ✅ [games/que-no-caiga](games/que-no-caiga): las hojas de eventos traducidas de su
  `data.js`, y lo que usa del runtime minificado (Platform, animaciones, colisiones) traducido
  también; imágenes, sonidos y proyecto originales. Se juega con el ratón o tocando la pantalla.
- **Original:** [elvisbrevi.itch.io/que-no-caiga](https://elvisbrevi.itch.io/que-no-caiga)

### Retro Penal

- **Fecha:** exportado el 22/23-jul-2015; temática del Mundial (creación por confirmar).
- **Stack:** Construct 2, HTML5 y escritorio con NW.js (Windows, macOS, Linux).
- **De qué trata:** penales de fútbol con estética retro y la selección chilena.
- **Port:** pendiente.
- **Original:** [elvisbrevi.itch.io/retro-penal](https://elvisbrevi.itch.io/retro-penal)
  (`win32.zip`, `osx32.zip`, `linux32.zip` y versión HTML5)

### One Button Prototype

- **Fecha:** julio–agosto de 2015 (builds del 11-jul al 17-ago-2015).
- **Stack:** Unity 5.1.2f1 (C# con los Standard Assets 2D); macOS, Linux, Android y WebGL.
- **De qué trata:** plataformas-puzle de un solo botón; el código incluye un dragón como personaje.
- **Port:** pendiente.
- **Original:** [elvisbrevi.itch.io/prototipo-2000](https://elvisbrevi.itch.io/prototipo-2000)

### Neuro Battle

- **Fecha:** build WebGL del 31-ago-2015; GameJam Santo Tomás Concepción 2015.
- **Stack:** Unity 5.1.1f1, solo WebGL (C# compilado con IL2CPP).
- **De qué trata:** shooter hecho en una game jam.
- **Port:** pendiente.
- **Original:** [elvisbrevi.itch.io/neuro-battle](https://elvisbrevi.itch.io/neuro-battle) (`Neuro Battle.zip`)

### Da Car

- **Fecha:** exportado el 23-nov-2015.
- **Stack:** Construct 2, HTML5.
- **De qué trata:** carreras de moto estilo Dakar.
- **Port:** pendiente.
- **Original:** [elvisbrevi.itch.io/da-car](https://elvisbrevi.itch.io/da-car)

---

## Cómo está organizado

```
games/<slug>/        un crate de Cargo por juego (port + assets extraídos + referencia del original)
crates/museo/        piezas comunes: pantalla virtual con bandas, paso fijo y controles táctiles;
                     plantilla web (index.html + mq_js_bundle.js de macroquad) en crates/museo/web
crates/xna/          equivalentes de XNA 3.1 (Rectangle, Clamp, SpriteBatch.Draw, SpriteFont, Random)
originales/          binarios originales (fuera de git) y catalogo.json con sus sha256
tools/originales.py  inventario y descarga de los originales desde itch.io
tools/build_web.sh   versión web de un juego en dist/<slug> a partir de la plantilla
tools/xnb_extract.py, tools/unity_assets.py, tools/c2_eventos.py
                     extracción de contenido XNA, Unity y Construct 2 (ver PLAN.md)
tools/rea_reconstruction.py   C# de ILSpy -> import-managed-reconstruction de REA
tools/ps20_desensamblar.py    píxel shaders ps_2_0 de los efectos XNA, para reescribirlos en GLSL
tools/humo/          tests de humo: nativo (Xvfb + xdotool) y web (Playwright: escritorio y móvil)
PLAN.md              plan por fases con checkpoints
```

## Jugar

```sh
cargo run --release -p iman            # PC (o proyecto-nutria, the-electric-eel-attack, que-no-caiga)
tools/build_web.sh iman                # web: sirve dist/iman con cualquier servidor estático
```

En Linux hace falta `libasound2-dev`. Para la versión web: `rustup target add wasm32-unknown-unknown`.

## Recuperar los originales

Los binarios no se versionan. Para bajarlos de nuevo desde itch.io (comprueba los sha256 de
`originales/catalogo.json`):

```sh
python3 -I tools/originales.py descargar
```
