# The Meaning of Christmas

**The Meaning of Christmas** (Elvis Brevi,
[itch.io](https://elvisbrevi.itch.io/the-meaning-of-christmas)) es un juego navideño para el
móvil sobre el consumismo. Caen regalos sobre un cielo de noche con luna, y desde abajo sube una
montaña de dinero: cada regalo que llega al dinero sin quemar lo hace crecer. Para quemar regalos
hay que tocar tres a la misma altura: el primero enciende una franja de fuego a su altura, los
otros dos tienen que tocarse dentro de ella, y los tres se vuelven rojos. Cada regalo rojo que
cae al dinero suma 25 puntos y lo hace bajar. Cuando el dinero cubre casi toda la pantalla, se
vuelve al título («The True Meaning of Christmas»), con el puntaje. En el título suena «Noche de
paz» y durante el juego, otra música.

Se hizo en diciembre de 2011 con Java y libGDX 0.9.2 para Android: el APK se armó el
1-ene-2012. Ahora está portado a Rust + macroquad y se juega en PC y en el navegador, también en
móviles, tocando la pantalla. La lógica se tradujo del Java decompilado del APK, y lo que usa de
libGDX (el dibujo, la fuente por defecto, la animación, el tiempo del frame y la música) se
tradujo del código de libGDX que viene en el mismo APK. Las imágenes, la fuente y la música son
las originales (la música, pasada de MP3 a Ogg Vorbis).

![título](docs/titulo.png) ![fuego](docs/fuego.png)

![dinero](docs/dinero.png) ![fin](docs/fin.png)

## Ejecutar

```sh
cargo run --release -p the-meaning-of-christmas
```

En Linux hacen falta las cabeceras de ALSA y OpenGL/X11 (`sudo apt install libasound2-dev`).
Los assets van embebidos en el binario.

### Versión web

```sh
rustup target add wasm32-unknown-unknown
../../tools/build_web.sh the-meaning-of-christmas   # genera dist/the-meaning-of-christmas
```

El `index.html` sale de la plantilla común de [`crates/museo/web`](../../crates/museo/web).

### Controles táctiles

<img src="docs/movil.png" align="right" width="206" alt="móvil en vertical">

Como en Android, no hay botones en pantalla: se toca el título para empezar y se tocan los
regalos. Se juega en vertical; en horizontal queda centrado.

<br clear="right">

## Controles

| Entrada | Acción |
|---|---|
| Clic o toque en el título | Empezar |
| Clic o toque en un regalo | Elegirlo (el primero enciende la franja; tres en la franja se queman) |

## Cómo se hizo

1. **Descarga**: el APK de la página de itch.io (8 158 243 bytes), comprobado contra el sha256 de
   `originales/catalogo.json`. Se descomprimió y decompiló sin ejecutar nada, en
   `originales/the-meaning-of-christmas` (`jadx` y `apktool`).
2. **Triaje**:
   - Las imágenes y la música son del 10 al 31-dic-2011, y el resto del APK se armó y firmó el
     **1-ene-2012** a las 17:31 (fechas del zip), con el certificado de depuración de Android
     (creado el 17-ago-2011) y `android:debuggable="true"`. `minSdkVersion` 3, sin
     `targetSdkVersion`, y la actividad en vertical.
   - Es libGDX **0.9.2** (`com.badlogic.gdx.Version`). El juego se hizo sobre el «hello world» de
     libGDX: el paquete de la aplicación es `com.badlogic.gdx` y las clases del juego están en
     `com.badlogic.gdx.helloworld`, junto con un lanzador de escritorio (JOGL, 240×320) que no se
     usa en Android.
   - [REA](https://rea.tools/) (`rea-agents` 6.7.0, con apktool 3.0.3) `decode-android-resources`:
     el manifiesto, la versión 1.0 y los textos («The Meaning of Christmas» y «Welcome to The
     Meaning of Christmas»), sin decodificar el código
     (`ev_c5fdc5ccb3d3d000a744d250d91c5b4e0ffcbfa3c3b62acb6b399e4401ffe46e`). Las herramientas de
     REA que recuperan el Java piden un servidor JADX MCP, que no está configurado
     (`REA_JADX_MCP_JAR`), así que el código se decompiló con JADX directamente.
3. **JADX** 1.5.6: las siete clases del juego (517 líneas) están en `reference/java`, con el
   lanzador de Android. Además se leyó, del libGDX del mismo APK, lo que el juego usa: `BitmapFont`
   (cómo se lee y se dibuja la fuente), `Animation`, `SpriteBatch`, `Texture` (filtro Nearest
   por defecto), `WindowedMean` y `AndroidGraphics` (el tiempo del frame).
4. **Assets** ([`tools/extraer.py`](tools/extraer.py), que lee el APK como zip): las diez
   imágenes que carga el código (`campanas.png`, `game over.png` y `monedas.png` vienen en el
   APK pero no se usan), la fuente `arial-15` (`.fnt` de BMFont y su `.png`) que libGDX trae
   para `new BitmapFont()`, y las dos músicas, de MP3 a Ogg Vorbis con ffmpeg (macroquad no lee
   MP3).
5. **libGDX** ([`src/gdx.rs`](src/gdx.rs)), con lo que no se ve leyendo el juego:
   - Las coordenadas tienen el origen abajo a la izquierda, en píxeles de la pantalla.
   - `new BitmapFont()` lee `arial-15.fnt` como `BitmapFontData` (sin voltear: el `yoffset` de
     cada glifo pasa a ser `-(alto + yoffset)`) y dibuja con `integer`: la `y` de `draw` es la
     altura de las mayúsculas (se le suma `base - capHeight`) y las posiciones se truncan a
     enteros, con el kerning de la fuente. La fuente no tiene espacio: libGDX crea uno con el
     avance de la «l».
   - `Gdx.graphics.getDeltaTime()` en Android es la media de los últimos cinco frames
     (`WindowedMean`), y el último mientras no hay cinco.
   - `Animation.getKeyFrame(t, true)` es `(int) (t / 0.025) % 4`.
   - Un `Music` que termina deja de sonar, y el juego lo vuelve a empezar en el frame siguiente
     (`if (!isPlaying()) play()`): aquí suena en bucle.
6. **El juego** ([`src/juego.rs`](src/juego.rs)): `Juego.render`, `Inicio`, `Regalo`, `Cielo`,
   `Animacion` y los campos `static` de `Operaciones`, en el mismo orden: el título o, en la
   partida, la comprobación del dinero, los toques y la caída de cada regalo, la caída en el
   dinero (`controlarCaida`) y el dibujo.

## Correspondencia con el original

| libGDX (Android) | Rust |
|---|---|
| `assets/data`, `com/badlogic/gdx/utils/arial-15.*` del APK | `assets/` ([`tools/extraer.py`](tools/extraer.py)) |
| `com.badlogic.gdx.helloworld` | `src/juego.rs` |
| `SpriteBatch`, `Texture`, `BitmapFont`, `Animation`, `Music`, `Gdx.graphics`, `Gdx.input` | `src/gdx.rs` |
| La pantalla del teléfono | `src/main.rs` con `museo::Pantalla` (320×480) |

## Rarezas del original que se conservan

- **Dos de las ocho columnas están fuera de la pantalla.** Cada regalo vuelve a salir en una
  columna al azar del 10 % al 115 % del ancho, de 15 en 15: en las del 100 % y el 115 % cae sin
  verse, nadie lo puede tocar y siempre hace crecer el dinero.
- **Siempre hay un regalo esperando.** Son nueve regalos para ocho columnas, y uno que sale en
  una columna ocupada se queda quieto, sorteando otra columna en cada frame, hasta que encuentra
  una libre.
- **Al empezar, el dinero sube de golpe.** Los nueve regalos empiezan en (0, 0), así que en el
  primer frame de la primera partida todos caen en el dinero.
- **Cuánto se mueve el dinero depende del frame**: un regalo perdido lo sube `5 * delta * 60`
  píxeles y uno quemado lo baja `10 * delta * 60`, así que a 60 fps son 5 y 10 píxeles, y con
  frames más lentos, más.
- **Los regalos elegidos se quedan en el aire.** Un regalo seleccionado no cae: si se eligen uno
  o dos y no se completa la franja, quedan quietos hasta que un toque fuera de la franja cancela
  todo (vuelven a ser blancos). Lo que cuenta para la franja es el punto tocado, no el regalo.
- **Un regalo rojo se puede volver a contar**: después de quemarse sigue cayendo rojo y no
  seleccionado, así que tocarlo dentro de una franja nueva suma otra vez, y tocarlo fuera no
  cancela nada.
- **El fuego va más rápido de lo que dice el código.** `Animacion` tiene velocidad 0,1, pero el
  tiempo avanza cada vez que se dibuja, y la franja son 13 trozos: el fuego va 1,3 veces más
  rápido que sin frenar, y trozos vecinos pueden mostrar cuadros distintos.
- **La velocidad no se reinicia.** Los regalos aceleran por tramos de puntaje (de 1,2 sobre 500
  a 2,8 sobre 4500, con los límites excluidos: justo en 1000, 1500... no cambia), y en la partida
  siguiente empiezan a la velocidad con que terminó la anterior. Tampoco se reinician las
  posiciones, los colores ni una franja a medias.
- El dinero se dibuja encima de los regalos: los que llegan se hunden en él.
- La luna se dibuja al 30 % del ancho y del alto, así que en un teléfono vertical sale ovalada;
  la imagen del título se estira al 110 % del ancho y baja un 10 % más allá del borde.
- El título dice «The True Meaning of Christmas» y la aplicación, «The Meaning of Christmas». El
  texto pide hacer «click» en un teléfono.

## Diferencias con el original

- **Pantalla**: el juego mide todo en porcentajes de la pantalla del teléfono, salvo la fuente,
  de 15 píxeles. El port usa 320×480, la de un Android HVGA: el APK, con `minSdkVersion` 3 y sin
  declarar otras pantallas, corre en modo de compatibilidad con esa densidad. Se escala a la
  ventana con bandas, como el resto del museo.
- **Música**: Ogg Vorbis en lugar de los MP3 originales, en bucle (ver arriba). En la web el
  navegador no deja sonar nada hasta el primer toque, que además empieza la partida: la primera
  vez el título queda en silencio.
- **Azar**: el orden de las tiradas es el del código, pero el generador es el de macroquad, no
  `java.util.Random`.
- En PC el clic hace de toque.

## Regenerar los assets

```sh
python3 -I tools/extraer.py "<The Meaning Of Christmas.apk>"   # hace falta ffmpeg
```

## Pruebas

```sh
node ../../tools/humo/nativo.mjs the-meaning-of-christmas   # tras cargo build --release
node ../../tools/humo/web.mjs the-meaning-of-christmas      # tras tools/build_web.sh
```

Siguen los pasos de `humo.json`: el título, el toque para empezar y la partida, con los regalos
cayendo y el dinero subiendo. En la web se prueban escritorio, móvil horizontal y móvil vertical.
Se comprobó aparte, con cambios temporales sin subir: un jugador automático que toca regalos en
la franja (las capturas `fuego.png`: la franja, los regalos rojos, el puntaje y los dos regalos
que se quedan en el aire con la franja a medias), y el dinero puesto cerca del tope para llegar
al final de la partida (`dinero.png` y el título con el puntaje, `fin.png`). Una traza de las
posiciones confirmó que los regalos caen a 60 píxeles por segundo y las dos columnas fuera de la
pantalla.

## Pendientes

- El sonido no se probó en hardware real (el servidor no tiene tarjeta de sonido).
- Los controles táctiles se probaron solo en emulación de Chromium.
- Las herramientas Android de REA que recuperan el Java (las de JADX) no están configuradas: falta
  un JAR de JADX MCP en `REA_JADX_MCP_JAR`.
