# Super Happy Jump

**Super Happy Jump** (Elvis Brevi, [itch.io](https://elvisbrevi.itch.io/super-happy-jump)) es un
juego de saltos infinito: «double jump, cactus and coins». Un vaquero con pañuelo rojo corre solo
hacia la derecha; se toca para saltar y se vuelve a tocar en el aire para el doble salto. Cada 2,5
segundos aparece una pareja de cactus con una moneda en el hueco: la moneda suma un punto y tocar
un cactus termina la partida. Al final se ve el puntaje y el mejor, que queda guardado.

Se hizo con Unity 4.6 para el Unity Web Player (también salió en iOS y Android): la versión web se
compiló el 9-dic-2014. Ahora está portado a Rust + macroquad y se juega en PC y en el navegador,
también en móviles, tocando la pantalla. La lógica se tradujo del C# decompilado de
`Assembly-CSharp.dll`, y las piezas del motor que usa (la física 2D con Box2D, el Animator, las
partículas, la GUI de `OnGUI`) se recrearon en [`crates/unity`](../../crates/unity) a partir de
los datos del propio juego. Las imágenes, los sonidos, la escena y los prefabs son los originales:
el port lee al arrancar el `escena.json` que se extrae del `web.unity3d`.

![título](docs/titulo.png) ![carrera](docs/carrera.png)

![salto](docs/salto.png) ![récord](docs/record.png)

## Ejecutar

```sh
cargo run --release -p super-happy-jump
```

En Linux hacen falta las cabeceras de ALSA y OpenGL/X11 (`sudo apt install libasound2-dev`).
Los assets van embebidos en el binario. El mejor puntaje se guarda en
`~/.local/share/museo/super-happy-jump.txt` (o en la carpeta de datos del sistema).

### Versión web

```sh
rustup target add wasm32-unknown-unknown
../../tools/build_web.sh super-happy-jump   # genera dist/super-happy-jump
```

El `index.html` sale de la plantilla común de [`crates/museo/web`](../../crates/museo/web). En la
web el mejor puntaje va al `localStorage` del navegador, como los `PlayerPrefs` del Web Player.

### Controles táctiles

<img src="docs/movil.png" align="right" width="320" alt="móvil en horizontal">

Como en el original, no hay botones en pantalla: en móviles y tablets el toque hace de clic, para
saltar y para el botón ▶. Se juega en horizontal; en vertical queda centrado.

<br clear="right">

## Controles

| Entrada | Acción |
|---|---|
| Clic o toque en ▶ | Empezar (y, en la pantalla final, volver al título) |
| Clic o toque | Saltar; en el aire, doble salto |

## Cómo se hizo

1. **Descarga**: el `web.unity3d` que itch.io servía al Unity Web Player (385 194 bytes), con la
   URL firmada de la página, comprobado contra el sha256 de `originales/catalogo.json`.
2. **Triaje**:
   - Es un bundle `UnityWeb` 3.x.x de Unity **4.6.0f3** comprimido con LZMA. Trae
     `Assembly-CSharp.dll` (39 936 bytes), `UnityEngine.UI.dll` (de Unity, del 18-nov-2014) y dos
     archivos serializados: `mainData` (271 objetos, con la única escena,
     `assets/_Scenes/test.unity`) y `sharedassets0.assets` (199 objetos: texturas, sprites, clips,
     controladores, prefabs, la piel de la GUI). Según su cabecera PE, `Assembly-CSharp.dll` se
     compiló el **9-dic-2014** a las 22:52 UTC.
   - REA ([rea.tools](https://rea.tools/), `rea-agents` 6.7.0) no abre el formato `UnityWeb`
     (`inspect-artifact`: «unsupported binary format»); el bundle se abrió con UnityPy.
   - `inspect-managed-artifact`: PE32 solo IL, metadatos completos, runtime `unity-mono`
     (`ev_f64054c153ce0f926872fcbcae608201878c960f9a54726ab4f1eaca838eb297`, sobre
     `ev_3c17806a16032c024991e4c9eccbd52ce2e79b36ea2ebf060c3afaf6bbf671ad`).
   - `inspect-managed-members`: 58 tipos, 218 métodos, 263 campos y las cadenas que carga el código
     (`BestScore`, los nombres de los estilos, `Particulas`, `Suelo`, `Game Over`...)
     (`ev_655d6009500d0e8b5f73e706945778c26b5334e3d8fe150ba9c762b9ab5e0c53`).
   - `import-managed-reconstruction`: el C# decompilado de 132 métodos quedó anclado al hash de su
     IL y su firma (`ev_5abb5b691908474fe84da9149e93692c15bf5eca035359224a248a88675cdf52`). Los
     demás son constructores que solo inicializan campos y las clases de las corrutinas.
3. **ILSpy** (`ilspycmd` 9.1): el `UnityEngine.dll` de Unity 4.6 no viene en el bundle (es parte
   del Web Player), así que las referencias se resolvieron con el de Unity 5.1.2 de la versión de
   Linux de [One Button Prototype](https://elvisbrevi.itch.io/prototipo-2000) y sus bibliotecas de
   Mono 2.0; las llamadas que usa el juego son las mismas en 4.6. El payload de REA se armó con
   [`tools/rea_reconstruction.py`](../../tools/rea_reconstruction.py). De los 41 archivos que da
   ILSpy, 12 son del juego y están en `reference/csharp`: diez del autor y
   `PlatformerCharacter2D` y `Platformer2DUserControl`, del ejemplo 2D de Unity, modificadas
   (polvo, chispas, sonidos, doble salto, toque para saltar). Los demás son de terceros y no se
   guardan: el CrossPlatformInput de los Standard Assets y `Camera2DFollow` y `Restarter` del
   ejemplo 2D, sin uso en la escena, y los scripts de Cartoon FX, de los que la escena solo usa
   `CFX_AutoDestructShuriken`.
4. **Assets** ([`tools/extraer.py`](tools/extraer.py), con UnityPy): los dos archivos
   serializados traen los type trees, así que los campos de los MonoBehaviour se leen sin
   generarlos. Extrae:
   - las 17 texturas que se ven, decodificadas a PNG tal como las guardó Unity (DXT5 la hoja del
     personaje, las monedas y el cartel; ARGB4444 las de partículas);
   - los cinco `AudioClip` (WAV PCM);
   - y `escena.json`: los 69 GameObjects de la escena con su jerarquía y componentes; los prefabs
     que instancia el código (`muro`, `polvo` y `dobleSalto`); los 37 sprites (rectángulo, pivote
     sacado de los vértices de su malla y píxeles por unidad); los cinco clips de Mecanim, cuyo
     *streamed clip* se decodifica a claves (tiempo, sprite); los dos controladores; los tres
     sistemas de partículas; los materiales; los estilos de la `GUISkin`; el `PhysicsMaterial2D`
     «Slippery» (sin fricción); y los ajustes de tiempo, física 2D y sorting layers.

   La fuente «Arial Rounded Bold» que el bundle trae incrustada no se extrae (ver
   [Diferencias](#diferencias-con-el-original)).
5. **El motor** ([`crates/unity`](../../crates/unity)), con lo que no se ve leyendo el C#:
   - `Time`: `deltaTime` es el tiempo real del frame (como mucho el *Maximum Allowed Timestep*,
     1/3 s) por `timeScale`; los `FixedUpdate` van a paso fijo de 0,02 s detrás de `Time.time`.
     Con `timeScale = 0` no corren `FixedUpdate`, ni `Invoke`, ni el Animator, ni las partículas,
     pero sí `Update` y `OnGUI`.
   - Física 2D sobre [`crates/box2d`](../../crates/box2d):
     - un cuerpo de Box2D por `Rigidbody2D`, y cada `Collider2D` es una fixture del
       `Rigidbody2D` más cercano hacia arriba en la jerarquía: el polvo que el código cuelga del
       personaje pasa a ser parte de su cuerpo, y vuelve a ser estático al soltarlo;
     - los colisionadores escalan con `lossyScale` (las cajas con su centro, con
       `SetAsBox`; los círculos, con la mayor escala), tienen densidad 1 y después
       `Rigidbody2D.mass` fija la masa con `SetMassData`. Sin material, la fricción es 0,4;
     - los triggers son sensores de Box2D, y sus `BeginContact` son los `OnTriggerEnter2D`, que
       reciben los scripts de los dos objetos;
     - si el código mueve un Transform con cuerpo, el cuerpo se teletransporta (`SetTransform`) y
       se despierta. Sin eso, el `Destroyer`, un cuerpo dinámico quieto que se mueve con la cámara,
       se dormiría y no vería el suelo que tiene que pasar adelante. Después de cada paso, los
       cuerpos escriben su posición en su Transform;
     - `Physics2D.OverlapCircle` es un `QueryAABB` más `b2TestOverlap`, filtrado por capas.

     Para esto `crates/box2d` ganó los sensores, los eventos de contacto por fixture,
     `QueryAABB`, `DestroyFixture`, `ApplyForceToCenter` y `SetMassData`, traducidos del mismo
     Box2D 2.3.1. La escena nueva «corredor» de
     [`comparar.sh`](../../crates/box2d/comparar/comparar.sh) (un corredor con círculo y caja,
     saltos por fuerza, cactus, moneda, polvo y el `Destroyer`) da trayectorias, eventos y
     consultas idénticos bit a bit al C++ durante 300 pasos.
   - Animator (Mecanim): estados con un clip de sprites, parámetros y transiciones con
     condiciones, primero las de *Any State* y sin interrumpirse (son atómicas). En Unity 4 la
     duración de una transición está en tiempo normalizado del estado de origen, así que las de
     este juego duran milésimas: 0,1 de un clip de 1/60 s.
   - ParticleSystem (Shuriken): los módulos que usan los tres sistemas (inicial, forma de esfera y
     semiesfera, emisión con ráfagas, tamaño y color en la vida, hoja de 4 cuadros y velocidad en
     la vida), con las curvas de Hermite y los gradientes de Unity. Como en Unity 4, la
     simulación local (*moveWithTransform*) sigue al emisor e ignora la escala de su Transform:
     el `dobleSalto` tiene escala 0,5 y sus chispas se ven a tamaño completo.
   - La cámara es de perspectiva (48° de campo vertical, en z = −10); todo está en z = 0, así que
     1 unidad son 67,4 píxeles. La cola de dibujo se ordena por sorting layer, *order in layer* y
     de lejos a cerca. Las partículas usan los shaders `Particles/Additive`,
     `Particles/Alpha Blended` y `Particles/Multiply`, reescritos en GLSL.
   - IMGUI: `OnGUI` corre una vez por evento del frame (MouseDown, MouseUp y Repaint); un botón
     se aprieta con MouseDown y responde al soltarlo dentro; los estados *hover* y *active* del
     estilo usan su propio fondo, y si no tienen, no se dibuja ninguno. `GUI.Window` dibuja encima
     del resto, con coordenadas relativas y recortando su contenido. El texto se coloca con las
     métricas verticales de FreeType (el ascenso y el alto de línea con el *line gap*), las mismas
     que Unity guardó para la fuente incrustada.
   - `PlayerPrefs` es `museo::Preferencias`: un archivo en PC y `localStorage` en la web.
6. **Los scripts** (`src/juego.rs`): cada uno es una estructura con sus campos; los `static`
   (`CameraRunner.x`, `Spawn.activo`, `HUD.estadoStatic`) viven fuera de la escena y sobreviven a
   `Application.LoadLevel`, que vuelve a cargar la escena al empezar el frame siguiente. El frame
   sigue el orden de Unity:
   1. los `FixedUpdate` (`PlatformerCharacter2D` y después `Platformer2DUserControl`), cada uno
      seguido del paso de la física y sus triggers (`Muro`, `PowerUp`, `Destroyer`);
   2. los `Update` en el orden de los objetos (`CameraRunner`, `HUD` y el control del jugador);
   3. el `Invoke` de `Spawn` y la corrutina de `CFX_AutoDestructShuriken`;
   4. el Animator y las partículas, y el borrado de lo destruido;
   5. el dibujo y al final `HUD.OnGUI`.

   `TimControllerScript`, otro control del personaje (saltaba con la barra espaciadora), está
   desactivado en la escena y no se traduce.
7. **Fuentes**: [`tools/fuentes.sh`](tools/fuentes.sh) baja Nunito de google/fonts y la instancia
   con fontTools en los pesos 800 y 900.

## Correspondencia con el original

| Unity 4.6 | Rust |
|---|---|
| `web.unity3d` (`mainData`, `sharedassets0.assets`) | `assets/` ([`tools/extraer.py`](tools/extraer.py)) |
| `Assembly-CSharp.dll` | `src/juego.rs` |
| `Time`, GameObject, Transform, `Instantiate`, `Destroy` | `crates/unity` (`tiempo.rs`, `escena.rs`) |
| Física 2D (Box2D) | `crates/unity` (`fisica2d.rs`) y `crates/box2d` |
| Animator (Mecanim) | `crates/unity` (`animator.rs`) |
| ParticleSystem (Shuriken) | `crates/unity` (`particulas.rs`) |
| Camera, SpriteRenderer, ParticleSystemRenderer | `crates/unity` (`render.rs`) |
| `OnGUI` y la `GUISkin` | `crates/unity` (`gui.rs`) |
| AudioSource | `crates/unity` (`audio.rs`) |
| `PlayerPrefs` | `museo::Preferencias` |
| El Web Player de 960×600 | `src/main.rs` con `museo::Pantalla` |

## Rarezas del original que se conservan

- **Para volver a jugar hay que pasar por el título.** El botón ▶ de la pantalla final recarga la
  escena, y la escena empieza en el título con el tiempo detenido: son dos toques.
- **El botón ▶ desaparece con el ratón encima y mientras se aprieta.** Su estilo solo tiene fondo
  en el estado normal. Tras volver al título, el ratón sigue sobre el botón y no se ve hasta
  apartarlo. En el móvil desaparece mientras se toca.
- En el título el personaje flota un poco sobre el suelo: la escena empieza congelada antes de su
  primera caída, y al empezar cae unos centímetros.
- **El triple salto.** El círculo que detecta el suelo (radio 0,2, a 0,65 bajo el centro) lo sigue
  tocando durante los dos primeros pasos de física del salto, unos 40 ms: un segundo toque así de
  rápido cuenta como salto desde el suelo y el doble salto queda disponible.
- **Los pasos suenan en bucle, y en el aire zumban.** En cada paso de física sin suelo el código
  llama a `Play` del sonido de los pasos, que vuelve a empezar: en el aire se oyen solo sus
  primeros 20 ms una y otra vez, y en el suelo el bucle sigue. Empieza con la primera caída y no
  para hasta la pantalla final, también con el personaje muerto.
- **El polvo se queda atrás.** Al saltar, la nube de polvo que va bajo los pies se suelta y queda
  donde estaba, emitiendo. Hasta que el `Destroyer` (que va 10,8 unidades detrás) no la borra, no
  aparece otra: durante alrededor de un segundo se corre sin polvo. El `Destroyer` busca la etiqueta
  «Particulas», pero la del polvo es «Particula»; la borra igual porque, suelta, no tiene padre.
- El efecto del salto tiene dos partes: las chispas blancas y un «Black Hit» oscuro
  (`Particles/Multiply`). El código pone la primera en la sorting layer «Player», pero la segunda
  se queda en «Default», la de más atrás, y el cielo la tapa: nunca se ve.
- **El temblor de la cámara depende de los frames.** Al morir, la cámara tiembla 25 frames
  (más corto en pantallas rápidas), y el desplazamiento con que termina queda en un campo
  `static`: en la partida siguiente el temblor se centra en ese valor.
- Muerto, el personaje se queda quieto, pero si murió en el aire un toque todavía deja en cero su
  velocidad vertical (el salto solo mira la velocidad máxima para aplicar la fuerza).
- Los cactus salen cada 2,5 s justos (el mínimo y el máximo del azar son iguales) y a una altura
  al azar. Tras morir sale uno más, fuera de la pantalla: el `Invoke` ya estaba pedido.
- En la pantalla final la cinta de «Game Over» es más ancha que la ventana y queda recortada por
  ella.
- Una moneda recogida se oculta pero sigue ahí; si la toca el polvo o el `Destroyer`, el script
  lanza una excepción (`&` sin cortocircuito y un `GetComponent` nulo) y no pasa nada más.
- El mejor puntaje se guarda en cuanto se ve la pantalla final con un récord.

## Diferencias con el original

- **Fuentes**:
  - El original usaba la Arial del sistema para el puntaje y «Game Over»: el port usa Liberation
    Sans 2.1.5 (SIL OFL 1.1), con las métricas de Arial, de
    [`crates/construct2/fuentes`](../../crates/construct2/fuentes).
  - Para «Score», «Best» y «New Record!» el juego trae incrustada Arial Rounded MT Bold, de
    Monotype, que no se puede redistribuir: el port usa Nunito (SIL OFL 1.1, en `fuentes/`) en
    peso 800, y 900 donde el estilo pedía negrita. El texto se coloca con el ascenso y el
    interlineado que Unity guardó de la original.
- **Box2D**: Unity 4.6 traía su propio Box2D compilado en el Web Player, de versión no publicada;
  el port usa `crates/box2d` (2.3.1). Unity agrupa los colisionadores sin `Rigidbody2D` en un
  cuerpo estático; aquí cada objeto tiene el suyo.
- **Lo que hace Unity por dentro** se dedujo de su documentación y de su comportamiento conocido,
  porque el motor es nativo y el Web Player ya no corre en los navegadores: que un Transform
  movido por código despierte al cuerpo, que un estado de estilo sin fondo no dibuje nada, que en
  una transición de Mecanim se vea el sprite del estado que pesa más, cómo se estira una
  partícula *stretched* (aquí, centrada en su posición).
- **Azar**: el orden de las tiradas es el del código, pero el generador es el de macroquad, no el
  de Unity (que tampoco es el de las partículas).
- **Sonido**: los WAV originales. Cada clip suena una vez a la vez; en el juego ningún clip lo
  usan dos fuentes al mismo tiempo.
- **Ventana**: el Web Player era de 960×600 fijos; el port escala esa pantalla a la ventana con
  bandas negras, como el resto del museo.
- No se dibujan los dos quads con el material `Default-Diffuse` (`GroundSpawn` y `Destroyer`),
  que en 960×600 quedan fuera de la vista, ni el plano `ScreenshotView`, que está detrás de la
  cámara.

## Regenerar los assets

```sh
~/.venvs/museo/bin/python -I tools/extraer.py <web.unity3d>   # por ejemplo ../../originales/super-happy-jump/web.unity3d
tools/fuentes.sh                                              # Nunito, desde google/fonts
```

## Pruebas

```sh
node ../../tools/humo/nativo.mjs super-happy-jump   # tras cargo build --release
node ../../tools/humo/web.mjs super-happy-jump      # tras tools/build_web.sh
```

Siguen los pasos de `humo.json`: el título, el toque en ▶, la carrera, un salto y la pantalla
final al chocar con un cactus. En la web se prueban escritorio, móvil horizontal y móvil vertical.
Se comprobó aparte:
- `crates/box2d/comparar/comparar.sh`: las escenas de siempre y la nueva «corredor», idénticas
  al C++ de Box2D 2.3.1;
- `cargo test -p unity`: las curvas, los gradientes y las transiciones del Animator;
- con un cambio temporal sin subir (las dos primeras monedas a la altura del corredor), tomando
  la captura `record.png`: las monedas suman y se ocultan, la pantalla de récord, el mejor
  puntaje guardado (`BestScore=2`), el regreso al título con el botón ▶ oculto bajo el ratón, y
  la segunda partida, que termina con «Score» y «Best».

No se pudo comparar con el original en marcha: el Unity Web Player ya no existe en los
navegadores actuales.
