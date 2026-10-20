# One Button Prototype

**One Button Prototype** (Elvis Brevi, [itch.io](https://elvisbrevi.itch.io/prototipo-2000)) es un
prototipo de plataformas de un solo botón. Un robot corre solo y da media vuelta cada vez que choca
con una pared; el único control es el clic (o el toque), que lo hace saltar. Los pinchos reinician
el nivel, los dragones caminan de pared a pared y solo se los puede vencer cayéndoles encima, y una
estrella al final de cada nivel abre la ventana «Nivel» para elegir entre los tres niveles. Con la
tecla P se pausa y aparece la misma ventana.

Se hizo con Unity 5.1.2f1 en julio de 2015, con el personaje del ejemplo 2D de los Standard Assets
modificado: las versiones de macOS, Linux y Android se compilaron el 22-jul-2015 y la de WebGL se
exportó el 17-ago-2015. Ahora está portado a Rust + macroquad y se juega en PC y en el navegador,
también en móviles, tocando la pantalla. La lógica se tradujo del C# decompilado de
`Assembly-CSharp.dll` y `-firstpass`, y las piezas del motor que usa (la física 2D con Box2D y sus
colisionadores poligonales, la interpolación de los cuerpos, el Animator con blend trees, la GUI de
`OnGUI` con la piel integrada de Unity, el `GUIText`) se recrearon en
[`crates/unity`](../../crates/unity) a partir de los datos del propio juego. Los gráficos, la
fuente del contador de FPS y las tres escenas son los originales: el port lee al arrancar el
`escena.json` que se extrae del build de Linux. El original no tiene sonido.

![nivel 1](docs/nivel1.png) ![menú](docs/menu.png)

![nivel 2](docs/nivel2.png) ![nivel 3](docs/nivel3.png)

## Ejecutar

```sh
cargo run --release -p one-button-prototype
```

En Linux hacen falta las cabeceras de ALSA y OpenGL/X11 (`sudo apt install libasound2-dev`).
Los assets van embebidos en el binario.

### Versión web

```sh
rustup target add wasm32-unknown-unknown
../../tools/build_web.sh one-button-prototype   # genera dist/one-button-prototype
```

El `index.html` sale de la plantilla común de [`crates/museo/web`](../../crates/museo/web).

### Controles táctiles

<img src="docs/movil.png" align="right" width="320" alt="móvil en horizontal">

Como en la versión de Android, no hay botones en pantalla: en móviles y tablets el toque hace de
clic, para saltar y para los botones de la ventana «Nivel». Sin teclado no hay pausa, así que la
ventana solo aparece al llegar a la estrella. Se juega en horizontal; en vertical queda centrado.

<br clear="right">

## Controles

| Entrada | Acción |
|---|---|
| Clic o toque | Saltar (si el robot está en el suelo) |
| P | Pausar y abrir la ventana «Nivel» (otra vez P la cierra) |
| Clic o toque en 1, 2 o 3 | Ir a ese nivel |
| Ctrl izquierdo (mantener) | Agacharse: corre a un cuarto de la velocidad |

## Cómo se hizo

1. **Descarga**: los cuatro archivos de la página de itch.io (`_Mac.zip`, `_Linux.zip`,
   `demo.apk` y `Archivo comprimido.zip`, el WebGL), comprobados contra los sha256 de
   `originales/catalogo.json`. Se extrajeron sin ejecutar nada: el build de Linux en
   `originales/prototipo-2000/linux` y el WebGL en `originales/prototipo-2000/webgl`, con el `.data`
   desempaquetado por `tools/unity_assets.py emscripten`.
2. **Triaje**:
   - Los builds de escritorio y de Android son del 22-jul-2015 (fechas de los zip; el APK trae
     además el motor de Unity del 11-jul). `Assembly-CSharp.dll` (23 552 bytes) y
     `Assembly-CSharp-firstpass.dll` (27 136) se compilaron el **22-jul-2015** a las 06:12 UTC,
     según su cabecera PE. El proyecto se seguía llamando «New Unity Project» y el ejecutable
     «demo».
   - Es Unity **5.1.2f1** con Mono. `BuildSettings` lista tres escenas
     (`Assets/_Niveles/1.unity`, `2.unity` y `3.unity`), guardadas en `mainData`, `level0` y
     `level1`, cada una con su `sharedassetsN` (90, 97 y 45 GameObjects).
   - El WebGL del 17-ago-2015 es el mismo juego: sus tres escenas, comparadas objeto por objeto
     con UnityPy, son idénticas a las de Linux, y los metadatos de IL2CPP traen los mismos nombres
     de miembros y los mismos literales en el mismo orden. Por eso el port parte del build de
     Linux, que trae los ensamblados de C# (el WebGL está compilado a asm.js).
   - [REA](https://rea.tools/) (`rea-agents` 6.7.0), sin cargar ni ejecutar nada:
     - `inspect-managed-artifact`: PE32 solo IL, metadatos completos, runtime `unity-mono`, de
       `Assembly-CSharp.dll`
       (`ev_dad753d9bed54c725b2c8c7521a1808b2a96580eaa3e6aeeee10ce616d06df40`) y de
       `Assembly-CSharp-firstpass.dll`
       (`ev_9be371d572eeaa30a0b42e7da82768621275133f8ae5c678d5ef561c60742700`).
     - `inspect-managed-members` de los seis tipos del juego: 30 métodos, 43 campos, las llamadas,
       los hashes del IL de cada método y los 15 literales que carga el código («p», «Nivel»,
       «Spike», «Final», «ParedRight», «ParedLeft», «Dragon», «GroundCheck»...)
       (`ev_2da9f7381b7ad2b0b45538b07fd737f24b42adfc83d03b127add8c6c6f729869`); y el de
       `FPSCounter` de los Standard Assets
       (`ev_1eb76c5fbece0623b6d179673f544f754ae9fda41c33e2329f2b5a35ba399efc`).
3. **ILSpy** (`ilspycmd` 9.1) con las referencias del mismo build (`UnityEngine.dll` de Unity
   5.1.2 y las bibliotecas de Mono 2.0). En `reference/csharp` quedan los siete scripts que usan
   las escenas: `GUI_Prototipo` (el menú de niveles), `PlatformerDragon2D` y
   `Platformer2DDragonControl` (los dragones), que son del autor; `PlatformerCharacter2D`,
   `Platformer2DUserControl` y `Camera2DFollow`, del ejemplo 2D de Unity y modificados (correr
   solo, rebotar en las paredes, saltar con el clic, morir y pisar dragones, la cámara un poco más
   alta); y `FPSCounter`, de los Standard Assets. El resto del ensamblado (CrossPlatformInput,
   `CameraFollow`, `Restarter` y las utilidades de `-firstpass`) no lo usa ninguna escena. Las
   funciones de `Vector3` que usa la cámara (`SmoothDamp`, `MoveTowards`) también se tradujeron
   del C# de ese `UnityEngine.dll`. [`tools/rea_reconstruction.py`](../../tools/rea_reconstruction.py)
   arma el payload de `import-managed-reconstruction` con 24 de los 30 métodos (los otros seis son
   constructores que solo inicializan campos), pero no se importó (ver
   [Pendientes](#pendientes)).
4. **Assets** ([`tools/extraer.py`](tools/extraer.py), con UnityPy): el build no trae type trees,
   así que las clases del motor se leen con los de UnityPy y los campos de los MonoBehaviour con
   TypeTreeGeneratorAPI a partir de los ensamblados de `Managed/`. Extrae:
   - las 39 texturas de los sprites que se ven, recortadas al rectángulo que ocupan sus sprites (la
     hoja de trampas de 2048x2048 solo aporta los pinchos), y las ocho de la piel integrada de la
     GUI;
   - la fuente OpenSans Regular del contador de FPS, que viene entera en el build (Apache 2.0, con
     su licencia en `assets/fuentes`);
   - y `escena.json`: las tres escenas con su jerarquía y componentes (también los
     `PolygonCollider2D` de los pinchos y las estrellas y el `GUIText` del contador); los 138
     sprites, cada uno como el rectángulo que envuelve su malla, que es ajustada (*Tight*), con la
     parte de la textura que le corresponde según su `uvTransform`; los 17 clips de Mecanim; los
     dos controladores, con el blend tree 1D del salto; los estilos `box`, `button`, `label` y
     `window` de la `GameSkin` integrada de Unity (en `unity default resources`); las métricas de
     Arial y OpenSans; los dos `PhysicsMaterial2D` («Slippery», sin roce, y «BouncyBox», con
     rebote 1) y los ajustes de tiempo, física 2D y sorting layers.
5. **El motor** ([`crates/unity`](../../crates/unity)), que ya traía lo de Super Happy Jump
   (Unity 4.6) y ganó lo que usa este juego, con lo que no se ve leyendo el C#:
   - Varias escenas: `Application.LoadLevel(n)` carga la escena `n` al empezar el frame
     siguiente; si `n` no está en el build, Unity solo escribe un error.
   - Física 2D:
     - `PolygonCollider2D`: Box2D solo acepta polígonos convexos de hasta 8 vértices, así que
       cada camino se triangula cortando orejas y los triángulos vecinos se fusionan mientras el
       resultado siga siendo convexo (Hertel-Mehlhorn). Las piezas pasan por el `Set` de Box2D
       2.3.1 tal como se publicó, que suelda los vértices a menos de 0,05 unidades.
     - Los contactos se agrupan por par de colisionadores: `OnCollisionEnter2D` y
       `OnTriggerEnter2D` llegan cuando empieza a tocarse la primera fixture del par, y
       `Collision2D.collider` es el colisionador del otro objeto.
     - `Physics2D.OverlapCircleAll`, y `Collider2D.bounds` como la caja que envuelve las fixtures
       del colisionador.
     - *Interpolate*: antes de los `FixedUpdate` de un frame los cuerpos interpolados vuelven a
       tener en su Transform la pose de la física, y después se dibujan (y los ve `Update`, como
       la cámara) en `(Time.time - Time.fixedTime) / fixedDeltaTime` entre la pose de antes del
       último paso y la de después.
     - Unity 5 reemplazó `fixedAngle` por las restricciones del cuerpo: `FreezeRotation` es la
       rotación fija de Box2D.
   - Animator (Mecanim de Unity 5):
     - Blend trees 1D: el estado `Jumping` mezcla once clips de un cuadro según `vSpeed`; como
       las curvas de sprite no se mezclan, se ve el del clip que más pesa.
     - El modo *AnimatePhysics* del robot: su Animator avanza en cada paso fijo, después de los
       `FixedUpdate` y antes de la física; los dragones van en modo normal, con `Update`.
     - Las curvas constantes sobre otros componentes, con *Write Defaults*: los clips de
       agacharse cambian el tamaño del `BoxCollider2D`, y al salir de ellos vuelve el original.
       Esos clips son de Unity 4 y también apuntan a `m_Center`, el nombre viejo de `m_Offset`,
       que en Unity 5.1 ya no existe: esa parte de la curva no se aplica.
     - Tiempo de salida y duraciones fijas (este juego no los usa).
   - IMGUI: `GUI.Window` con id, título y foco (la ventana en la que se apretó el ratón se dibuja
     con el estado *on*, «window on»), y el tamaño por defecto de cada fuente (13 la Arial
     integrada) para los estilos con tamaño 0.
   - `GUIText`: el texto en coordenadas de la vista, anclado y alineado, que el `GUILayer` dibuja
     después de la escena y antes de `OnGUI`.
   - Con cámara ortográfica, los sprites de una misma capa y orden se ordenan por su profundidad
     en la vista (aquí todos están en z = 0, así que queda el orden del archivo). La cámara mide
     18 unidades de alto: 33,3 píxeles por unidad.
   - `matematica.rs`: `Vector3.SmoothDamp`, `Vector3.MoveTowards` y `Mathf.Sign`.
6. **Box2D** ([`crates/box2d`](../../crates/box2d)): ganó `set_soldando`, el
   `b2PolygonShape::Set` con la soldadura de vértices que el box2d.js de Construct 2 tiene
   comentada. [`comparar.sh`](../../crates/box2d/comparar/comparar.sh) compila ahora también un
   Box2D como el de Unity (la soldadura activada y `b2_maxPolygonVertices` = 8) para la escena
   nueva «rebote»: un corredor que da la vuelta al tocar paredes sin roce, saltos por fuerza, una
   caja con rebote 1 sobre una repisa y sensores poligonales con vértices soldados. Las
   trayectorias, los contactos, las consultas y los vértices que quedan tras soldar son
   idénticos bit a bit al C++ durante 400 pasos, como las escenas de siempre.
7. **Los scripts** (`src/juego.rs`): cada uno es una estructura con sus campos; los `static`
   (`PlatformerCharacter2D.m_Grounded` y el de `PlatformerDragon2D`) viven fuera de la escena. El
   frame sigue el orden de Unity:
   1. si hay pasos fijos, los cuerpos interpolados vuelven a su pose de la física;
   2. por cada paso de 0,02 s: los `FixedUpdate` en el orden de los objetos (el robot primero
      `PlatformerCharacter2D` y después `Platformer2DUserControl`; los dragones), el Animator del
      robot, la física y los `OnTriggerEnter2D`/`OnCollisionEnter2D`;
   3. la interpolación;
   4. los `Update` en el orden de los objetos (`Camera2DFollow` y `GUI_Prototipo` en la cámara,
      el clic del robot, el `FPSCounter`);
   5. el Animator de los dragones y el borrado de lo destruido (los dragones pisados y el
      `Platformer2DDragonControl` que se destruye a sí mismo);
   6. el dibujo, el `GUIText` y `GUI_Prototipo.OnGUI`.

## Correspondencia con el original

| Unity 5.1 | Rust |
|---|---|
| `demo_Data` (`mainData`, `level0`, `level1`, `sharedassets*`) | `assets/` ([`tools/extraer.py`](tools/extraer.py)) |
| `Assembly-CSharp.dll` y `-firstpass` | `src/juego.rs` |
| `Time`, GameObject, Transform, `Destroy`, `LoadLevel` | `crates/unity` (`tiempo.rs`, `escena.rs`) |
| Física 2D (Box2D), `PolygonCollider2D` | `crates/unity` (`fisica2d.rs`, `poligonos.rs`) y `crates/box2d` |
| Animator (Mecanim) | `crates/unity` (`animator.rs`) |
| Camera, SpriteRenderer | `crates/unity` (`render.rs`) |
| `OnGUI`, la `GameSkin` integrada y `GUIText` | `crates/unity` (`gui.rs`) |
| `Vector3`, `Mathf` | `crates/unity` (`matematica.rs`) |
| El lienzo WebGL de 960×600 | `src/main.rs` con `museo::Pantalla` |

## Rarezas del original que se conservan

- **Un solo botón, de verdad.** El robot no se puede frenar ni girar: corre solo (`h = 1`) y da
  la vuelta cuando choca con un objeto cuyo nombre contiene «ParedRight» o «ParedLeft», que son
  colisionadores invisibles puestos como paredes. Las paredes no tienen roce («Slippery»), el
  suelo sí, y el robot corre a 9,76 en vez de 10: el código fija la velocidad antes de cada paso
  y el roce le quita un poco dentro del paso.
- **El botón 4 no hace nada.** `GUI_Prototipo` dibuja cuatro botones (`cantidadNiveles = 4`), pero
  el build tiene tres escenas: `Application.LoadLevel(3)` solo escribe un error.
- **La estrella no lleva al nivel siguiente**: pausa el juego y abre la ventana «Nivel», donde se
  elige. Los pinchos vuelven a cargar el mismo nivel.
- **El clic en la ventana también es un salto.** `Platformer2DUserControl` mira el botón del ratón
  en `Update`, también con el juego en pausa, y deja el salto pendiente para el próximo
  `FixedUpdate`: si se cierra la ventana con P, el robot salta enseguida. Al elegir un nivel no
  pasa, porque la escena nueva empieza sin salto pendiente.
- **El suelo es «todo».** `m_WhatIsGround` tiene todas las capas y las consultas de Physics2D ven
  los triggers: el robot cuenta como en el suelo sobre un dragón, al lado de la estrella o
  rozando los pinchos. El círculo que lo mira (radio 0,2) llega 0,19 unidades bajo los pies, así
  que se puede saltar un instante antes de aterrizar.
- **Pisar o morir.** Al chocar con un dragón, si el `GroundCheck` del robot está más alto que el
  centro de la caja del dragón, el dragón se destruye; si no, el nivel vuelve a empezar. El
  segundo dragón del nivel 3 empieza muy atrás, a la izquierda del robot, y llega más tarde.
- Un dragón que toca los pinchos se destruye su propio script de control (`Destroy(this)`): sigue
  con la velocidad que tenía y ya no se da vuelta.
- **La cámara mira 10 unidades adelante** (`lookAheadFactor = 10` y umbral 0): cada vez que el
  robot da la vuelta, la vista se va al otro lado. Va además una unidad más alta que el robot, y
  el campo `tiempoSalto` que se agregó a `Camera2DFollow` no se usa.
- **El contador de FPS solo está en el nivel 1** (el objeto `FramerateCounter` solo existe en esa
  escena) y al reiniciar el nivel muestra «FPS» medio segundo, hasta su primera medición.
- En el aire, la transición de *Any State* a `Jumping` puede volver al mismo estado y lo reinicia
  en cada actualización; no se nota porque los clips del salto son de un cuadro.
- Agacharse (Ctrl izquierdo) deja la caja del robot más baja y más ancha, pero centrada donde
  estaba (ver la curva de `m_Center` en [Cómo se hizo](#cómo-se-hizo)).
- La ventana «Nivel» usa la piel integrada de Unity, semitransparente: el juego en pausa se ve
  detrás, y al hacer clic en ella pasa a la textura «window on», un poco más opaca.

## Diferencias con el original

- **Fuente**: la GUI usaba la Arial del sistema (la fuente integrada de Unity); el port usa
  Liberation Sans 2.1.5 (SIL OFL 1.1), con las métricas de Arial, de
  [`crates/construct2/fuentes`](../../crates/construct2/fuentes). El contador usa la OpenSans
  original.
- **Box2D**: Unity 5.1 traía su propio Box2D compilado, de versión no publicada; el port usa
  `crates/box2d` (2.3.1) con la soldadura de vértices del Box2D publicado. Unity agrupa los
  colisionadores sin `Rigidbody2D` en un cuerpo estático; aquí cada objeto tiene el suyo.
- **Las piezas de los `PolygonCollider2D`** no son las de Unity, que no publica su algoritmo,
  pero cubren la misma región, que es lo que ven los triggers y las consultas.
- **Lo que hace Unity por dentro** se dedujo de su documentación y de su comportamiento conocido,
  porque el motor es nativo y los binarios originales no se ejecutan: que los cuerpos interpolados
  vuelvan a su pose de la física antes de los `FixedUpdate`, el momento de la animación en
  *AnimatePhysics*, que la curva de `m_Center` ya no se aplique, que una ventana tome el foco al
  hacer clic en ella, que `Collider2D.bounds` use la pose actual del cuerpo y que en una
  transición de Mecanim se vea el sprite del estado que pesa más.
- **Sprites**: cada uno se dibuja como el rectángulo que envuelve su malla ajustada; lo que queda
  fuera de la malla es transparente en la textura.
- **Orden de dibujo**: con sprites en la misma capa, el mismo orden y la misma profundidad, Unity
  no garantiza un orden; aquí es el del archivo de la escena.
- **Ventana**: los builds de escritorio abrían el diálogo de resolución de Unity, en pantalla
  completa (1024×768 por defecto), con el título «New Unity Project»; el port usa el lienzo de la
  versión WebGL (960×600) escalado a la ventana con bandas negras, como el resto del museo, y se
  llama «One Button Prototype». No tiene la pantalla de presentación de Unity.

## Regenerar los assets

```sh
~/.venvs/museo/bin/python -I tools/extraer.py <demo_Data>   # por ejemplo ../../originales/prototipo-2000/linux/_Linux/demo_Data
```

## Pruebas

```sh
node ../../tools/humo/nativo.mjs one-button-prototype   # tras cargo build --release
node ../../tools/humo/web.mjs one-button-prototype      # tras tools/build_web.sh
```

Siguen los pasos de `humo.json`: el nivel 1 con el contador de FPS, un salto, la pausa con P y la
ventana «Nivel», y los niveles 2 y 3 elegidos en ella (en el móvil, donde no hay teclado, los
toques son saltos). En la web se prueban escritorio, móvil horizontal y móvil vertical. Se
comprobó aparte:
- `crates/box2d/comparar/comparar.sh`: las escenas de siempre y la nueva «rebote», idénticas al
  C++ de Box2D 2.3.1;
- `cargo test -p unity`: la descomposición de polígonos, el blend tree, `SmoothDamp` y lo de
  antes;
- con un registro temporal de la física: que el robot corre, aterriza, da la vuelta en las
  paredes y muere en los pinchos, y que los dragones caminan, se dan vuelta y lo matan de frente;
- con un cambio temporal (el robot puesto junto a la estrella del nivel 1): que la estrella pausa
  y abre la ventana.

## Pendientes

- **`import-managed-reconstruction` en REA**: REA autentica la Evidence de miembros por su
  contenido y hay que pasarla entera en la llamada (unos 250 KB para estos seis tipos, por las
  referencias de todo el ensamblado); no se importó. El payload se regenera con
  `tools/rea_reconstruction.py` a partir de esa Evidence.
- No se pudo comparar con el original en marcha: los binarios originales no se ejecutan.
- Los niveles no se jugaron de principio a fin sin ayuda, y no se probó pisar un dragón (en el
  nivel 3, el primero camina bajo una plataforma y el segundo junto a la pared del borde).
- Los controles táctiles se probaron solo en emulación de Chromium.
