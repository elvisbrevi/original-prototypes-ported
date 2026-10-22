# Retro Penal

**Retro Penal** (Elvis Brevi, [itch.io](https://elvisbrevi.itch.io/retro-penal)) is a penalty
shootout game. Its itch.io tags (*chile*, *seleccion-chilena*, *mundial*, *cazely*) point to the
Chilean national team. There are three attempts: an arrow moves back and forth over the penalty spot, and when you tap
the player kicks in that direction. The goalkeeper dives to one side or crouches, and the posts and the
goalkeeper deflect the ball. Every goal makes the arrow and the goalkeeper faster.

It was made in July 2015 with Construct 2, for the browser and for desktop (NW.js on Windows,
macOS and Linux), and is now ported to Rust + macroquad: it plays on PC and in the browser, also
on mobile. The logic was translated from the project's event sheets (`data.js`) and from the
minified Construct 2 runtime (`c2runtime.js`). The images, sounds and project (types,
animations, collision polygons, behaviours and layouts) are the originals: the port reads
`data.js` at startup. The runtime pieces are in
[`crates/construct2`](../../crates/construct2), which is also used by [Que No Caiga](../que-no-caiga).

![menu](docs/menu.png) ![arrow](docs/flecha.png) ![goal](docs/gol.png)

![miss](docs/fallo.png) ![ending](docs/final.png)

## Running

```sh
cargo run --release -p retro-penal
```

On Linux you need the ALSA and OpenGL/X11 headers (`sudo apt install libasound2-dev`).
The assets are embedded in the binary.

### Web version

```sh
rustup target add wasm32-unknown-unknown
../../tools/build_web.sh retro-penal   # generates dist/retro-penal
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
| Click or tap | Start, OK, kick (anywhere) and Retry |

Over the buttons the cursor becomes a hand, as in the original (Mouse.SetCursor).

## How it was made

1. **Download**: the HTML5 version that itch.io serves in an iframe (`tools/originales.py` crawls it
   file by file) and the desktop packages `win32.zip`, `osx32.zip` and `linux32.zip`.
   According to its `offline.appcache`, the HTML5 one was exported on **23-Jul-2015** at 14:40 UTC.
2. **Triage**:
   - REA ([rea.tools](https://rea.tools/), `rea-agents` 6.7.0): `analyze-javascript-application`
     on the HTML5 folder
     (`ev_d2026d791a06ca6424b5fd5eb563fdb82b69a3d9ba1d818e0c724848d4d34f4b`, summary
     `ev_cedb30a278038548b36d7e9cac9325d026c5ffe2291dd9c983fbc9d85b897e40`) and on the Windows
     `package.nw` (`ev_12cfcc4a45b1af48c0f002ecb023e0e60165b9ea85f6aa9737cd305eb4718131`, summary
     `ev_ccb1164a0f4828c2da782a24f50a4d42d15bfe8a4d2dc3ed224bb698640f0d0d`).
   - The NW.js packages (NW.js from March 2015) were opened without running them. All three bring the
     same game files, dated **22-Jul-2015** at 10:07 (zip local time); only
     `package.json` differs between Windows and the others. The images and OGG files are identical to the
     HTML5 ones, but `data.js`, `c2runtime.js` and `index.html` are not: it is an export from the previous day.
     It still has the Browser plugin, with calls to `javascript:goles()`, `intentos()`,
     `inicio()`, `reiniciar()`, `compartir()` and `fin()` (functions the page does not define), and the
     "Share" button type from the ending, with its event. The port follows the HTML5 one, the one played
     on itch.io.
3. **Events**: [`tools/c2_eventos.py`](../../tools/c2_eventos.py) reads `data.js` and dumps the event
   sheets to pseudocode (`reference/eventos.txt`) and the project to `reference/resumen.txt`. The
   names of the objects (`t0`, `t1`...) and of the minified conditions and actions
   (`Y.prototype.D.ir`) were identified by reading their code (`reference/aces.txt`) and are in
   `reference/nombres.json`.
4. **Runtime**: the following were translated from the minified runtime (in `crates/construct2`):
   - the Bullet behaviours (with bouncing off solids: the bounce angle, the push
     outwards and the fractional push), Turret, Custom Movement, Solid and Wrap, and their tick
     order;
   - collision with rotation and mirroring (boxes, quadrilaterals and polygons);
   - the "On collision with another object" condition, with its per-pair memory and the collisions
     the behaviours register;
   - System.Wait, which stores the chosen instances and continues on the tick it is fulfilled;
   - the event blocks: Or (with a trigger among its conditions), Else and the Touch and Audio
     triggers;
   - Audio with tags, loop, volume in dB, Stop, Stop all and "On ended";
   - Mouse.SetCursor and "Is over object".

   The snippets are in `reference/runtime.txt`.
5. **Assets**: [`tools/extraer.sh`](tools/extraer.sh) copies the images, the OGG files that play and
   `data.js`.
6. **Port of the sheets** (`src/juego.rs`), with the same rules as in
   [Que No Caiga](../que-no-caiga) (chosen instances, triggers between ticks, tick order):
   - Each sheet is a function (`hoja_menu`, `hoja_juego` and `hoja_game_over`) and the tick calls the
     one for the current layout.
   - System.Wait stores the rest of the actions in `construct2::Esperas` as a `Seguir` value
     (`Choque(fallo)` or `FinDePartida`), together with the uids of the chosen instances. The
     waits are fulfilled at the start of the tick, before the behaviours, and they survive the
     layout change.
   - Every "On collision" condition has its own `MemoriaColision` (four in this game). It remembers on
     which tick it saw each pair overlapping and also counts the collisions Bullet registers when it
     bounces.
   - "Every X seconds" (`Cada`) stores its last firing and resets it if it falls more than 40 ms behind,
     like the runtime.
   - Bullet's bounce off solids follows `calculateSolidBounceAngle`. It rotates in 5-degree
     steps around the previous position looking for the first free direction on each side. The
     normal ends up between the two and the new angle is the reflection of the incoming one. Then the ball
     moves in the new direction pixel by pixel until it leaves the solid and is brought close to
     it again in fractions (`pushOutSolid`). If it does not get out that way, it goes to the nearest free gap
     (`pushOutSolidNearest`).
   - Audio: `Stop` fires "On ended", like Web Audio, and the end of a sound is reported between ticks.

## Correspondence with the original

| Construct 2 | Rust |
|---|---|
| `data.js` (types, animations, behaviours, layouts) | `crates/construct2` (`datos.rs`) |
| Sprite, Text, Audio, behaviours, collisions, Wait, Every, On collision | `crates/construct2` |
| Runtime tick, "Menu Principal", "Juego" and "Game Over" sheets | `src/juego.rs` |
| `index.html` (400×500 canvas) | `src/main.rs` with `museo::Pantalla` |

## Original quirks that are kept

- **On the final screen `click` plays on every tick** as long as "Retry" is not pressed. The
  NW.js package explains why: there it was the "Share" button's event (touching it played the
  click and called `javascript:compartir(...)`). For the HTML5 version that button was deleted (and the
  Browser plugin), Construct 2 removed the conditions and actions that used it, and an event
  with no conditions was left. The text "Compartir" is still on screen, without a button. What is left of the event
  ("if 'comenzar' is not playing") is always true, and every click adds to the previous ones: you hear a
  continuous hum.
- The arrow (Turret) points at an invisible point 3 px above its base and moves back and forth
  between x = 197 and 203. That is why it sweeps from −135° to −45° (45° on each side of the vertical) and turns faster
  near the vertical. The point moves at `velocidadFlecha` px/s: 5 at the start and 0.5 more
  per goal, up to 15.
- The goalkeeper decides when the shot leaves. His invisible marker moves back and forth between x = 190 and 210 at
  `velocidadArquero` px/s (3 at the start and 5 more per goal, up to 28). If the marker is between 197 and
  203 he crouches; if not, he dives to where the marker was going. He flies 3 px per tick (it depends on
  frames per second) until x = 75 or 320, and stays down there.
- After diving to the left the goalkeeper stays mirrored: nothing returns him to normal until
  he dives to the right.
- The ball shrinks only once: the scale goes from 0.6 to 0.42 on the first tick of the shot (the
  condition is "scale > 0.5").
- The goal only counts if the ball is entirely inside an invisible box that goes from x = 51 to
  350, narrower than the goal (the posts are at 32 and 368). Shots tight against a post go
  into the net without counting, carry on and come out at the top: a miss.
- Tapping again before the shot leaves (the kick comes out on the third frame, at 0.4 s)
  restarts the animation and aims again with the arrow, which keeps turning even while hidden.
- The next shot comes when the goal sound ends (1.9 s) or 3 s after a miss.
- The waits (System.Wait) survive the layout change. The 3 s wait of the last miss
  ends already on the final screen and records the miss. If "Retry" is pressed in that first
  second, ResetGlobals arrives first and the mark stays: in the next run the arrow appears
  before OK is pressed.
- Three sounds (`chute`, `comenzar3` and `pitazo3`) are preloaded but no event plays them.
- The ball background of the menu and the ending uses Wrap. Each half reappears 1 px from the right
  edge without looking at the other, so between the two there is a gap of up to whatever they advance in one
  tick (8 px at 60 fps).
- Some objects are left off screen: the "TITULO DEL JUEGO" text from the menu and the "GAME OVER" one from the
  ending, replaced by images.
- "Is on mobile" hides the attempts bar when starting on a phone, but the bar already starts
  hidden and OK shows it anyway: it has no effect.

## Differences from the original

- **Fonts**: the original drew the texts with the system font "Arial". The port uses
  Liberation Sans 2.1.5 (regular and bold, SIL OFL 1.1), with Arial's metrics, in
  [`crates/construct2/fuentes`](../../crates/construct2/fuentes).
- **Sound**: the OGG files are used. The browser chose between OGG and M4A; both are the same sound.
  macroquad cannot tell apart different playbacks of the same file: Stop and SetVolume act on
  the whole file. Here each tag has its own file (except `click3`, which never stops), so
  nothing changes. Like Web Audio, stopping a playing sound fires its "On ended".
- **Window**: the original scaled the 400×500 canvas only by integer multiples (*letterbox
  integer scale*). The port scales it to the window with black letterboxing, like the rest of the museum.
- **Is on mobile**: the original looked at the browser; the port looks for a touch screen.

## Regenerating the assets

```sh
tools/extraer.sh <html5 folder>        # for example ../../originales/retro-penal/html5
```

## Tests

```sh
node ../../tools/humo/nativo.mjs retro-penal   # after cargo build --release
node ../../tools/humo/web.mjs retro-penal      # after tools/build_web.sh
```

They follow the steps in `humo.json`: the menu, the instructions, the arrow, the shot and its result. These were
checked separately:
- ten shots in a row: goals, misses off the post and wide, the attempt crosses and the ending;
- the bounce off the post, tick by tick;
- with a temporary change that is not committed (a single attempt): the ending, "Retry" and the wait that
  survives the layout change.
