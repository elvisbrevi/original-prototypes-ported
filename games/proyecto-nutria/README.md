# Proyecto Nutria (Prototype Version)

**Proyecto Nutria** (Elvis Brevi, [itch.io](https://elvisbrevi.itch.io/proyecto-nutria)) is a
two-player action platformer. A coipo (the Chilean "otter") and a chicken cross a
level of rock platforms, pick up boxes with laser guns and shoot each other. If a bullet hits a
character, he is thrown upwards, spinning and in red. It was made in 2009 with C# and
XNA 3.1 for Windows (compiled on 11-Nov-2009) and is now ported to Rust + macroquad: it plays
on PC and in the browser, also on mobile. The logic was reconstructed by reverse-engineering the
published executable and translated almost line by line. The graphics, font and pause effect
are the originals, extracted from the `.xnb` files.

![level 1](docs/nivel1.png)

![shot](docs/disparo.png) ![pause](docs/pausa.png)

## Running

```sh
cargo run --release -p proyecto-nutria
```

On Linux you need the ALSA and OpenGL/X11 headers (`sudo apt install libasound2-dev`).
The assets are embedded in the binary. The original game has no sound.

### Web version

```sh
rustup target add wasm32-unknown-unknown
../../tools/build_web.sh proyecto-nutria   # generates dist/proyecto-nutria
```

The `index.html` comes from the common template in [`crates/museo/web`](../../crates/museo/web). On the
web, Esc does nothing (there is no window to close).

### Touch controls

<img src="docs/movil.png" align="right" width="180" alt="mobile in portrait">

On phones and tablets a D-pad, the A (shoot) and B (jump) buttons and START (pause) appear.
They are drawn by the shared crate [`crates/museo`](../../crates/museo), configured in `src/main.rs`.
They are enabled if the browser reports a touch pointer (`(pointer: coarse)`) or as soon as the
first touch arrives. In portrait the game sits at the top and the controls below. In landscape, since the
game fills the screen (16:9), the controls sit on top of the frame.

<br clear="right">

## Controls

| Key | Action |
|---|---|
| ← / → | Run (accelerates up to 6 px per step and slides when released) |
| X | Jump (button "B"; hold to jump up to 250 px) |
| Z | Shoot with the gun (button "A"; 10 bullets per box) / select menu option |
| Enter | Pause: CONTINUE, RESTART, EXIT |
| ↑ / ↓ | Move through the menu |
| Esc | Quit |

With a single keyboard you control the coipo. The chicken only moved with a DirectInput gamepad, which was not
ported (see below), so it stays still on its platform as on a PC without a gamepad, but it does take
the shots.

## How it was made

1. **Download**: `Proyecto_Nutria.rar` from itch.io. It is a ClickOnce deployment of XNA 3.1:
   `Proyecto Nutria.exe` (52 KB, pure CIL, CLR 2.0, x86) and the `Content/` folder. According to the
   PE header it was compiled on **11-Nov-2009** at 12:52 UTC (the ClickOnce manifests are from the same day).
2. **REA** ([rea.tools](https://rea.tools/), `rea-agents` 6.7.0):
   - `inspect-managed-artifact`: PE32 x86 assembly, IL only, complete metadata, version 1.0.0.0
     (`ev_df021cd93d7021058a4e8a709710e50f3b1c383df812627c7166c2abbbbdf3d7`).
   - `inspect-managed-members`: 42 types, 155 methods, 234 fields and the strings the code loads
     (content paths, weapon names, menu options, `..\..\..\Datos\Save.xml`)
     (`ev_2c96271246856c5b3e62c535b736e8b44296dc9729b428bb2b71b0c864faea9c`).
   - `import-managed-reconstruction`: the decompiled C# of the 104 methods with a body was anchored
     to the exact IL hash and the signature of each method
     (`ev_6eec8dd8725bcdb315c64fae70749ec7bfb500226ea0d9b3b4b07892673e71e3`). The remaining 31 are
     implicit constructors and auto-property accessors, with no code of their own.
3. **ILSpy** (`ilspycmd` 9.1) to get readable C#, with the references of Microsoft's *XNA Framework
   Redistributable 3.1*. The REA payload is assembled with
   [`tools/rea_reconstruction.py`](../../tools/rea_reconstruction.py).
4. **Assets**: [`tools/xnb_extract.py`](../../tools/xnb_extract.py) (IMAN's extractor
   generalised) converts the `Color` textures and the `SpriteFont` to PNG, only the ones the game uses
   (list in `tools/contenido.txt`).
5. **Pause effect**: `Shaders/blur_grayscale.xnb` does not bring the source code, so its pixel
   shader ps_2_0 was disassembled with [`tools/ps20_desensamblar.py`](../../tools/ps20_desensamblar.py) and rewritten in GLSL
   (`src/shader.rs`): it adds 12 samples from a Poisson disk of radius 0.005 (in the coordinates of
   each texture, without the centre sample), converts to grey with `(r + g + b) / 3` and divides by 12. It leaves
   as alpha the sum of the 12 alphas, and it does not use the sprite's tint.
6. **Port**:
   - As in IMAN, each class is a module and the fixed step is `museo::PasoFijo::xna()`.
     `crates/xna` provides `Rectangle`, `Clamp`, `SpriteBatch.Draw`, `SpriteFont` and
     `System.Random`. `tiempo.rs` copies the fixed-step `GameTime`, in which `TotalGameTime` adds
     the step before each `Update`.
   - `Game1.Draw` draws in two parts. The frame and the icons go on the 1280×720 virtual screen.
     The level goes in a render target the size of the `areaJuego` viewport, cleared in
     black and with the `Camera2D` transformation (scale and then translation of
     −position × scale). That render target is copied to the screen without blending
     (`Pantalla::copiar_opaco`), so that the semi-transparent sprites do not darken it.
   - In the pause, whatever is drawn with the camera (everything except the sky) uses the effect's
     material instead of the sprite one, like the original's `SpriteBatch.Begin` with an effect.
   - The particles use `xna::Random`, a copy of .NET 2.0's `System.Random` (Knuth's
     subtractive generator), seeded with the clock like `new Random()`.
   - Without a gamepad, the original creates a single control device, the keyboard. The port does the same, and the
     touch buttons are read as their keys.

`reference/csharp/` contains the decompiled C# as a reference to compare against the port.

## Correspondence with the original

| C# (XNA) | Rust |
|---|---|
| `Game1`, `Program` | `main.rs` (with `museo::PasoFijo` and `museo::Pantalla`) |
| `Camara`, `Camera2D` | `camara.rs` |
| `Control`, `Teclado` | `control.rs` |
| `Nivel`, `Nivel_1` | `nivel.rs` |
| `Personaje`, `Coipo`, `Pollo` | `personaje.rs` |
| `Fisica` | `fisica.rs` |
| `Plataforma` | `plataforma.rs` |
| `Caja`, `Arma`, `Pistola`, `Bala` | `caja.rs`, `arma.rs` |
| `Generador`, `Explosion`, `Particula` | `particulas.rs` |
| `MenuPersonajes`, `Menu` | `menu.rs` |
| `Iconos.IconoPersonajes` | `iconos.rs` |
| `Shaders.Blur_Grayscale` | `shader.rs` |
| `MathHelper.Clamp`, `Rectangle`, `SpriteBatch.Draw`, `SpriteFont`, `System.Random` | [`crates/xna`](../../crates/xna) |

## Original quirks that are kept

- `MenuPersonajes` is not a menu to choose a character, but the pause menu. Its faces are loaded but
  never drawn, and the **EXIT option does not exit**: it restarts like RESTART, only without
  moving the camera.
- On restart, the coipo is the same object and **keeps the gun** and whatever bullets it has left. The
  chicken, however, is created again. The boxes already collected stay as they were.
- Only the icon of the characters with a control device is updated, so **the chicken's health bar
  is never seen** (its rectangle stays 0 wide).
- The bullets never disappear: they keep flying outside the level.
- Gravity only goes back to 0 when jumping. While the character is on the ground it keeps falling 9 px
  per step and the collision returns it to the surface. When the jump is released the character switches to "falling"
  on every step.
- To the right the character moves forward even when hitting a wall; to the left, it does not.
- `MathHelper.Clamp` applies the maximum first, which fixes the position after a collision below.
- The camera frames both characters (pulling back up to scale 0.38) and follows the one furthest to the
  left with a delay of 6 px per step. The position (500, 500) that `Game1` gives it is lost
  because `Camera2D.Initialize` resets it. RESTART sets its scale to 0.36 for one frame.
- The crop of the `scroll1` background is 150 px taller than the texture. `SpriteBatch`'s Clamp mode
  stretches its last row, and when the camera pulls back some vertical stripes are visible under the field.
- The dust the character kicks up while running adds the total game time (not the step's) to its timer.
  The first particle looks at the last one through an
  `IndexOutOfRangeException` that is caught. Alpha subtracts like a C# byte.
- `Nivel_1` creates a coipo and a chicken (and by mistake places the coipo twice), but the menu
  immediately replaces them with the coipo at (0, 280) and the chicken at (2600, 800).

## Differences from the original

- **Gamepad**: the original used DirectInput (`JoystickGenerico`) and with a gamepad connected the
  second character became playable. This port only supports a keyboard (and touch controls),
  like the original on a PC without a gamepad.
- **Randomness**: the particles use a copy of .NET 2.0's `System.Random` seeded with the
  clock, like `new Random()`; the sequences do not match any particular run of the original.
- **Window**: the game is drawn at 1280×720 and scaled to the window with black letterboxing. The
  original asked for 1280×720 in a window.
- Code the original never uses on screen was not ported: the stopwatch (it is updated but not
  drawn, which is why its 8 MB font was not extracted either), the coins (the level's list is always
  empty; the counter itself is there), the flamethrower (no box creates it), the `Filter` and
  `Negative` effects, saving to `Datos\Save.xml`, the manual debug camera and the `Enemigo` class
  (empty).

## Regenerating the assets

```sh
python3 -I ../../tools/xnb_extract.py "<path>/Application Files/Proyecto Nutria_1_0_0_0/Content" assets tools/contenido.txt
```

## Tests

```sh
node ../../tools/humo/nativo.mjs proyecto-nutria   # after cargo build --release
node ../../tools/humo/web.mjs proyecto-nutria      # after tools/build_web.sh
```

They follow the steps in `humo.json`: run to the first box, shoot, jump, pause and
continue. A bullet hitting the chicken was checked separately, going down to its platform.
