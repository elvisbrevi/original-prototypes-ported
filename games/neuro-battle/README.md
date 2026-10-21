# Neuro Battle

**Neuro Battle** (Elvis Brevi y el equipo «GROUP_5», [itch.io](https://elvisbrevi.itch.io/neuro-battle))
es un prototipo de acción hecho en la GameJam Santo Tomás Concepción de 2015. Se maneja una neurona
verde dentro de un tejido rojo: por el nivel aparecen otras neuronas (básicas, de cola larga, de
defensa y de ataque) y, al tocarlas con las patas, se unen a la tuya y giran con ella. Desde los
bordes llegan enemigos que te persiguen: si tocan tu neurona, la partida termina y el puntaje son
los segundos que aguantaste; si tocan una neurona unida, mueren los dos. Una neurona de ataque
unida dispara. Las instrucciones lo resumen como «Une NEURONAS, ALIMÉNTALAS y EVOLUCIONA tu
HUMANO» (alimentar y evolucionar no llegaron a programarse).

Se hizo con Unity 5.1.1f1 y solo se publicó en WebGL, con el C# compilado con IL2CPP a asm.js: el
build es del 31 de agosto de 2015. Ahora está portado a Rust + macroquad y se juega en PC y en el
navegador, también en móviles, con controles táctiles. Como el WebGL no trae ensamblados de C#, la
lógica se reconstruyó a partir del asm.js con los metadatos de IL2CPP, que en esta versión de Unity
estaban compilados dentro de su memoria, y el original se observó corriendo en un Chromium actual.
Los gráficos, los sonidos y las cuatro escenas son los originales; la física 2D (Box2D), el
Animator y la GUI son las piezas de [`crates/unity`](../../crates/unity).

![nivel](docs/nivel.png) ![título](docs/titulo.png)

![instrucciones](docs/instrucciones.png) ![zoom](docs/zoom.png)

## Ejecutar

```sh
cargo run --release -p neuro-battle
```

En Linux hacen falta las cabeceras de ALSA y OpenGL/X11 (`sudo apt install libasound2-dev`).
Los assets van embebidos en el binario.

### Versión web

```sh
rustup target add wasm32-unknown-unknown
../../tools/build_web.sh neuro-battle   # genera dist/neuro-battle
```

El `index.html` sale de la plantilla común de [`crates/museo/web`](../../crates/museo/web).

### Controles táctiles

<img src="docs/movil.png" align="right" width="360" alt="móvil en horizontal">

En móviles y tablets aparecen una cruceta (la neurona se mueve en ocho direcciones), los botones
«A» (disparar), «<» y «>» (girar) y «B» (soltar la última neurona unida), y «ZOOM». En el título,
las instrucciones y el game over, tocar la pantalla equivale a la E. Se juega en horizontal.

<br clear="right">

## Controles

| Entrada | Acción |
|---|---|
| W, A, S, D | Mover la neurona (con la aceleración y la inercia de los ejes de Unity) |
| ← y → | Girar la neurona |
| Shift derecho | Disparar con las neuronas de ataque unidas |
| Alt izquierdo o botón derecho del ratón | Soltar la última neurona unida |
| E | Jugar (en el título, las instrucciones y el game over) y zoom (en el nivel) |
| Esc | Volver al título (en el nivel, las instrucciones y el game over) |

## Cómo se hizo

1. **Descarga**: `Neuro Battle.zip` (13,99 MB), comprobado contra el sha256 de
   `originales/catalogo.json` y extraído en `originales/neuro-battle/extraido` sin ejecutar nada.
   Es un WebGL de Unity 5.1.1f1 (producto «Neuro», lienzo de 960×600): `Neuro Battle.js` (24,7 MB
   de asm.js de Emscripten, minificado), su memoria inicial `Neuro Battle.html.mem` (3,2 MB) y el
   paquete de archivos `Neuro Battle.data` (28,9 MB), escritos el 31-ago-2015 entre las 10:37 y
   las 10:47.
2. **Triaje con [REA](https://rea.tools/)** (`rea-agents` 6.7.0):
   - `inspect-artifact` del zip
     (`ev_ea8cecdb980921ee8e4cf64d048b4dcea8aee9aae7e8ec8c0943e8595ed99946`) y
     `analyze-javascript-application` del cargador (`index.html`, `UnityConfig.js`,
     `fileloader.js` y `UnityProgress.js`): sin red ni almacenamiento propios
     (`ev_6d65876e17591498545582fb9368462509ed6a7f5798fd55b1b3c3bca28647d2`).
   - El `.data` se desempaqueta con `tools/unity_assets.py emscripten` según las `DataRequest` del
     `fileloader.js`: las cuatro escenas de `BuildSettings` (`titulo`, `instrucciones`, `level` y
     `game_over`, en `mainData` y `level0` a `level2`), `sharedassets0` a `3` y los recursos
     integrados.
3. **Los metadatos de IL2CPP**
   ([`tools/il2cpp.py`](tools/il2cpp.py)): el WebGL 5.1.2 de One Button Prototype trae un
   `global-metadata.dat`, pero en 5.1.1 IL2CPP todavía compilaba los metadatos como datos estáticos
   de C++, así que están en la memoria inicial del asm.js (que Emscripten carga en la dirección 8),
   y herramientas como Il2CppDumper no los leen. Se leen así:
   - el `Il2CppMetadataRegistration` (la única estructura que apunta a la tabla de 5166
     `Il2CppType`) lleva a las ocho asambleas, los métodos genéricos y la tabla de referencias a
     métodos que resuelve el código generado;
   - cada imagen tiene su lista de `TypeInfo` (0xa8 bytes: nombre, espacio de nombres, lista de
     `MethodInfo`, tamaño, flags, cantidad de campos) y las definiciones de sus campos (nombre,
     tipo y offset, en el orden de los tipos); la visibilidad y si es estático están en los
     atributos de su `Il2CppType`;
   - cada `MethodInfo` (0x38 bytes) trae el índice de su función en la tabla de su firma. Las
     tablas del asm.js están minificadas, pero cada `dynCall_<firma>` exportado dice cuál es: con
     la firma del método (los estáticos también reciben un `this` nulo, todos terminan con el
     `MethodInfo*` oculto y un struct se devuelve por un puntero que va primero) se llega al cuerpo
     de cada método;
   - los literales son objetos `String` estáticos (clase, monitor, largo y UTF-16).

   La herramienta vuelca en [`reference/il2cpp`](reference/il2cpp) los tipos del juego
   (`metadatos.txt`), los 85 métodos del motor que llaman (`motor.txt`) y el asm.js de cada método
   (`asmjs/`) con las llamadas, los tipos y métodos que se resuelven al primer uso (`DO`, `FO`),
   las llamadas dentro de un `try` (`invoke_*`) y los textos anotados.
4. **La lógica, reconstruida a mano** de ese asm.js en [`reference/csharp`](reference/csharp)
   (`BaseMovimiento` y `BaseRotacion`, la neurona del jugador; `Union` y `animacionUnion`, las de
   las demás; `disparar` y `bala`; `bordescript`; `muerte_enemigo`; `Aparicion`, que hace aparecer
   neuronas cada 6 s y lleva el tiempo; el `Camera2DFollow` de los Standard Assets; `titulo`,
   `intrucciones` y `gameover`; y `PlayerControl` y `matar_neurona`, que nadie usa) y en
   [`reference/unityscript`](reference/unityscript) (`GeneradorVitaminas`, los 17 generadores que
   clonan al enemigo cada 5 s, e `IA_Enemy`). Lo que no se ve en un C#:
   - IL2CPP 5.1.1 no comprueba nulos: en `bala`, restarle vida a un borde (que no tiene
     `muerte_enemigo`) escribe en la memoria baja del asm.js y la bala igual se destruye;
   - `Mathf.Round` quedó *inlined* como `roundf` en `IA_Enemy.Seguir`;
   - en UnityScript, las llamadas a los métodos propios son virtuales;
   - el `try`/`catch (UnityException)` de `Aparicion` pasa por los `invoke_*` de Emscripten, y no
     atrapa la `ArgumentException` de instanciar algo nulo.
5. **Assets** ([`tools/extraer.py`](tools/extraer.py), con UnityPy, como el de One Button
   Prototype): sin ensamblados, los campos de los MonoBehaviour se leen con type trees que
   TypeTreeGeneratorAPI genera de unos ensamblados *stub* con solo los campos de cada script,
   escritos con `il2cpp.py` y compilados con el `csc` del SDK de .NET contra el `UnityEngine.dll`
   5.1.2 de One Button Prototype; los 72 MonoBehaviour se leen enteros. `escena.json` lleva las
   cuatro escenas (1, 1, 96 y 1 objetos), los cinco prefabs que instancian los scripts, 189
   sprites (de 12 texturas, recortadas a lo que usan), las tres pantallas de la GUI, 9 clips y 7
   controladores de Mecanim, el `GUIStyle` del puntaje, los ejes del InputManager y la física 2D
   con su matriz de capas. Los dos sonidos, en AAC dentro de MP4 (el formato de Unity 5.1 para
   WebGL), pasan a Ogg Vorbis con ffmpeg. La fuente del puntaje, «Light Pixel-7» de Style-7, es
   gratis solo para uso personal y no se copia.
6. **El original en marcha**: el WebGL de 2015 todavía corre en el Chromium de Playwright (sin
   interfaz, con WebGL por SwiftShader), servido en local; tarda unos 15 s en cargar y muestra la
   pantalla «made with unity» antes del título.
   - REA `capture-browser-scenario`, con la consola y una captura por paso
     (`ev_7e177af039fd0bb53b0ee21cea32633a6af97b34c31d1baa5250f791d35024f0`): título,
     instrucciones, nivel y, sin tocar nada, «game over» a los 5,7 s («TU PUNTAJE FUE DE 5
     SEGUNDOS»). Justo al morir aparecen 34 `NullReferenceException`: las 34 copias del enemigo
     (17 generadores, dos tandas) leen al jugador ya destruido.
   - Con Playwright y las teclas mantenidas: al cruzar una neurona libre, se une y desde entonces se
     arrastra y gira con la tuya. Las patas son cuerpos *kinematic* con triggers, y en el Box2D
     estándar dos cuerpos kinematic no se tocan: el de Unity 5.1 sí lo permitía con triggers.
   - Con el original a 20 fps: una E de 40 ms muestra las instrucciones, de 120 ms las salta y de
     250 ms además entra al nivel con zoom (ver [Rarezas](#rarezas-del-original-que-se-conservan)).
   - Muriendo dos veces seguidas, el segundo puntaje es de 10 segundos: el tiempo se acumula.
   - Con la API de WebAudio intervenida: `PlayOneShot(clip, 100)` pone la ganancia en 100.
7. **El motor** ([`crates/unity`](../../crates/unity)), que ya traía lo de One Button Prototype, y
   [`crates/box2d`](../../crates/box2d):
   - Box2D ganó el `b2Filter` (`SetFilterData` con su `Refilter`), con 32 bits para las capas de
     Unity: la capa del objeto es la categoría y su fila de la matriz de colisiones, la máscara.
     Aquí importa: los muros (capa `Background`) solo chocan con las neuronas (capa `Neurona`),
     las balas los atraviesan y las neuronas no se tocan entre sí. También ganó la opción de que
     dos cuerpos kinematic se toquen si uno de los colisionadores es un trigger, lo que se observó
     en el original.
   - [`comparar.sh`](../../crates/box2d/comparar/comparar.sh) tiene una escena nueva,
     «neuronas»: una neurona dinámica que el código teletransporta contra una pared, patas
     kinematic que giran con ella, una pieza que se une (su trigger pasa al cuerpo de la neurona),
     un enemigo estático que se mueve, una bala que el filtro deja atravesar la pared y un cambio
     de capa a mitad. Es idéntica bit a bit al Box2D 2.3.1 en C++ compilado con los mismos dos
     cambios, como las seis escenas de siempre.
   - Un colisionador sin Rigidbody2D propio se pega al del padre más cercano: cuando una neurona
     se une, su trigger pasa a ser parte del cuerpo de la tuya (y así la protege de los enemigos,
     que son colisionadores estáticos).
   - El InputManager ganó `GetButton` y `GetButtonUp` (Fire2), la escena, `instanciar_datos` (un
     objeto de la escena clonado con «(Clone)»), el audio, `PlayOneShot`, y la GUI, una escala
     para dibujar una fuente sustituta del tamaño de la original.
8. **Los scripts y el frame** (`src/juego.rs`): cada script es una variante con sus campos; los
   `static` (`Aparicion.contadorTiempo` y `BaseRotacion.choque`) viven fuera de la escena. El frame
   sigue el orden de Unity 5.1: la escena pedida con `Application.LoadLevel` se carga al empezar el
   frame siguiente; la física a paso fijo (ningún script tiene `FixedUpdate`) con sus
   `OnTriggerEnter2D` y `OnCollisionEnter2D`; lo destruido se borra después de la física y después
   de los `Update`; los `Start` pendientes; los `Update` en el orden de los objetos; el Animator; el
   dibujo y `OnGUI`. Un script que lee un objeto destruido corta su método ahí, como la excepción
   del original (así se reproduce que, muerto el jugador, los enemigos se queden quietos ese
   frame).

## Correspondencia con el original

| Unity 5.1 WebGL | Rust |
|---|---|
| `Neuro Battle.data` (`mainData`, `level0` a `level2`, `sharedassets*`) | `assets/` ([`tools/extraer.py`](tools/extraer.py)) |
| El asm.js de IL2CPP y sus metadatos | `reference/` ([`tools/il2cpp.py`](tools/il2cpp.py)) y `src/juego.rs` |
| `Time`, GameObject, Transform, `Instantiate`, `Destroy`, `LoadLevel` | `crates/unity` (`tiempo.rs`, `escena.rs`) |
| Física 2D (Box2D) con la matriz de capas | `crates/unity` (`fisica2d.rs`) y `crates/box2d` |
| InputManager | `crates/unity` (`entrada.rs`) |
| Animator (Mecanim) | `crates/unity` (`animator.rs`) |
| Camera, SpriteRenderer | `crates/unity` (`render.rs`) |
| `OnGUI`: `GUI.DrawTexture` y `GUI.Label` con su `GUIStyle` | `crates/unity` (`gui.rs`) |
| `AudioSource.PlayOneShot` (WebAudio) | `crates/unity` (`audio.rs`) |
| El lienzo WebGL de 960×600 | `src/main.rs` con `museo::Pantalla` |

## Rarezas del original que se conservan

- **El puntaje se acumula.** `Aparicion.contadorTiempo` es estático y nunca vuelve a 0: la
  segunda partida suma el tiempo de la primera (verificado en el original). Se muestra
  redondeado, con `ToString("0")`.
- **La E se lee con `GetKey`.** Si sigue apretada cuando carga la escena siguiente, cuenta otra
  vez: una E que dura más de un frame salta las instrucciones, y si dura un frame más, el nivel
  empieza con zoom (es la misma tecla). A 60 fps casi cualquier pulsación las salta; en el móvil,
  un toque es la E durante un solo frame.
- **Los generadores clonan al enemigo de la escena**, no a un prefab: cada clon copia su estado
  en ese momento (su vida, su objetivo y su punto de patrulla, que comparten). Si ese enemigo
  muere, `Instantiate` falla y no aparecen más enemigos.
- **El enemigo patrulla hasta un punto y se queda ahí**: el punto de patrulla no se renueva.
  Persigue a la neurona a menos de 15 unidades, avanzando por separado en x y en y
  (`Mathf.Sign(Mathf.Round(...))`: alineado a menos de media unidad, avanza hacia la derecha o
  hacia arriba), y desaparece solo si llega a menos de 1,1 del centro. El estado «huir» existe
  pero nadie lo usa (multiplica la posición entera por `deltaTime`).
- **La última neurona de la lista nunca aparece**: `Aparicion` tiene 20 y sortea con
  `Random.Range(0, 19)`. Aparecen en cualquier punto del rectángulo (-66, -32)–(18, 17), también
  fuera de los muros.
- **Las patas se unen a cualquier «pata»**: también a las del borde (que se llaman así), y una
  neurona libre que toca el borde queda colgando de él y ya no se puede unir. Una neurona de ataque
  se vuelve válida para disparar con cualquier pata que toque, aunque no esté unida.
- **Las balas avanzan una unidad por frame**: su velocidad depende de los fps.
- **El sonido sale saturado**: `PlayOneShot(clip, 100)` lleva la ganancia de WebAudio a 100.
- **Soltar neuronas** (Alt izquierdo o el botón derecho) suelta la última neurona unida
  directamente a la tuya, solo si hay alguna y una vez por pulsación; las que cuelgan de ella se
  van con ella.
- **Escape en el título** llama a `Application.Quit`, que en el WebGL no hace nada; en el port
  tampoco.
- `BaseRotacion.choque`, `Union.estado`, `velocidadHuida` y `rangoMovimiento` se escriben o se
  serializan pero nadie los lee; `animacionUnion` no hace nada.

## Diferencias con el original

- **Fuente**: «Light Pixel-7» (Style-7) es gratis solo para uso personal; el port usa
  [Micro 5](https://fonts.google.com/specimen/Micro+5) (SIL OFL 1.1, en `assets/fuentes`), del
  mismo estilo, dibujada 2,1 veces más grande para que el texto mida lo mismo, con las métricas
  verticales de la original. Comparado con una captura del original, el puntaje queda en el mismo
  lugar (500×24 contra 485×22 píxeles).
- **Lo que hace Unity por dentro** se dedujo de su comportamiento y de lo que se observó:
  cuándo se cargan las escenas (comprobado con los tiempos de la E) y cuándo se borra lo
  destruido, el orden de los `Update` (el de los objetos en el archivo), y que dos triggers
  kinematic se toquen (observado). Si un trigger kinematic y un colisionador estático se tocaban
  no se sabe; en este juego no cambiaría nada, así que el port sigue a Box2D (no se tocan).
- **`Instantiate` de un objeto de la escena** copia el objeto como está en el archivo más el estado
  de sus scripts, no el objeto vivo entero: para el enemigo es lo mismo (solo cambia su posición y
  su animación vuelve a empezar, como en Unity).
- **El azar** es el de macroquad, sembrado con la hora, no el de Unity.
- **Ventana**: el lienzo WebGL de 960×600 escalado a la ventana con bandas negras, como el resto del
  museo, sin la pantalla «made with unity» ni la página de la plantilla WebGL.

## Regenerar los assets

```sh
python3 -I tools/il2cpp.py "<carpeta Release>" reference/il2cpp
~/.venvs/museo/bin/python -I tools/extraer.py "<carpeta Release>" <data desempaquetado> <Managed de Unity 5.1>
# por ejemplo: ../../originales/neuro-battle/extraido/Neuro Battle/Release, ../../originales/neuro-battle/data
# y ../../originales/prototipo-2000/linux/_Linux/demo_Data/Managed
```

El `.data` se desempaqueta antes con `tools/unity_assets.py emscripten` (ver PLAN.md). Hacen falta
el SDK de .NET (para compilar los *stubs*) y ffmpeg.

## Pruebas

```sh
node ../../tools/humo/nativo.mjs neuro-battle   # tras cargo build --release
node ../../tools/humo/web.mjs neuro-battle      # tras tools/build_web.sh
```

Siguen los pasos de `humo.json`: título, instrucciones, nivel, moverse y girar, y el zoom; en la
web, en escritorio, móvil horizontal y móvil vertical. Se comprobó aparte:
- `crates/box2d/comparar/comparar.sh`: la escena «neuronas» y las de siempre, idénticas al C++;
- `cargo test -p unity`: `GetButtonUp` y lo de antes;
- contra el original en Chromium: la unión de neuronas con las mismas teclas, el inicio del nivel,
  el game over a los ~5 s y el lugar del puntaje;
- con un cambio temporal (sin enemigos): unir neuronas en cadena, disparar con una de ataque
  unida (el borde destruye la bala), girar y soltar la última.

## Pendientes

- **El sonido** no se probó en hardware real (el servidor no tiene tarjeta de sonido).
- **Los controles táctiles** se probaron solo en emulación de Chromium.
- **No se jugó una partida larga** de principio a fin: las mecánicas se probaron por separado, y
  disparar y soltar neuronas con un cambio temporal sin enemigos.
- **Lo que hace Unity por dentro y no se pudo observar** (el orden de los `Update`, el momento
  exacto del borrado de lo destruido, los triggers kinematic contra estáticos) quedó deducido.
- **REA** no lee los metadatos de IL2CPP compilados de Unity 5.1.1 (los lee `tools/il2cpp.py`) ni
  tiene un ensamblado administrado que importar (`import-managed-reconstruction` no aplica).
