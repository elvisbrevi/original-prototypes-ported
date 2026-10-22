# IMAN (Prototype)

**IMAN** (Elvis Brevi, [itch.io](https://elvisbrevi.itch.io/iman)) is a platformer in which
a boy uses the magnet from his backpack to pull boxes and to lift himself onto moving platforms.
It has two levels and an ending. It was made in 2009 with C# and XNA 3.1 for Windows (compiled on
13-Nov-2009) and is now ported to Rust + macroquad: it plays on PC and in the browser, also on
mobile. The logic was reconstructed by reverse-engineering the published executable and translated
almost line by line. The graphics, font, shader and sound are the originals, extracted
from the `.xnb` / `.xwb` files.

![menu](docs/menu.png) ![level 1](docs/nivel1.png)

## Running

```sh
cargo run --release -p iman
```

On Linux you need the ALSA and OpenGL/X11 headers (`sudo apt install libasound2-dev`).
The assets are embedded in the binary, so it can be run from any folder.

### Web version

```sh
rustup target add wasm32-unknown-unknown
../../tools/build_web.sh iman   # generates dist/iman (index.html + mq_js_bundle.js + iman.wasm)
butler push ../../dist/iman elvisbrevi/imn-2026:html5   # publishing test of the port
```

The `index.html` comes from the common template in [`crates/museo/web`](../../crates/museo/web), with the
title from `[package.metadata.museo]` in `Cargo.toml`. On the web, Esc and the "Quit" option do nothing (there is no window to close).

### Touch controls

On phones and tablets a D-pad, the A (magnet) and B (jump) buttons and START appear. They are drawn by
the shared crate [`crates/museo`](../../crates/museo), configured in `src/main.rs`: they are enabled if the
browser reports a touch pointer (`(pointer: coarse)`) or as soon as the first touch arrives. They support
several fingers at once (walking and jumping). In portrait the game sits at the top and the controls below.

## Controls

| Key | Action |
|---|---|
| ← / → | Move |
| X | Jump (button "B"; hold to jump higher) |
| Z | Use the magnet (button "A") / select menu option |
| ↑ | Aim the magnet upwards |
| ↑ / ↓ | Move through the menu |
| Enter | Pause |
| F4 | Fullscreen |
| Esc | Quit |

The magnet comes out of the backpack, so it points to the opposite side of where the boy is facing. It pulls the
boxes/barrels towards the boy and, aimed upwards, it lifts him towards a moving platform.

## How it was made

1. **Download**: `IMAN_(Prototipo).rar` from itch.io. It is a ClickOnce deployment of XNA 3.1:
   `Iman.exe` (43 KB, pure CIL, CLR 2.0) and the `Content/` folder.
2. **REA** ([rea.tools](https://rea.tools/), `rea-agents` 6.5.0):
   - `inspect-managed-artifact`: PE32 x86 assembly, IL only, complete metadata.
   - `inspect-managed-members`: 33 types, 160 methods, 214 fields, 28 strings (content paths,
     shader names, menu texts).
   - `import-managed-reconstruction`: the decompiled C# of the 75 methods with logic was anchored
     to the exact IL hash and signature of each method
     (`ev_407cda2868622f5d8af545932c3621da2277a883ae85946f3e3e2955380c71a0`).
3. **ILSpy** (`ilspycmd` 9.1) to get readable C#, resolving references against the
   assemblies of Microsoft's *XNA Framework Redistributable 3.1*. That reference was also used
   to check the XNA semantics the port has to copy:
   - `SpriteSortMode.Immediate` is 0 in XNA 3.1, so the title shader **is** applied.
   - `MathHelper.Clamp` applies the maximum first and then the minimum; the physics depends on it.
   - `SpriteBatch` flips only the UVs, filters linearly with clamp and blends non-premultiplied alpha.
   - `Game` uses a fixed step of 166667 ticks (~60 Hz); all physics runs in pixels per step.
   - `SpriteFont.InternalDraw` algorithm (kerning, clipping and spacing).
4. **Assets**: `tools/xnb_extract.py` reads the uncompressed XNB v4 files (`Color` and DXT3 textures), the
   `SpriteFont` (glyphs, clipping, kerning) and the XACT wave bank (8-bit PCM at 8 kHz, which is
   resampled to 44.1 kHz because quad-snd does not resample).
5. **Shader**: `PostprocessEffect.xnb` is the effect from XNA's *Non-Photorealistic* sample; the
   default values were read from the binary (`SketchThreshold = 0.1`, `SketchBrightness = 0.333`)
   and the `ColorSketch` technique was rewritten in GLSL.
6. **Port**:
   - Every C# class is a Rust module (see the table below), with the same fields and the
     same order of operations. The `(int)` casts truncate towards zero as in C#
     (`xna::int`), and `xna::clamp` applies the maximum first, like `MathHelper.Clamp`.
   - The `Game` loop with a fixed step is `museo::PasoFijo::xna()`. Each frame runs the
     `Update`s for as many 166,667-tick steps as needed (with at most 500 ms of lag, like `Game.Tick`)
     and a single `Draw`.
   - Everything is drawn on a 640×480 virtual screen (`museo::Pantalla`, a render target) that
     is scaled to the window with letterboxing. `xna::dibujar` reproduces `SpriteBatch.Draw`: the origin in
     source pixels, the scale and rotation around it, and flipping only the UVs.
     `xna::SpriteFont` reproduces XNA's `InternalDraw`.
   - The shader is a macroquad material with the stroke texture. `SketchJitter` takes a value
     at random every 0.05 s (500,000 ticks), as in `Shaders.NonPhotoRealistic`.
   - `control.rs` copies `PCInput`: "pressed on this step" and "held". The touch buttons of
     `museo::Tactil` are read the same way as the keys.

`reference/csharp/` contains the decompiled C# as a reference to compare against the port.

## Correspondence with the original

| C# (XNA) | Rust |
|---|---|
| `Game1`, `Program`, `Camaras.Camera2D` | `main.rs` (with `museo::PasoFijo` and `museo::Pantalla`), `escenarios::Camara` |
| `Controles.PCInput` / `Control` | `control.rs` |
| `Objetos.Objeto`, `Plataforma`, `Niño` | `objetos.rs` |
| `Fisicas.Fisica` | `fisica.rs` |
| `Escenarios.Escenario`, `Escenario_01`, `Escenario_02` | `escenarios.rs` |
| `Graficos.*` (Cielo, Background, Suelo, Front, Ramas) | `graficos.rs` |
| `Menus.Menu` | `menu.rs` |
| `Shaders.NonPhotoRealistic` | `shader.rs` |
| `MathHelper`, `Rectangle`, `SpriteFont`, colours | [`crates/xna`](../../crates/xna) |

The original's quirks are also kept, because they are part of how it plays: the boy and the
boxes end up a few pixels sunk into the ground (gravity is never reset on landing), the
inverted `Clamp` in the collisions, and the credits becoming active (but invisible) when level 2
starts.

## Differences from the original

- **Gamepad**: the original used DirectInput (`JoystickNormal`); this port only supports a keyboard.
- **F4**: the original toggled fullscreen on every step while F4 was held; here it
  toggles once per press.
- **Window**: the game is drawn at 640×480 and scaled to the window with black letterboxing.
- **Audio**: the XACT engine is replaced by direct playback of the single wave (`jump`).
- Code the original never uses was not ported: the manual debug camera (disabled in
  `Initialize`), `crearSubida`/`crearBajada`, pixel-perfect collision (`IntersectPixels`) and mouse
  input.

## Regenerating the assets

```sh
python3 -I tools/xnb_extract.py "<path>/Application Files/Iman_1_0_0_0/Content" assets
```
