# Da Car

**Da Car** (Elvis Brevi, [itch.io](https://elvisbrevi.itch.io/da-car)) is a Dakar-style motorbike
racing prototype: the project is called "Dakar 2015" and its itch.io tags are *dakar* and
*moto*. The bike moves over terrain with hills that appear at random, and you have to reach the
finish line without the rider hitting his head. If he hits it, you start over, but the clock
keeps running. The bike, the wheels and the rider are Box2D bodies joined by joints.

It was made with Construct 2 for the browser (exported on 23-Nov-2015) and is now ported to
Rust + macroquad: it plays on PC and in the browser, also on mobile, by tilting the device.
The logic was translated from the project's event sheet (`data.js`) and from the minified
Construct 2 runtime (`c2runtime.js`), and Box2D from the Box2D 2.3.1 C++ that the one in the
runtime came from. The images, sounds and project (types, collision polygons, behaviours and
layouts) are the originals: the port reads `data.js` at startup. The runtime pieces are in
[`crates/construct2`](../../crates/construct2) and Box2D in [`crates/box2d`](../../crates/box2d).

![start](docs/inicio.png) ![race](docs/carrera.png)

![jump](docs/salto.png) ![finish](docs/meta.png)

## Running

```sh
cargo run --release -p da-car
```

On Linux you need the ALSA and OpenGL/X11 headers (`sudo apt install libasound2-dev`).
The assets are embedded in the binary.

### Web version

```sh
rustup target add wasm32-unknown-unknown
../../tools/build_web.sh da-car   # generates dist/da-car
```

The `index.html` comes from the common template in [`crates/museo/web`](../../crates/museo/web), which
also passes the device tilt (`deviceorientation`) to the game. On iOS permission is
requested on the first touch.

### Touch controls

<img src="docs/movil.png" align="right" width="320" alt="mobile in landscape">

As in the original, there are no buttons: on phones and tablets you accelerate by touching the screen and the bike
leans by turning the device. Every tick the game applies an angular velocity of five times
`beta`, the front-to-back tilt. It is played in landscape; in portrait it stays
centred.

<br clear="right">

## Controls

| Input | Action |
|---|---|
| ↑ or W, click or tap (hold) | Start and accelerate |
| ← → or A D | Lean the bike backwards or forwards |
| Tilt the device | Lean the bike (mobile only) |
| Click "retry" or tap "share" | Start over |

## How it was made

1. **Download**: the HTML5 version that itch.io serves in an iframe (32 files, which
   `tools/originales.py` crawls one by one). According to its `offline.appcache`, it was exported on
   **23-Nov-2015** at 12:34 UTC.
2. **Triage**:
   - REA ([rea.tools](https://rea.tools/), `rea-agents` 6.7.0), `analyze-javascript-application`:
     on the whole HTML5 folder it went past the 5-minute limit, because `c2runtime.js` brings Box2D
     compiled to asm.js (the partial evidence was left as
     `ev_f8a800c21eda57ade14b763eb01e205e0e138e1f7f099f8827c93554a915dca6`). It was run again on
     `index.html`, `data.js`, `sw.js` and `appmanifest.json`
     (`ev_64c5d91c4dbe33b092df1a5901d3d1a2a6f860445410aa2db0abf6ce0475e50e`, summary
     `ev_97924980dd5d2edf8462771b8b27a08509732d8797d5c572c723630e687e43c3`), and the runtime was
     read by hand.
   - Plugins: Sprite, Text, Particles, Audio, Touch (with the mouse), Mouse and Keyboard.
     Behaviours: Physics, Wrap and ScrollTo.
3. **Events**: [`tools/c2_eventos.py`](../../tools/c2_eventos.py) dumped the "InGame" sheet to
   pseudocode (`reference/eventos.txt`) and the project to `reference/resumen.txt`. The names of
   the objects and of the minified conditions and actions, identified by reading their code
   (`reference/aces.txt`), are in `reference/nombres.json`. In `src/juego.rs` each group of
   events is a function, in the order of the sheet.
4. **Box2D** ([`crates/box2d`](../../crates/box2d)): the runtime brings kripken's `box2d.js`
   (Box2D 2.3.1 compiled with Emscripten to asm.js). What Construct 2 read was identified by the
   field offsets in the asm.js memory: a polygon's normal is at +148
   bytes and its vertex count at +276, so `b2_maxPolygonVertices` is 16 and not 8. It was translated
   from the C++ of that same commit (`1e6e9da`), in `f32` and with the same order of operations:
   - the dynamic AABB tree with its free list, and the broad phase with its pair buffer sorted and
     free of duplicates;
   - the circle-circle, polygon-circle and polygon-polygon collisions (SAT with clipping);
   - the contact solver with warm starting and the two-point block solver;
   - the islands with body sleeping;
   - the TOI with sub-steps;
   - the revolute joint with limits and motor.

   The bodies, fixtures and contacts live in arenas with index-linked lists that
   keep the order of the original's, because that order decides the result.
   [`comparar/comparar.sh`](../../crates/box2d/comparar/comparar.sh) compiles that Box2D with g++ and
   runs the same scenes in both (a box, a stack, a bullet and a bike with wheels and
   joints): the trajectories match bit for bit for 200 to 600 steps.
5. **Physics behaviour** (`crates/construct2/src/fisica.rs`), translated from the runtime:
   - scale of 50 px per metre;
   - the body's position is the centre of the sprite's quadrilateral, not its origin;
   - the shapes: the collision polygon, split into convex pieces with the same splitter as
     Construct 2 (each piece goes to `b2PolygonShape::Set`), a box or a circle of radius
     `0.01 · min(width, height)` m;
   - a fixed step of 1/60 s per tick (8 velocity iterations and 3 position ones), taken in the
     tick of the first object with Physics;
   - if an event or Wrap moves the sprite, the body is moved there (`SetTransform`) and receives as
     linear velocity the displacement in pixels, without converting it to metres, as the runtime does;
   - the new contacts (`BeginContact`) are registered for the "On collision" condition;
   - the joints are anchored at an image point of the sprite and are deleted when the
     other object is destroyed.
6. **The rest of the runtime**, also in `crates/construct2`:
   - ScrollTo, which moves the view to the average of its objects in `tick2`, after the events;
   - Wrap in "viewport" mode, with each layer's view according to its parallax;
   - the layers with parallax, visibility and unbounded scroll;
   - the Particles plugin, with the random numbers requested in the same order;
   - "Trigger once", "Is silent" and the Mouse click on an object.
7. **The "InGame" sheet** (`src/juego.rs`), with the same rules as in
   [Que No Caiga](../que-no-caiga) (chosen instances, triggers between ticks, tick order):
   - Each group of events is a function, in the order of the sheet.
   - The two Or blocks of collisions have one `MemoriaColision` per condition (five each).
   - "Trigger once" stores the last tick in which it was evaluated, and it is also fulfilled on the first tick
     of the layout.
   - "Restart layout" is left pending and done on the next tick, before the events. The
     global variables and the state of the conditions (Every, Trigger once) live in `Juego`,
     outside the layout, which is why they survive the restart.
   - After the events the registered collisions are forgotten and ScrollTo moves the view.
8. **Input**: `crates/museo` gained `sensores::beta()`, which on the web reads the latest
   `deviceorientation` (natively it is 0).
9. **Assets**: [`tools/extraer.sh`](tools/extraer.sh) copies the images, the OGG files and `data.js`.

## Correspondence with the original

| Construct 2 | Rust |
|---|---|
| `data.js` (types, animations, behaviours, layouts) | `crates/construct2` (`datos.rs`) |
| Box2D 2.3.1 in asm.js | `crates/box2d` |
| Physics, Wrap, ScrollTo, Particles, Sprite, Text, Audio, On collision | `crates/construct2` |
| Runtime tick and "InGame" sheet | `src/juego.rs` |
| `index.html` (800×450 canvas) and Touch.Beta | `src/main.rs` with `museo::Pantalla` and `museo::sensores` |

## Original quirks that are kept

- **The clock is not reset on crashing.** The global variables survive "Restart layout": the
  clock keeps adding up between attempts and only goes back to 0 when you reach the finish line.
- The clock reads 1 as soon as the race starts: "Every 1 second" is not evaluated while you are not
  racing, and when it is evaluated again more than a second has already passed since the last time.
- **The buttons at the end work during the race.** The "GUI Final" layer is only hidden, and
  Construct 2's conditions do not check whether an object is visible. A click where "retry" is or a
  tap where "share" is (the centre right of the screen) restarts the race. Touch
  also receives the mouse, so a click on "share" restarts as well.
- "share" does not share anything: it restarts, like "retry".
- On mobile, tapping "retry" does nothing: it is a Mouse click, and Construct 2's Touch
  cancels the clicks the browser emulates after a tap. Only "share" restarts.
- A click on "retry" with the mouse is also a touch, so the race starts as soon as
  it restarts.
- **The game runs faster on screens above 60 Hz.** The physics advances 1/60 s per tick and the
  wheel speed rises 15 °/s per tick, without looking at real time. In the smoke test (Xvfb,
  no vertical sync, about 240 fps) the bike reaches the finish line in a couple of seconds.
- Before starting, the bike rocks about 30 px back and forth, with a period of about 7 s. Nothing
  stops the wheels, which spin freely around the bike's image points, and those points
  end up less than 1 px from their centres: they roll barely off-centre.
- Releasing ← or → ("Trigger once") stops both wheels for one tick, even while
  accelerating.
- The speed rises to 2805 °/s, not 2800: 15 is added while it is below 2800. On release
  it drops by 10 at a time, but the wheels keep spinning with the last value they were given: the bike keeps
  rolling until something brakes it.
- After the finish line the bike keeps going on its own: the events that move it stop running, but its wheels
  keep their spin.
- "accelerar" plays only once since the game is opened: it requires "Is silent", and the looping engine
  that starts with it never stops playing, not even on restart.
- Each of the four hills rolls its own `round(random(0, 4))` and comes out if the number is
  its own. The first one needs a 0, which is half as likely as 1, 2 or 3, and 4 is not
  any of them. Sometimes two or three hills come out on top of each other. If none comes out, the spawn stays behind and
  is rolled again on the next tick.
- The texts are cut off: the box of "Presiona UP para comenzar" only has room for one line and
  it reads "Presiona UP para". At the end you see "Tu tiempo fue: N", without "segundos.".
- The ground is nine segments that Wrap moves from one side of the view to the other. Each one reappears 1 px
  from the edge without looking at the others, so sometimes there are slits left.
- The player layer has a vertical parallax of 0.1. If the bike goes up a lot (top y ≈ 175),
  Wrap sees the ground below the view and moves it to the top.
- On mobile, the tilt is applied every tick and overrides the arrow keys. If the browser does not give the
  orientation, the bike does not turn.

## Differences from the original

- **Fonts**: the original drew the texts with the system font "Arial". The port uses
  Liberation Sans 2.1.5 (SIL OFL 1.1), with Arial's metrics, in
  [`crates/construct2/fuentes`](../../crates/construct2/fuentes).
- **Sound**: the OGG files are used; the browser chose between OGG and M4A, which are the same sound. The
  1 dB volume of the two sounds ends up at macroquad's maximum.
- **Box2D**: the original's is asm.js, which does the intermediate arithmetic in `f64` and rounds to
  `f32` when storing to memory. The port computes in `f32`, like the C++ it matches bit for bit,
  so over time its trajectories drift away from the browser's.
- **Chance**: the order of the random numbers is the original's, but the generator is
  macroquad's, not `Math.random`.
- **Is on mobile**: the original looked at the browser; the port looks for a touch screen.
- **Window**: the 800×450 canvas is scaled to the window with black letterboxing, like the rest of the
  museum.

## Regenerating the assets

```sh
tools/extraer.sh <html5 folder>        # for example ../../originales/da-car/html5
```

## Tests

```sh
node ../../tools/humo/nativo.mjs da-car   # after cargo build --release
node ../../tools/humo/web.mjs da-car      # after tools/build_web.sh
```

They follow the steps in `humo.json`: the start, the race accelerating and the hills. These were checked separately:
- thirty seconds accelerating non-stop: head crashes, restarts with the accumulated time,
  jumps over the hills, the dust and the ground with Wrap;
- with a temporary change that is not committed (the finish line at x = 1500), taking the `meta.png` capture: the
  final screen, the click on "retry", the restart with the layer hidden and the race that
  starts with that same click;
- the tilt: in Chromium, with the orientation emulated by CDP (`beta` of 0, 30 and −30 reaches
  the page), and natively with the game forced to mobile and a fixed `beta`: with −40 and accelerating, the
  bike lifts like with ←. It was not tested on a real phone.
