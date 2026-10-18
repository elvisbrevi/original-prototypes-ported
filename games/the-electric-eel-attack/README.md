# The Electric Eel Attack — port a Rust + macroquad

Port de **The Electric Eel Attack** (Elvis Brevi, 2011,
[itch.io](https://elvisbrevi.itch.io/the-electric-eel-attack)), escrito originalmente en C# con
XNA 3.1 y un motor propio, `Engine.dll`. Es un plataformero de acción. Yerko tiene que reunir los
tres mapas de cada nivel para que se abra el vórtice de salida, mientras lo atacan anguilas
eléctricas: los *hunters* patrullan y se electrifican cada cierto tiempo, y los *kamikazes* saltan
desde abajo. Se las puede pisar o disparar con la energía de las pilas (10 disparos por pila).
Tiene cuatro niveles: el campo, una caverna en la que se sale por un lado y se entra por el otro,
la ciudad y una fábrica con un cañón que apunta con su mira. La lógica se reconstruyó por
ingeniería inversa de los dos ensamblados y se tradujo casi línea por línea. Los gráficos, las
fuentes, los mapas, los efectos y los sonidos son los originales.

![título](docs/titulo.png) ![nivel 1](docs/nivel1.png)

![nivel 3](docs/nivel3.png) ![nivel 4](docs/nivel4.png) ![muerte](docs/muerte.png)

## Ejecutar

```sh
cargo run --release -p the-electric-eel-attack
```

En Linux hacen falta las cabeceras de ALSA y OpenGL/X11 (`sudo apt install libasound2-dev`).
Los assets van embebidos en el binario.

### Versión web

```sh
rustup target add wasm32-unknown-unknown
../../tools/build_web.sh the-electric-eel-attack   # genera dist/the-electric-eel-attack
```

El `index.html` sale de la plantilla común de [`crates/museo/web`](../../crates/museo/web). Al
empezar el juego el navegador decodifica los sonidos, y al llegar al nivel 3 la segunda canción,
lo que puede tardar un momento.

### Controles táctiles

<img src="docs/movil.png" align="right" width="180" alt="móvil en vertical">

En móviles y tablets aparecen una cruceta con izquierda y derecha, los botones A (disparo),
B (salto) y R (reiniciar el nivel), y START para empezar. Los dibuja el crate compartido
[`crates/museo`](../../crates/museo), configurado en `src/main.rs`. Se activan si el navegador
indica un puntero táctil (`(pointer: coarse)`) o en cuanto llega el primer toque. En vertical el
juego se coloca arriba y los controles debajo.

<br clear="right">

## Controles

| Tecla | Acción |
|---|---|
| ← / → | Andar |
| X | Saltar (se rebota al pisar una anguila) |
| Z | Disparar (gasta 10 de energía; las pilas la recargan) |
| Enter | Empezar desde el título |
| R | Reiniciar el nivel |
| Esc | Pantalla completa (como en el original: no sale del juego) |

## Cómo se hizo

1. **Descarga**: `The_Electric_Eel_Attack.zip` de itch.io, un despliegue ClickOnce de XNA 3.1
   (versión 1.0.0.13). Los nombres del zip están en CP437 (`cañon`, `montaña`). El ejecutable
   (36 KB) se compiló el **9-mar-2011** a las 02:02 UTC y `Engine.dll` (54 KB) el 8-mar-2011,
   según sus cabeceras PE. Los dos son CIL puro para CLR 2.0, x86.
2. **REA** ([rea.tools](https://rea.tools/), `rea-agents` 6.7.0):
   - `inspect-managed-artifact`: `The Electric Eel Attack.exe`
     (`ev_d5478a582e5bf5d018c37eb2f48f62f7ee3cf54e0ae5f25788c7598b88945959`) y `Engine.dll`
     (`ev_89912de47b9d1175390d36c8dddf19c6e7ae78396bccb45c8df5326e2ceca931`).
   - `inspect-managed-members`: 11 tipos y 72 métodos en el juego
     (`ev_4cee641740877f572c0a5b1281191e0819faecf2fc40621d346cef37880b5fac`), 32 tipos y 196
     métodos en el motor (`ev_02b38c886ecc3f066ff8cbebf4cff4defdc0411180d4c8590aa99ed3834f5f16`).
   - `import-managed-reconstruction`: el C# decompilado de los métodos con cuerpo quedó anclado a
     su hash de IL y su firma, 33 del juego
     (`ev_7d2704337defc75934720079e28fe7d85435d026fde7c5df2f6a699deb327820`) y 129 del motor
     (`ev_6f70f86337fcb9f649d8d64e46ca3dd9a6178afd5c8c81d221427540cfe7bfe2`). El resto son
     constructores implícitos y accesores de propiedades.
3. **ILSpy** (`ilspycmd` 9.1) con las referencias del *XNA Framework Redistributable 3.1* y
   `Engine.dll`. Para comprobar la semántica de XNA que el port copia, también se decompiló
   `SpriteFont` del redistribuible: `InternalDraw` e `InternalMeasure`, con rotación, escala y
   texto volteado. Están en [`crates/xna`](../../crates/xna).
4. **Assets**: [`tools/extraer.sh`](tools/extraer.sh) usa
   [`tools/xnb_extract.py`](../../tools/xnb_extract.py) para las texturas, las tres `SpriteFont`
   y los 17 `SoundEffect` (PCM; se guardan como WAV sin convertir), y copia los cinco mapas XML
   del editor del motor (`Content/Mapas`).
5. **Efectos**: de los trece efectos del motor, solo se ven tres. Sus píxel shaders ps_2_0 se
   desensamblaron con [`tools/ps20_desensamblar.py`](../../tools/ps20_desensamblar.py) y se reescribieron en
   GLSL (`src/motor/shader.rs`). `varios.xnb` tiene siete pasadas, y el orden de los shaders en el
   archivo es el inverso del de las pasadas.
   - `Negative`: `1 − rgb`.
   - `Brightness`: `color × Intensity`, alfa incluido.
   - `TopFade`: `color × v`.

   Ninguno usa el tinte del sprite.

`reference/csharp/` contiene el C# decompilado del juego (`juego/`) y del motor (`Engine/`).

## Correspondencia con el original

| C# | Rust |
|---|---|
| `EAAE.Game1`, `Program` | `main.rs` |
| `EAAE.Control` | `control.rs` |
| `EAAE.Niveles.*` (`Nivel`, `Titulo`, `Nivel1`–`Nivel4`) | `niveles.rs` |
| `Engine.Datos.Escenario` | `motor/escenario.rs` |
| `Engine.Datos.Pantalla` y los campos estáticos de `Escenario` | `motor/mod.rs` (`PantallaXna`, `Globales`) |
| `Engine.Objetos.*` (`Objeto`, `Grafico`, `Plataforma`, `Personaje`, `SistemaParticulas`, `Particula`, `Texto`, `Estado`) | `motor/objeto.rs` |
| `Engine.Fisicas.Fisica` | `motor/fisica.rs` |
| `Engine.Shaders.*` | `motor/shader.rs` |
| `Engine.Audios.Sonido` | `audio.rs` |
| `MathHelper.Clamp`, `Rectangle`, `SpriteBatch.Draw`, `SpriteFont`, `System.Random` | [`crates/xna`](../../crates/xna) |

## Rarezas del original que se conservan

- `Escenario.getObjetoPorNombre` devuelve el primer objeto cuyo nombre **contiene** el texto
  buscado. El juego busca así todo, cada vez («bala1» encuentra «bala1» porque está antes que
  «bala10» en el mapa).
- El mapa del nivel 1 pide el efecto `Negative` para toda la escena, pero `cortina()` llama a
  `desactivarShaders()` en cada paso y nunca se ve. Los efectos de escena que sí se ven son
  `Negative` al morir y `Brightness` ×2 al reunir los tres mapas (eso duplica también el alfa).
- Al morir el juego se pausa y se pone en negativo mientras baja el telón. En los niveles 3 y 4
  el daño de las plataformas electrificadas y de la bola del cañón solo cuenta cuando el sonido
  «dead» del héroe ya suena, un paso después del choque.
- El nivel que pide cambiar de nivel termina su propia actualización después de cargar el
  siguiente. Durante ese paso todavía mueve la cámara a su héroe, y el nuevo nivel se dibuja una
  vez sin haberse actualizado.
- La cámara (`Pantalla`) guarda su matriz y solo la recalcula tras un `reset()`, así que al cargar
  un mapa la posición nueva no se aplica hasta que algo llama a `reset()`.
- Los *hunters* restan 1 a la componente azul de su color en cada paso (como byte, dando la
  vuelta) y están electrificados mientras vale entre 100 y 150.
- Al empujar un bloque se miran siempre las colisiones de «bloque1», sea cual sea el bloque.
- El título «The Electric Eel» alterna entre mezcla aditiva y normal casi en cada fotograma.
- La física usa el cuerpo calculado en el paso anterior, y los `Grafico` calculan el suyo con la
  posición de scroll del paso anterior.
- Los sistemas de partículas se actualizan y se dibujan aunque estén inactivos (las partículas que
  quedan terminan su vida).

## Diferencias con el original

- **Sonido**: cada objeto tiene sus instancias de sonido, como en XNA. macroquad no dice si un
  sonido sigue sonando, así que cada instancia recuerda su duración. Para no tener que decodificar
  audio al cargar un nivel, las instancias se cargan al empezar (dos juegos por sonido, que se
  alternan entre cargas para que lo que suena del nivel anterior termine).
- **Música**: las dos canciones (400 y 213 s a 8 kHz) se decodifican enteras, así que solo se
  tiene cargada la que toca. El cambio al llegar al nivel 3 puede hacer una pausa breve.
- **Aleatoriedad**: las partículas usan una copia de `System.Random` de .NET 2.0 sembrada con el
  reloj. Los barriles del nivel 4 usan `new Random(i)`, igual que en el original, así que caen
  exactamente igual.
- **Ventana**: el juego se dibuja en 640×480 y se escala con bandas negras.
- No se portaron las funciones del editor del motor (guardar escenarios, grilla, datos y cuerpos
  de depuración, cámara manual), los diez efectos que no usa ningún mapa, ni la colisión por
  píxel. Tampoco el ratón, que el control leía pero ningún nivel usaba.
- **Para probar el port**: en nativo, `EEL_NIVEL=n` (de 1 a 4) empieza directamente en el nivel
  n. No existe en el original.

## Regenerar los assets

```sh
tools/extraer.sh "<ruta>/Application Files/The Electric Eel Attack_1_0_0_13/Content"
```

## Pruebas

```sh
node ../../tools/humo/nativo.mjs the-electric-eel-attack   # tras cargo build --release
node ../../tools/humo/web.mjs the-electric-eel-attack      # tras tools/build_web.sh
```

Siguen los pasos de `humo.json`: el título, Enter, y andar, saltar y avanzar en el nivel 1. Se
comprobaron aparte, con `EEL_NIVEL`, recorridos más largos y cambios temporales sin subir:
- los niveles 2 a 4;
- recoger un mapa;
- la muerte (efecto negativo y reinicio);
- reunir los mapas (`Brightness` ×2 y vórtice);
- la cadena de niveles 1 → 2 → 3 → 4 → título al tocar el vórtice.
