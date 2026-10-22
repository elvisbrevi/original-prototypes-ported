# Que No Caiga

**Que No Caiga** (Elvis Brevi, [itch.io](https://elvisbrevi.itch.io/que-no-caiga)) is a game with
a World Cup theme (the tags say *brasil*, *chile*, *mundial*). You have to keep the
ball in the air: every touch makes the player kick, and if the ball goes through the area at his feet the
run is over. It counts the seconds you last. At second 9 and at 40 another ball appears. From
second 50 bombs fall: kicked ones shine and explode in the air, but if they reach your feet they explode there and
burn the player.

It was made with Construct 2 for the browser (exported on 23-Jul-2015) and is now ported to
Rust + macroquad: it plays on PC and in the browser, also on mobile. The logic was translated from the
project's event sheets (`data.js`) and from the minified Construct 2 runtime
(`c2runtime.js`). The images, sounds and project (types, animations, collision polygons
and layouts) are the originals: the port reads `data.js` at startup.

![menu](docs/menu.png) ![kick](docs/patada.png) ![bomb](docs/bomba.png) ![ending](docs/final.png)

## Running

```sh
cargo run --release -p que-no-caiga
```

On Linux you need the ALSA and OpenGL/X11 headers (`sudo apt install libasound2-dev`).
The assets are embedded in the binary.

### Web version

```sh
rustup target add wasm32-unknown-unknown
../../tools/build_web.sh que-no-caiga   # generates dist/que-no-caiga
```

The `index.html` comes from the common template in [`crates/museo/web`](../../crates/museo/web).

### Touch controls

<img src="docs/movil.png" align="right" width="180" alt="mobile in portrait">

The original was already played by touching the screen (its Touch object, which also accepts the mouse), so
no buttons are needed: on phones and tablets you touch the game, and in portrait it stays centred.
[`crates/museo`](../../crates/museo) likewise detects the touch pointer and reports it to the page.

<br clear="right">

## Controls

| Input | Action |
|---|---|
| Click or tap | Start, kick, press PLAY and OK |

The arrow keys also reached the Platform behaviour of the ball and the bombs (with the
default controls enabled), but their maximum speed and jump strength are 0: they do
nothing, and they were not ported.

## How it was made

1. **Download**: the HTML5 version that itch.io serves in an iframe (`tools/originales.py` crawls it
   file by file): `index.html`, `c2runtime.js` (Construct 2 runtime minified with
   Closure Compiler), `data.js` (the project), `images/` and `media/` (OGG and M4A). According to its
   `offline.appcache`, it was exported on **23-Jul-2015** at 14:31 UTC.
2. **REA** ([rea.tools](https://rea.tools/), `rea-agents` 6.7.0): `analyze-javascript-application`
   on the HTML5 folder (`ev_db50787b1eb906e7188fabc23c036767b52d9d8d02634d473a6564c5a1921c31`,
   summary `ev_32ce2708b515db2bf7bb58b69e5a7695ed2c51260558007c87a532409e50d36a`). It found the
   four scripts and warned about the two inline blocks in `index.html`. None of them defines the functions
   `inicio()`, `tiempo()`, `fin()`, `reiniciar()` or `compartir()` that the game calls.
3. **Events**: [`tools/c2_eventos.py`](../../tools/c2_eventos.py) reads `data.js` and dumps the event
   sheets to pseudocode (`reference/eventos.txt`) and the project to `reference/resumen.txt`.
   The minified export lost the object names (`t0`, `t1`...) and the names of the
   conditions and actions (`kc.prototype.ca.Mr`). The objects were named after their image, and the
   conditions and actions by reading their minified code (`reference/aces.txt`). All of that is in
   `reference/nombres.json`.
4. **Runtime**: the following were translated from the minified runtime:
   - the Platform behaviour;
   - Sprite's tick and animation change;
   - the collision test between polygons;
   - Text (alignment and line height);
   - Audio (volume in dB, tags);
   - the runtime tick: dt from 0 to 0.1 s, time scale, and the order of behaviours,
     animations, layout change and events.

   The snippets are in `reference/runtime.txt`. Since Retro Penal (checkpoint 2.4) these
   pieces live in [`crates/construct2`](../../crates/construct2), shared by the Construct 2
   games.
5. **Assets**: [`tools/extraer.sh`](tools/extraer.sh) copies the images, the OGG files and `data.js`.
6. **Port of the event sheets** (`src/juego.rs`):
   - The sheets are not interpreted: every event is Rust code, in the order of the sheet. The
     conditions pick instances like in Construct 2. Each one filters the list of instances it
     receives, sub-events inherit that list and actions are applied to each chosen
     instance. `random()` is evaluated once per instance.
   - Triggers run when the runtime fires them: "On any touch start" between ticks (like
     the DOM events), "On animation finished" during the sprite tick and "On start of
     layout" when changing layout. A `SetAnim` inside a trigger is applied afterwards, as in
     the runtime.
   - The tick follows `c2runtime.js`. The browser dt is clamped to 0.1 s and multiplied by
     the time scale. Then come the behaviours, the animations, the layout change and
     the events. Destroyed objects disappear when each top-level event finishes.
   - Text is drawn with Liberation Sans and the runtime's line height (`ceil(pt × 96 / 72) + 4`
     px), and it stops drawing the lines that do not fit in the box.
   - The game's Touch also receives the mouse: on PC you play with the click.
     `simulate_mouse_with_touch(false)` prevents a touch from also counting as a click.

## Correspondence with the original

| Construct 2 | Rust |
|---|---|
| `data.js` (types, animations, layouts) | `crates/construct2` (`datos.rs`) |
| Sprite, Text, Audio, Platform behaviour, collisions | `crates/construct2` |
| Runtime tick, "Start events" and "Game events" sheets | `juego.rs` |
| `index.html` (400×500 canvas, *letterbox*) | `main.rs` with `museo::Pantalla` |

## Original quirks that are kept

- **An event with no conditions** in the "Game Over" group plays the `click` sound **on every tick** of
  the run, and calls `javascript:compartir(...)`. It was probably left that way when a share button was
  deleted. The Audio plugin creates a new instance while the previous ones are still playing, so
  you hear a continuous hum.
- The JavaScript calls (`inicio()`, `tiempo()`, `fin()`, `reiniciar()`, `compartir()`) do
  nothing: the exported page does not define those functions.
- When the run starts the time scale is 0. The game stays frozen until the first touch,
  and goes back to 0 when you lose.
- `ContadorBomba` is a global variable that is not reset: in the next run the bombs
  arrive at whatever pace the previous one ended with.
- The foot areas (the red one, which makes you lose, and the green one, where the kick reaches the ball)
  are behind the background and are not visible.
- Kicking depends on the "patear" animation, which lasts a fifth of a second (1 frame at speed
  5). The ball has to be in the green area during that time.
- The kick uses `random(250, 400)` from second 20 and `random(350, 450)` from second 30, when
  the gravity of the balls already on screen also goes up to 450 (the ones appearing later still use 400).
- When a Sprite changes frame it takes the proportion of the new one. The bomb explosion uses
  frames of 110 px instead of 50, and at the feet it is also scaled ×2.
- The bomb that explodes at the feet is destroyed in a trigger. It still exists for the second
  trigger, "on animation finished", which is the one that ends the run.
- After losing, touches still put the player in the kick pose.
- The bomb in the layout starts off screen (x = −93) and falls forever.

## Differences from the original

- **Fonts**: the original drew the texts with the system fonts, "Arial" and "Arial
  Black". The port uses Liberation Sans 2.1.5 (regular and bold, SIL OFL 1.1, with Arial's
  metrics), in [`crates/construct2/fuentes`](../../crates/construct2/fuentes). Arial Black has no
  free equivalent and bold is used.
- **Sound**: the OGG files are used. The browser chose between OGG and M4A; both are the same sound.
- **Randomness**: `random()` uses macroquad's generator instead of `Math.random()`.
- **Window**: the 400×500 canvas is scaled with black letterboxing, like the original's *letterbox*
  mode. Fullscreen and the web manifest orientation were not ported.

## Regenerating the assets

```sh
tools/extraer.sh <html5 folder>        # for example ../../originales/que-no-caiga/html5
```

## Tests

```sh
node ../../tools/humo/nativo.mjs que-no-caiga   # after cargo build --release
node ../../tools/humo/web.mjs que-no-caiga      # after tools/build_web.sh
```

They follow the steps in `humo.json`: the menu, PLAY, the first kick and the ending. These were checked
separately with clicks at the pace of the ball:
- several kicks in a row;
- with a temporary change without going up (the time starting at 47), the bomb from second 50, its
  explosion at the feet and the ending.
