# Museum plan: porting the original prototypes to macroquad

Work plan to recreate in Rust + macroquad every game published on
[elvisbrevi.itch.io](https://elvisbrevi.itch.io) (except IMÁN 2026, which was only a publishing test).
It is meant to be worked on across several sessions: tick each box when you complete it and include the
plan change in the same commit.

**Next step:** the museum is complete: eleven playable ports on PC and in the browser, and
published on their itch.io pages (Fast Car Racer was skipped for lack of a binary; GitHub Pages was
dropped). What is left are the pending items of each game (above all testing sound and touch
controls on real devices) and, if a new port is made, publishing it with
`tools/publicar_itch.sh` when it is closed (4.5).

## Rules

1. **Fidelity.** The logic is translated from the original (decompiled code or engine data), not
   reinvented. The quirks of the original are kept and documented.
2. **Original assets**, extracted from the binaries. If one cannot be extracted, a provisional
   substitute is used and noted as pending.
3. **Every port has a native (PC) and a web (wasm) version**, with touch controls that appear on their
   own on phones and tablets.
4. **itch.io, only with confirmation.** The ports live in this repo and their web version is published on
   each game's itch.io page (phase 4), but every upload is confirmed first with the owner of
   the account.
5. **Obstacles do not block.** Audio that cannot be converted, proprietary formats or missing binaries
   are noted as pending (in the game's README and in the phase 1 table) and work continues
   with the rest.
6. **Evidence with REA.** Triage and reconstruction are done with [REA](https://rea.tools/); the
   Evidence IDs are noted in each game's README.

## Resuming in another session

```sh
# Rust and web
rustup target add wasm32-unknown-unknown
sudo apt install libasound2-dev unar p7zip-full      # ALSA for macroquad; RAR/7z for the originals

# REA (Claude Code: restart after setup to get its MCP tools)
npm install --global rea-agents
rea setup --client claude_code --dry-run              # review the plan and then apply it with --yes

# .NET (XNA and Unity with Mono): decompiler with resolved references
sudo apt install dotnet-sdk-8.0
dotnet tool install -g ilspycmd --version 9.1.0.7988  # the latest version requires .NET 10

# Originals (not in git): download them and check sha256 against originales/catalogo.json
python3 -I tools/originales.py descargar

# Unity (3.4 to 5.1, Web Player and WebGL): UnityPy and MonoBehaviour type trees from the assemblies
sudo apt install python3.12-venv
python3 -m venv ~/.venvs/museo && ~/.venvs/museo/bin/pip install -r tools/requirements-unity.txt

# Android: JADX 1.5.6 and apktool 3.0.3 (Java 21) in ~/.local/opt; REA looks for `apktool` in the PATH
sudo apt install openjdk-21-jre-headless
curl -LO https://github.com/skylot/jadx/releases/download/v1.5.6/jadx-1.5.6.zip   # unzip -d ~/.local/opt/jadx
curl -L -o ~/.local/opt/apktool/apktool_3.0.3.jar \
  https://github.com/iBotPeaches/Apktool/releases/download/v3.0.3/apktool_3.0.3.jar
printf '#!/bin/sh\nexec java -jar "$HOME/.local/opt/apktool/apktool_3.0.3.jar" "$@"\n' > ~/.local/bin/apktool
chmod +x ~/.local/bin/apktool

# Audio: ffmpeg to convert MP3 to Ogg Vorbis (macroquad cannot read MP3)
sudo apt install ffmpeg

# Smoke tests (checkpoint 0.9): Playwright with the Chromium from the Playwright cache or $CHROMIUM
sudo apt install xvfb xdotool imagemagick
npm install --prefix tools/humo
node tools/humo/nativo.mjs <game>   # after cargo build --release -p <game>
node tools/humo/web.mjs <game>      # after tools/build_web.sh <game>
```

Extraction tools per stack (checkpoint 0.8; outputs go to `originales/<slug>/`):

- XNA 3.1: `games/iman/tools/xnb_extract.py` (XNB v4 uncompressed, `SpriteFont`, XACT banks).
- Unity: `~/.venvs/museo/bin/python -I tools/unity_assets.py inventario|exportar|emscripten ...`.
  Reads Unity 3.4, 4.6 (`web.unity3d`) and 5.1; with `--managed <Managed folder>` it also dumps the
  fields of the MonoBehaviours (in Unity 5 builds without type trees, with the MonoScript
  resolved by hand). AssetRipper was not needed. For a port, the game's `tools/extraer.py`
  writes the `escena.json` that `crates/unity` reads: the one in Super Happy Jump is the model for a
  Unity 4 bundle, the one in One Button Prototype for a Unity 5 build with several scenes and the
  one in Go Ball for a 3D game from Unity 3.4 (meshes, materials, GUISkin and terrain).
- Unity 5.1.1 WebGL with IL2CPP (without `global-metadata.dat`): `python3 -I
  games/neuro-battle/tools/il2cpp.py <Release folder> <output>` reads the metadata compiled into
  the asm.js initial memory and dumps types, fields, methods and the annotated asm.js of each
  method; Neuro Battle's `tools/extraer.py` compiles *stub* assemblies with them to
  read the MonoBehaviours.
- Construct 2: `python3 -I tools/c2_eventos.py resumen|eventos|fuente <html5 folder>` (project,
  types, layouts, reference table of the minified runtime and event sheets in pseudocode).
- Android: `~/.local/opt/jadx/bin/jadx -d <output> <apk>` and
  `java -jar ~/.local/opt/apktool/apktool_3.0.3.jar d -o <output> <apk>`. REA uses apktool
  (`decode-android-resources`) with the launcher `~/.local/bin/apktool`; its JADX tools
  ask for a JADX MCP JAR in `REA_JADX_MCP_JAR`, which is not there. For a port, The Meaning of
  Christmas's `tools/extraer.py` reads the APK as a zip and converts the MP3 to Ogg.

## Phase 0 — Infrastructure

- [x] **0.1** Rename the repo (`iman-macroquad` → `original-prototypes-ported`) and turn it into a
  Cargo workspace with one crate per game in `games/<slug>/`.
- [x] **0.2** Inventory and download of the originals: `tools/originales.py` →
  `originales/catalogo.json` (metadata, files, sha256). The binaries stay out of git.
- [x] **0.3** Public repo on GitHub.
- [x] **0.4** Shared `crates/museo` crate extracted from IMAN: virtual screen scaled with letterboxing,
  fixed step, touch detection (`(pointer: coarse)` + first touch) and touch controls
  **configurable per game** (D-pad, 1–4 buttons, touch zones). Migrate IMAN to use it.
- [x] **0.5** Common web template: have `tools/build_web.sh` generate `index.html` (title, name of the
  wasm, touch plugin) instead of copying `games/<slug>/web/`.
- [x] **0.6** GitHub Actions: native + wasm build of the whole workspace on every push.
- [ ] **0.7** (Dropped by the owner, 21-Oct-2026) GitHub Pages with all the web versions, linked from the museum
  README.
- [x] **0.8** Extraction tools per stack: UnityPy/AssetRipper (Unity 4/5 and Unity Web
  Player), Construct 2 `data.js` reader, apktool/JADX (Android).
- [x] **0.9** Reusable web smoke test (Playwright: mobile landscape/portrait and desktop) in
  `tools/`, like the one used by hand with IMAN.

## Checkpoint template per game

Every game in phase 2 follows these steps:

1. Triage with REA: identity of the binary, stack and version, original date, controls.
2. Asset extraction with a script in `games/<slug>/tools/` (regenerable).
3. Reconstruction of the logic: decompiled C#, Construct events or pseudocode, saved in
   `games/<slug>/reference/`.
4. Native port playable from start to finish.
5. Web version + touch controls, tested on mobile (landscape and portrait) and desktop with
   `tools/humo` (game steps in `games/<slug>/humo.json`, also for the native test).
6. Game README and its entry in the museum README. They tell what the game is, with what and when it was
   made and that it is now ported, without separating "original version" and "port". Under "How it was made" the
   README documents the technical process of the port, when finished: REA triage and Evidence,
   extraction, what was translated and how each engine piece was solved (what you cannot see by reading the
   code), and how it was verified.
7. Pending items noted.
8. Publish the web version on the game's itch.io page (phase 4), with confirmation.

## Phase 1 — Triage

A first triage was done on download (2026-10-18). It remains to be confirmed with REA game by game (step 1 of
each game in phase 2).

| Game | Original downloaded | Stack detected | RE path | Known risks |
|---|---|---|---|---|
| Proyecto Nutria | `Proyecto_Nutria.rar` (1.5 MB) | XNA 3.1, XNB v4 | ✅ done | no audio in the original; see pending items in 2.1 |
| IMAN | `IMAN_(Prototipo).rar` | XNA 3.1 | ✅ done | see IMAN pending items |
| The Electric Eel Attack | `The_Electric_Eel_Attack.zip` (7 MB) | XNA 3.1 + custom `Engine.dll` | ✅ done | see pending items in 2.2 |
| Go Ball | 3 zips (Win32, Win64, macOS) | Unity 3.4.0f5, Mono, with `.mdb` | ✅ done | see pending items in 2.9 |
| The Meaning of Christmas | APK (7.8 MB) | Java + libGDX 0.9.2 | ✅ done | see pending items in 2.8 |
| Fast Car Racer | — | Android (`cl.gioco.fcr`) | skipped (see 2.11) | **no binary**: itch.io only links to Google Play |
| Super Happy Jump | `web.unity3d` (385 KB) | Unity 4.6.0f3, Web Player | ✅ done | see pending items in 2.6 |
| Que No Caiga | HTML5 (39 files) | Construct 2 | ✅ done | see pending items in 2.3 |
| Retro Penal | HTML5 + NW.js (win/osx/linux) | Construct 2 | ✅ done | see pending items in 2.4 |
| One Button Prototype | Mac, Linux, APK and WebGL | Unity 5.1.2f1, Mono | ✅ done | see pending items in 2.7 |
| Neuro Battle | `Neuro Battle.zip` (WebGL) | Unity 5.1.1f1, IL2CPP → asm.js | ✅ done | see pending items in 2.10 |
| Da Car | HTML5 (32 files) | Construct 2 + Box2D (asm.js) | ✅ done | see pending items in 2.5 |

## Phase 2 — Ports

Suggested order: first whatever reuses tools already built and then the hardest.

### 2.0 IMAN ✅
- [x] REA triage, assets, logic, native port, web + touch, README.
- [x] Migrate to `crates/museo` (checkpoint 0.4).
- Pending: DirectInput gamepad not ported; sound untested on real hardware (the server has no
  sound card); touch controls tested only in Chromium emulation.

### 2.1 Proyecto Nutria (XNA 3.1) ✅
- [x] REA triage (`inspect-managed-artifact`, `inspect-managed-members`) + build date
  (11-Nov-2009 12:52 UTC, PE header).
- [x] Assets: `tools/xnb_extract.py` (generalisation of IMAN's) with the list of content used.
  There is no audio in the original.
- [x] Logic: `ilspycmd -r <XNA 3.1 ref>` + `import-managed-reconstruction` in REA (104 methods).
  The pause effect was disassembled (ps_2_0) and rewritten in GLSL.
- [x] Native port playable.
- [x] Web + touch.
- [x] Game README + museum entry.
- [x] Pending items noted: DirectInput gamepad not ported (the chicken was only playable with a gamepad);
  touch controls tested only in Chromium emulation.

### 2.2 The Electric Eel Attack (XNA 3.1 + Engine.dll) ✅
- [x] REA triage of `The Electric Eel Attack.exe` and `Engine.dll` (9 and 8-Mar-2011, PE headers).
- [x] Assets: XNB (textures, fonts, 17 PCM `SoundEffect` → WAV) and the XML maps from the engine's
  editor. The effects were disassembled and the three that are visible were rewritten in GLSL.
- [x] Logic: decompile the two assemblies (33 + 129 methods imported in REA).
- [x] Native port playable.
- [x] Web + touch.
- [x] Game README + museum entry.
- [x] Pending items noted: the music loads when you reach its level (brief pause on web); the
  sound was not tested on real hardware (the server has no sound card); the touch
  controls were tested only in Chromium emulation. The levels were tested up to the chain
  1 → 4 → title with temporary changes, but none was played from start to finish in the port.

### 2.3 Que No Caiga (Construct 2) ✅
- [x] Triage: Construct 2 from 2015 (r20x, no number in the export), Sprite, Text,
  Touch, Audio, Browser plugins (and Button, TextBox and Mouse unused) and the Platform behaviour (REA
  `analyze-javascript-application` on `originales/que-no-caiga/html5`).
- [x] `tools/c2_eventos.py` tool (checkpoint 0.8) and `games/que-no-caiga/reference/nombres.json`
  with the names of the minified conditions and actions.
- [x] Assets: `images/*.png` and `media/*.ogg` as is, and `data.js` is read at startup.
- [x] Native port playable: the two event sheets and whatever they use from the runtime (Platform without
  solids, Sprite animations, collision polygons, Text, Audio with tags, Touch with the
  mouse).
- [x] Web + touch (played by touching the screen, no buttons).
- [x] Game README + museum entry.
- [x] Pending items noted: the fonts are Liberation Sans instead of Arial and Arial Black; the
  sound was not tested on real hardware; it was not played up to second 50 without help (the bomb was tested
  with a temporary change).

### 2.4 Retro Penal (Construct 2) ✅
- [x] Triage (compare `package.nw` with the HTML5 version): REA `analyze-javascript-application`
  on both. The NW.js packages (22-Jul-2015) are an export from the day before the HTML5 one
  (23-Jul-2015): same images and OGG, but still with the Browser plugin and the "Share" button,
  whose conditionless event is the click every tick at the end of the HTML5 one.
- [x] Events with `tools/c2_eventos.py` and `games/retro-penal/reference/nombres.json`.
- [x] Shared runtime `crates/construct2`, extracted from Que No Caiga (which switches to using it, with the
  same smoke captures): Bullet with bouncing off solids, Turret, Custom Movement, Solid and Wrap,
  collision with rotation and mirroring, On collision, Wait, Or and Else blocks, Audio with loop, Stop, volume
  and On ended, Mouse.SetCursor.
- [x] Native port playable: the three event sheets.
- [x] Web + touch (played by touching the screen, no buttons).
- [x] Game README + museum entry.
- [x] Pending items noted: the fonts are Liberation Sans instead of Arial; the sound was not
  tested on real hardware; Stop and SetVolume act on the whole file (here it changes nothing); the
  window does not scale by integer multiples like the original; from Turret, predictive aiming and the firing
  triggers were not ported (they are not used).

### 2.5 Da Car (Construct 2) ✅
- [x] Triage: exported on 23-Nov-2015; Sprite, Text, Particles, Audio, Touch, Mouse and
  Keyboard plugins, and the Physics, Wrap and ScrollTo behaviours. REA `analyze-javascript-application`
  exceeded the 5 min limit with the whole folder (Box2D in asm.js inside `c2runtime.js`): it was
  run again without the runtime and the runtime was read by hand.
- [x] Events with `tools/c2_eventos.py` and `games/da-car/reference/nombres.json`.
- [x] Box2D 2.3.1 translated to Rust in `crates/box2d` (the `box2d.js` from kripken that the
  runtime ships, with `b2_maxPolygonVertices` = 16), bit-for-bit identical to the C++ in four scenes
  (`crates/box2d/comparar/comparar.sh`).
- [x] Native port playable: the Physics behaviour (shapes, concave polygon splitter,
  joints, registered collisions), ScrollTo, Wrap by the view, layers with parallax,
  Particles and the "InGame" sheet.
- [x] Web + touch: you accelerate by touching and steer by tilting the device (`museo::sensores::beta`,
  from `deviceorientation`; on iOS permission is requested on the first touch).
- [x] Game README + museum entry.
- [x] Pending items noted: the fonts are Liberation Sans instead of Arial; the sound was not
  tested on real hardware; the tilt was not tested on a real phone (only the orientation
  emulated by CDP in Chromium, which reaches the page, and a forced fixed tilt in native); it was
  not played up to the finish line without help (the finish line was tested with a temporary change).

### 2.6 Super Happy Jump (Unity 4.6 Web Player) ✅
- [x] Open `web.unity3d` with UnityPy: `Assembly-CSharp.dll` (compiled on 9-Dec-2014, PE
  header), the scene, the prefabs, sprites, Mecanim clips, particles, the `GUISkin` and the audio, with
  `games/super-happy-jump/tools/extraer.py` → `assets/escena.json`.
- [x] REA triage of the assembly (`inspect-managed-artifact`, `inspect-managed-members`, 132 methods
  imported with `import-managed-reconstruction`; REA does not open the `UnityWeb` format) + ILSpy, with
  the `UnityEngine.dll` from Unity 5.1.2 from One Button Prototype (the 4.6 one does not come in the bundle).
- [x] Shared `crates/unity` crate: Time, GameObjects, 2D physics on top of `crates/box2d` (which gained
  sensors, per-fixture events, `QueryAABB`, `DestroyFixture`, `ApplyForceToCenter` and
  `SetMassData`, compared bit for bit with the C++ in the "corredor" scene), Animator, particles
  (Shuriken), camera and draw order, IMGUI and AudioSource. `museo::Preferencias` for the
  `PlayerPrefs` (file on PC, `localStorage` on the web).
- [x] Native port playable: the game scripts and the Unity frame in `src/juego.rs`.
- [x] Web + touch (played by touching the screen, no buttons).
- [x] Game README + museum entry.
- [x] Pending items noted: the fonts are Liberation Sans instead of Arial and Nunito instead of
  Arial Rounded MT Bold (embedded in the original, from Monotype, not redistributable); the Box2D in the
  Web Player is of unknown version (2.3.1 is used); what Unity does internally (waking the
  body moved by code, style states without background, sprites in Mecanim transitions,
  stretched particles) was deduced and could not be compared with the original running, because the
  Unity Web Player no longer runs; the sound was not tested on real hardware; the touch controls were
  tested only in Chromium emulation; the coins and the high score were tested with a temporary
  change; the iOS and Android versions are not on itch.io (only links to the stores).

### 2.7 One Button Prototype (Unity 5.1, Mono) ✅
- [x] REA triage of `Assembly-CSharp.dll` and `-firstpass` (`inspect-managed-artifact` and
  `inspect-managed-members` of the game's six types and of `FPSCounter`): compiled on
  22-Jul-2015 (PE header). The desktop and Android builds are from that day; the WebGL from
  17-Aug-2015 has the same scenes (compared with UnityPy) and the same literals in IL2CPP.
- [x] Scenes and sprites with UnityPy (no type trees: TypeTreeGeneratorAPI for the MonoBehaviours)
  in `games/one-button-prototype/tools/extraer.py`: the three scenes, tight mesh sprites
  with their `uvTransform`, trimmed textures, clips (including the constant curves of the
  `BoxCollider2D`), controllers with a blend tree, the built-in `GameSkin` and OpenSans.
- [x] `crates/unity` extended for Unity 5.1: several scenes, `PolygonCollider2D` (convex
  pieces), contacts per collider pair with `OnCollisionEnter2D`, `OverlapCircleAll`,
  `bounds`, interpolation, Animator with 1D blend trees, *AnimatePhysics*, curves on the
  collider with *Write Defaults*, `GUI.Window` with title and focus, `GUIText`, `SmoothDamp`.
  `crates/box2d` gained the `Set` with Box2D 2.3.1 welding, compared bit for bit with a C++
  compiled like Unity's in the new "rebote" scene.
- [x] Native port playable.
- [x] Web + touch (a single button: the tap).
- [x] Game README + museum entry.
- [x] Pending items noted: `import-managed-reconstruction` was not done (REA asks for the whole members
  Evidence in the call, about 250 KB; the payload is regenerated with
  `tools/rea_reconstruction.py`); the GUI fonts are Liberation Sans instead of Arial; what
  Unity does internally (interpolation, *AnimatePhysics*, window focus, `bounds`, the
  `m_Center` curve not applied, the polygon pieces) was deduced and could not be compared
  with the original running; the levels were not played from start to finish without help (the star
  was tested with a temporary change) and landing on a dragon was not tested; the touch controls were
  tested only in Chromium emulation.

### 2.8 The Meaning of Christmas (libGDX) ✅
- [x] Triage: REA `decode-android-resources` (with apktool in the PATH) for the manifest and the
  texts; JADX directly for the code (the REA tools that recover it ask for a JADX MCP
  JAR). libGDX 0.9.2 on top of its "hello world", APK built on 1-Jan-2012 and signed with the
  debug certificate.
- [x] `classes.dex` decompiled with JADX 1.5.6: the game's seven classes in `reference/java`, and
  whatever it uses from libGDX (`BitmapFont`, `Animation`, `WindowedMean`, `AndroidGraphics`) read from the
  same APK.
- [x] Assets from `assets/data` and libGDX's `arial-15` font with
  `games/the-meaning-of-christmas/tools/extraer.py`; MP3 → Ogg Vorbis with ffmpeg.
- [x] Native port playable (320×480, portrait).
- [x] Web + touch (played by touching).
- [x] Game README + museum entry.
- [x] Pending items noted: the sound was not tested on real hardware; on the web the title stays
  silent until the first touch (which already starts the game); the touch controls were tested
  only in Chromium emulation; the gift burning and the end of the run were tested with
  temporary changes (an automatic player and the money near the limit); REA's JADX tools are
  missing (`REA_JADX_MCP_JAR`).

### 2.9 Go Ball (Unity 3.4) ✅
- [x] REA triage + ILSpy of `Assembly-CSharp.dll` and `Assembly-UnityScript.dll` (with their `.mdb`),
  from the 32-bit Windows build extracted into `originales/go-ball/win32` (without the `.exe` or the
  `.pdb`):
  - Compiled on 22-Sep-2011 at 04:15 UTC (PE header), Unity 3.4.0f5. REA
    `inspect-managed-artifact` of `Assembly-CSharp.dll`
    (`ev_f3b10410c604e1409754d98b8d98c4beb49b09d7eb0ab19ef296839fedf7eb78`) and
    `inspect-managed-members` of the game's six types: 35 methods and 30 literals
    (`ev_8022ac3d81b5715672524d9edfad7041f0da03cf1ef2468f9c8b827d41080d8f`).
  - Six scenes in `BuildSettings`: `menuPrincipal` (in `mainData`, with a grassy terrain
    —its four tree prototypes have no instances— and the decorative ball) and `nivel1` to `nivel5` (`level0` to `level4`). `PlayerSettings`: "Go
    Ball", 1024×600 (640×480 on the web). No lightmaps.
  - Scripts (about 600 lines of their own): `Bola` (the ball, a `Rigidbody` with `SphereCollider`:
    horizontal `AddForce` in `Acceleration` mode with the "Horizontal" axis and jump with "Saltar"
    setting the vertical speed; it dies with the "Sierra", with `OnParticleCollision` or when leaving
    `limiteMin`/`limiteMax`; "Explosivo1" pushes it), `MoverObjeto` (objects dragged
    with the mouse horizontally or vertically, in steps of 0.05, or that go back and forth on their own; the rotation
    branch compares the y with the horizontal limits), `Objetivo` (the goal, which rotates with
    iTween `RotateBy` in *ping-pong*), `Camara` (it can follow the ball, but no scene enables it; exit, maximise,
    options, which does nothing, and restart buttons, with windows that pause with `timeScale`),
    `Transicion` (fades with a `GUITexture` and level loading), `EmpezarJuego` (the menu) and
    `TimedObjectDestructor` (UnityScript). In addition, `iTween` (4640 lines, third-party) and
    `WaterSimple` (Standard Assets) in `-firstpass`.
  - What it asks of the engine: 3D rendering with Unity 3.4's built-in shaders and its lights, a
    terrain with trees (only in the menu), the water from the Standard Assets, legacy animations
    (`Animation`, 29), legacy particles (`EllipsoidParticleEmitter`, `ParticleAnimator`,
    `ParticleRenderer` and `WorldParticleCollider`), `GUITexture`, IMGUI with `GUISkin`,
    `OnMouseDown`/`OnMouseDrag` (rays against the colliders), `AudioSource` and PhysX 2.8:
    the only dynamic thing is the ball (and some "OuterCore" particle systems with
    `Rigidbody` in `nivel2` and `nivel3`), against 148 `MeshCollider`s that the code and the
    animations move. `PhysicsManager`: bounce threshold 2, sleep velocity 0.15, maximum
    angular velocity 7 (limits how the ball rolls) and 6 iterations.
  - Proposed approach, to be decided: 3D rendering with macroquad and GLSL shaders that imitate the
    built-in ones the materials use, and for the physics, a custom solver for a sphere against
    triangles (with friction, bounce and PhysX's angular velocity limit documented)
    instead of a full 3D engine, because PhysX cannot be translated.
- [x] Scenes and assets with UnityPy in `games/go-ball/tools/extraer.py` → `assets/`:
  - UnityPy reads Unity 3.4: the engine classes come with type trees; the MonoBehaviours share
    one without the script's fields, which are generated with TypeTreeGeneratorAPI from `Managed/`. The
    `GUISkin` and the `Terrain` are MonoBehaviours from `UnityEngine.dll` with the native 3.4 format
    and are read by hand (20 fixed styles and 7 of their own; the terrain fields).
  - `escena.json`: the six scenes (8, 36, 36, 62, 64 and 66 objects) with their hierarchy, the
    3D components (MeshRenderer, MeshCollider, BoxCollider, SphereCollider, Rigidbody, Light,
    Camera, AudioSource, GUITexture, the legacy particles and the script fields), the
    `RenderSettings` of each scene, the 44 materials with their shader, the skin (the styles the scripts
    use), three bitmap fonts (ASCII and Latin-1 only), the "85mm
    Lens" flare, the menu terrain (8-bit heights, blending and the 45 grass patches; no
    trees) and the settings (time, physics, "Fantastic" quality, InputManager axes).
  - 25 meshes in `mallas/*.bin` (Unity 3.4 stores submeshes as strips: they are split into
    triangles, with the collision ones separate), 58 textures (DXT1/DXT5 decoded, Alpha8 of
    the fonts turned white with alpha) and the six faces of the diamond's two cubemaps, and the six
    clips (OGG and one WAV) with their duration. `src/recursos.rs` embeds them.
  - The 29 `Animation`s are empty (without clips: the imported models leave them that way), so there
    are no animations to port; whatever moves is moved by the scripts and iTween.
  - The captures from the itch.io page (1440×900, "Development Build") serve as visual
    reference; they do not go in the repo.
- [x] Approach decided and tested with the five levels: the one in the proposal, in `crates/unity`.
  - Custom 3D rendering with miniquad directly (`render3d.rs`): static buffers per mesh,
    textures with mipmaps (Unity stores the rows bottom-up), cubemaps, the Unity view
    (z forward, flipped like its `worldToCameraMatrix`), the 2000, 2450,
    2901 and 3000 queues, particles as billboards and the light flare on the screen.
  - Unity 3.4's shaders rewritten in GLSL from their ARB code in the build
    (`shaders3d.rs`): Diffuse and Bumped Specular (ambient + per-pixel point light with
    Unity's attenuation), Tree Soft Occlusion (per-vertex lighting and occlusion in the tangents),
    Cutout Soft Edge Unlit (two passes), the cloud (fixed function), FX/Diamond (two passes with
    cubemaps), the skybox and Particles/Alpha Blended.
  - miniquad 0.4 turns off depth test when a pipeline does not write depth: the
    transparent objects are drawn with `glDepthMask(false)`, which on the web is added by the `index.html` from
    `crates/museo` (`mq_js_bundle.js` does not bring it).
  - Custom physics instead of PhysX (`fisica3d.rs`): a sphere against triangles of a
    MeshCollider, boxes and spheres, with sequential impulses (6 iterations), friction 0.6 from the
    default material, the "skin" of 0.01, the maximum angular velocity of 7 (the ball does not roll
    faster than 1.68 m/s), the constraints and the `AddForce` forces that wait for the next
    step (from `Update`, acceleration depends on fps, as in the original); rays for
    `OnMouseDown` and for the particles.
  - Legacy particles (`particulas3d.rs`), InputManager with axis smoothing
    (`entrada.rs`), AudioSource with distance attenuation (`audio3d.rs`) and the Unity 3
    GUI (bitmap fonts, `GUI.color`, `DrawTexture`, tooltips, states without background) in
    `gui.rs`, decompiling `SendMouseEvents`, `GUI` and `GUIUtility` from `UnityEngine.dll` 3.4.
  - The game (`games/go-ball/src`): the seven scripts, iTween `RotateBy` and the Unity
    3.4 frame. The levels look like the itch.io captures and are playable (dragging boxes, jumping,
    the goal, the fire). The menu terrain is missing.
- [x] Native port playable: the menu and the five levels, from start to finish (the level 5 goal
  returns to the menu).
  - The menu terrain (`crates/unity/src/terreno.rs`): the mesh of the height map in the area
    the camera sees (rows in z, heights over 32766), the two layers blended with the control map and about 3400 grass
    tufts in four layers, as billboards that sway with the
    wind (the translated BillboardWavingDoublePass vertex program). The random distribution of
    the tufts within each sample is not Unity's.
  - The flare calibrated with the itch.io captures: the fall-off of red in the light's row gives
    a glow (size 50) 1.25 screen-heights tall, that is 2.5 % of the height per unit of
    size; with that it matches point by point in level 1. The saws cover it and turn it off, like
    Unity's occlusion ray against its MeshColliders.
  - `src/recursos.rs` with `static` (with `const` the wasm repeated the textures: 24 MB instead of
    14).
  - Tested in Xvfb: the menu, "Jugar", dragging the boxes, rolling and jumping, the exit window
    (pauses and darkens) and the return to the menu; with a temporary change (the ball placed next to the
    goal or to the danger): the goal and the move to the next level, death in the fire and in the
    saws with the restart, and level 5 returning to the menu.
- [x] Web + touch: a horizontal D-pad and the "A" button to jump (the arrows and the up
  one); outside the controls, the finger acts as a mouse (the GUI buttons and dragging the
  boxes; the first finger that starts outside the controls). `humo.json`: the menu, "Jugar", rolling
  and jumping, the exit window and the return to the menu, natively and on the web (desktop, mobile
  landscape and portrait). Dragging a box with a finger was tested separately with Playwright (CDP
  taps in mobile landscape). The wasm weighs 14 MB.
- [x] Game README + museum entry, with captures in `games/go-ball/docs`. "How it was made"
  tells the triage with its Evidence, the extraction, each engine piece and its comparison with
  the itch.io captures.
- [x] Pending items noted: `import-managed-reconstruction` was not done (as in 2.7, REA asks for the whole
  members Evidence in the call, about 250 KB for the references of the whole
  assembly); what Unity does internally (ambient without doubling, attenuation, the size of
  the flare elements, the formulas of the legacy particles, the distribution and the waves
  of the grass, the style states without background) was deduced and compared only with the five captures
  from itch.io, which do not show the menu or the windows; the physics is a custom solver, not PhysX;
  the levels were not played from start to finish without help (the goal, the fire, the saws and the
  end of level 5 were tested with the ball placed by hand); the sound was not tested on real
  hardware and on the web the menu music waits for the first click or touch (furthermore, it plays very softly,
  as in the original); the touch controls were tested only in Chromium emulation; the
  macOS build (compiled separately the same day) was not reviewed in detail; the gamepad axis for
  "Horizontal" was not ported.

### 2.10 Neuro Battle (Unity 5.1 WebGL / IL2CPP) ✅
- [x] Unpack `Neuro Battle.data` (Emscripten file package) and look for the IL2CPP metadata
  (`global-metadata.dat`):
  - `Neuro Battle.zip` (13.99 MB) is the WebGL build from **31-Aug-2015** (zip dates: the `.data` and
    the `.js` were written between 10:37 and 10:47). Unity **5.1.1f1**, product "Neuro"
    (DefaultCompany), 960×600 canvas, C# compiled with IL2CPP to a minified Emscripten asm.js:
    `Neuro Battle.js` (24.7 MB), its initial memory `Neuro Battle.html.mem` (3.2 MB)
    and the `Neuro Battle.data` package (28.9 MB). REA `inspect-artifact` of the zip
    (`ev_ea8cecdb980921ee8e4cf64d048b4dcea8aee9aae7e8ec8c0943e8595ed99946`) and
    `analyze-javascript-application` of the loader (`index.html`, `UnityConfig.js`,
    `fileloader.js` and `UnityProgress.js`, without their own network or storage)
    (`ev_6d65876e17591498545582fb9368462509ed6a7f5798fd55b1b3c3bca28647d2`).
  - The `.data` is unpacked with `tools/unity_assets.py emscripten` into
    `originales/neuro-battle/data` (the 12 files from `fileloader.js`): the four scenes in
    `BuildSettings` (`titulo`, `instrucciones`, `level` and `game_over`, in `mainData` and `level0` to
    `level2`), `sharedassets0` to `3` (the audio in `sharedassets2.resource`) and the built-in
    resources. `methods_pointedto_by_uievents.xml` is empty.
  - **There is no `global-metadata.dat`** (the WebGL 5.1.2 of One Button Prototype does have it): in
    5.1.1, IL2CPP still compiles the metadata as static C++ data, which end up in the
    asm.js initial memory (`.html.mem`, loaded at address 8, `STATIC_BASE`). There is an
    image table (Assembly-CSharp with 16 types, Assembly-UnityScript with 4, UnityEngine with
    226, mscorlib with 791, System, System.Core and Mono.Security; neither uGUI nor Networking, although
    their MonoScripts are in `mainData`); each type is a `TypeInfo` of 0xa8 bytes (name,
    namespace, method list, instance size, flags); each method, a
    `MethodInfo` with its name, the index in the function table of its signature, the type that
    declares it and the return type; and each image brings the names of its fields and their definitions
    (name, type, offset). Il2CppDumper does not read this format (it needs a
    `global-metadata.dat`). The asm.js functions and their tables are minified; the tables
    are identified by the `dynCall_<signature>` that the module exports.
  - The scripts: in C#, `BaseMovimiento` and `BaseRotacion` (the player's neuron), `Union` and
    `animacionUnion` (the pieces that join it), `disparar` and `bala`, `bordescript`,
    `muerte_enemigo`, `Aparicion`, `Camera2DFollow` (from the Standard Assets), `titulo`,
    `intrucciones` and `gameover` (with `OnGUI`), and `PlayerControl` and `matar_neurona`, which no
    scene or prefab uses; in UnityScript, `GeneradorVitaminas`, `IA_Enemy` and the enum
    `EstadoEnemigo`.
  - The scenes: `titulo`, `instrucciones` and `game_over` only have the camera with its script;
    `level` has 96 objects: the player's neuron (`neurona_base` and its `centro`), pieces with
    `Union`, the border (`Bordes`, `Lateral` and four `pata` with `bordescript`), 17
    `GeneradorVitaminas`, the `enemigo` (`IA_Enemy` and `muerte_enemigo`) and the camera
    (`Camera2DFollow` and `Aparicion`). Prefabs: `neurona_basica`, `neurona_larga`,
    `neurona_ataque` (with `disparar`), `neurona_defensa` and `bala`.
- [x] Recover types, methods and texts (Il2CppDumper or similar) and extract scenes and assets:
  - `games/neuro-battle/tools/il2cpp.py`, a reader of its own for the compiled metadata: it starts from
    the `Il2CppMetadataRegistration` (at 0x294eb0: the eight assemblies, the table of 5166
    `Il2CppType`s, the generic methods and the method references that the code resolves), reads
    each image (types, fields with their type, visibility and offset, methods with their parameters) and
    goes from the `MethodInfo` to the asm.js function through the table of its signature (`dynCall_<signature>`;
    static methods also receive a null `this` and all of them the hidden `MethodInfo*`, almost
    always null; a struct is returned through a pointer that comes first). It dumps into
    `reference/il2cpp` the game's types (`metadatos.txt`), the 85 engine methods they
    call (`motor.txt`) and the body of each method (`asmjs/<Type>.js`) with the calls, the
    type and method references resolved on first use (`DO`/`FO`), the calls inside
    a `try` (`invoke_*`) and the literals (static `String` objects in UTF-16) annotated.
  - The logic, reconstructed by hand from that asm.js in `reference/csharp` (14 scripts and
    `PlayerControl`, unused) and `reference/unityscript` (`GeneradorVitaminas` and `IA_Enemy`). What
    is not visible in the C#: IL2CPP 5.1.1 does not check for nulls (in `bala`, subtracting life from a border
    writes to low memory and the bullet is destroyed anyway), `Mathf.Round` was left *inlined* as
    `roundf` in `IA_Enemy.Seguir`, in UnityScript the calls to their own methods are
    virtual, and the `try`/`catch (UnityException)` of `Aparicion` goes through `invoke_*`.
  - `games/neuro-battle/tools/extraer.py` (with UnityPy, like the one in One Button Prototype): since there
    are no assemblies, it writes with `il2cpp.py` some C# *stubs* with the fields of each script,
    compiles them with the `csc` from the .NET SDK against the `UnityEngine.dll` 5.1.2 from One Button
    Prototype and generates the type trees with them (TypeTreeGeneratorAPI); the 72 MonoBehaviours are
    read whole. `escena.json`: the four scenes (1, 1, 96 and 1 objects), the five prefabs that
    the scripts instantiate (`neurona_basica`, `_larga`, `_ataque`, `_defensa` and `bala`), 189
    sprites in 12 trimmed textures, the three GUI screens, 9 clips and 7 controllers,
    the `GUIStyle` of the score, the InputManager axes (`Horizontal`, `Vertical`, `Rotacion` and
    `Fire2`) and the 2D physics. The two sounds (AAC in MP4, Unity 5.1's format for WebGL)
    are converted to Ogg with ffmpeg. The score font, "Light Pixel-7" from Style-7, is "freeware for
    personal use only" and is not copied.
  - For the port: the layer matrix makes the walls (`Background` layer) only collide with the
    neurons (`Neurona` layer) and the neurons not collide with each other; the generators clone the
    `enemigo` from the scene (not a prefab); the neurons' legs are *kinematic* bodies with
    triggers, and joining a neuron depends on two of them touching (in standard Box2D two
    kinematic bodies do not generate contacts: you have to see what Unity 5.1 did in the original).
- [x] Test whether the original WebGL runs in a current browser; if it runs, observe it with REA
  (`capture-browser-scenario`) to cover the logic that cannot be read:
  - **It runs** in Playwright's Chromium 1248 (headless, WebGL 1.0 with SwiftShader), served
    locally with `python3 -I -m http.server`: it takes about 15 s to load, shows "made with
    unity, Personal Edition" and then the title. The `.htaccess` is not needed (`Release/` is served
    uncompressed).
  - REA `capture-browser-scenario` (console, errors and one capture per step)
    (`ev_7e177af039fd0bb53b0ee21cea32633a6af97b34c31d1baa5250f791d35024f0`): title, "E" →
    instructions, "E" → level and, without touching anything, "game over" after 5.7 s with "TU PUNTAJE FUE DE
    5 SEGUNDOS". Right on death, 34 `NullReferenceException`s come out: the 34 copies of the enemy
    (17 generators, two batches: on entry and 5 s later) read the `target` already destroyed in the
    frame before `game_over` loads.
  - With Playwright (held keys, which REA does not allow): the neuron moves with WASD; when
    crossing a free piece, the piece joins and from then on it moves and rotates with it (arrow keys).
    So in Unity 5.1's Box2D **two triggers of kinematic bodies do generate
    contacts** (standard Box2D does not create them; Unity 5.5 added `useFullKinematicContacts`
    for solids): the port has to allow it. If the "E" of "PLAY" is still pressed when
    entering the level, it also triggers the zoom.
- [x] Native port playable: the four menu scripts and the fourteen level scripts in
  `games/neuro-battle/src/juego.rs`, with the Unity 5.1 frame (LoadLevel at the start of the next
  frame, Physics2D at a fixed step with its messages, destroyed objects erased after the physics and
  after the Update calls, Start before the first Update, the Animator in Normal mode and OnGUI).
  - `crates/box2d`: `World::sensores_cinematicos` (two kinematic bodies touch if there is a
    sensor, as observed in the original) and `b2Filter` with `SetFilterData`/`Refilter` (with 32
    bits of category and mask, for Unity's layers). The new "neuronas" scene in
    `comparar.sh` (a dynamic body teleported against a wall, kinematic legs that rotate,
    a piece that joins by passing its fixture to the player's body, a static enemy, a bullet
    that the filter lets through the wall and a layer change halfway through) is bit-for-bit identical to the C++
    compiled with the same changes; the usual six are still identical.
  - `crates/unity`: the collision matrix by layers (previously ignored; in One Button Prototype
    and Super Happy Jump all layers collide), the contacts between kinematic triggers,
    `GetButton`/`GetButtonUp` in the InputManager, the axes in the 2D project,
    `Escena::instanciar_datos` (clones an object from the scene, with "(Clone)"),
    `Audio::play_one_shot`, `azar_entero` and `FuenteGui::escala` (a larger substitute font
    so that it measures like the original).
  - Compared with the original in Chromium: the joining of pieces (they join when the tips of
    the legs touch, they are dragged and rotate with the neuron), the start of the level, the "game over" at 5 s
    with the score in the same place (Micro 5 at 2.1 times the size of "Light Pixel-7"), the
    score that accumulates between runs (5 and then 10 seconds: `contadorTiempo` is static and
    never returns to 0, verified in the original) and the E: with the original at 20 fps, a 40 ms E
    shows the instructions, 120 ms skips them and 250 ms also enters with zoom, as
    `GetKey` predicts with loading on the next frame; the port does the same. With WebAudio hooked, the
    original sets the gain to 100 in `PlayOneShot(clip, 100)` (saturated sound); the port
    does too. Tested in Xvfb with temporary changes (no enemies): joining pieces in a chain,
    shooting with the joined attack one (the border destroys the bullet) and releasing the last one with Alt.
  - `Application.Quit` (Esc on the title) does nothing, as in the WebGL, the only build of the
    original.
- [x] Web + touch: a full D-pad (WASD), four buttons (shoot, rotate each way and
  release the last neuron) and START for the zoom; on the title, the instructions and the game over,
  touching the screen is the E for one frame (so on mobile the instructions are seen). The right
  mouse button also releases the neuron ("mouse 1" of Fire2). `humo.json`: title,
  instructions, level, moving and rotating, zoom, natively and on the web (desktop, mobile landscape and
  portrait). The wasm weighs 2.5 MB.
- [x] Game README + museum entry, with captures in `games/neuro-battle/docs`. "How it was
  made" tells the triage with its Evidence, the reading of the IL2CPP metadata, the
  reconstruction, the extraction with the *stubs*, the observation of the original and the engine
  pieces.
- [x] Pending items noted: the sound was not tested on real hardware; the touch controls were
  tested only in Chromium emulation; a long run was not played from start to finish
  (shooting and releasing neurons were tested with a temporary change without enemies); what Unity
  does internally and could not be observed (the order of the `Update` calls, the exact moment of erasing
  destroyed objects, whether a kinematic trigger touches a static collider) was left deduced; the score
  font is Micro 5 instead of "Light Pixel-7" (free only for personal use); REA does not read
  the compiled IL2CPP metadata of Unity 5.1.1 (`tools/il2cpp.py` reads them) and
  `import-managed-reconstruction` does not apply (there is no managed assembly).

### 2.11 Fast Car Racer (skipped)
- Skipped by decision of the account owner (21-Oct-2026). itch.io only links to Google Play
  (`cl.gioco.fcr`) and there is no downloadable binary: without it there is no faithful port, and it is not recreated
  from the gameplay video. If the APK/AAB appears (Play Console → *App bundle explorer*) or
  the project files, it can be resumed with the usual template.

## Phase 3 — Museum

- [x] Captures of each port in `games/<slug>/docs/` and in the museum README: the eleven ports
  have their `docs/` folder (3 to 8 captures) and one capture in their entry.
- [ ] Playable web versions linked from the README (dropped with 0.7).
- [x] Check that each entry has a confirmed date, stack, description, port status and link to
  itch.io: all twelve have them. Two facts cannot be confirmed with the files and the entry says
  so: when Que No Caiga was started (exported on 23-Jul-2015 and with an itch.io page from those
  weeks, by its number, but with the 2014 World Cup theme) and the engine of Fast Car Racer
  (no binary).
- [x] READMEs of the games and museum entries with the same approach: what game it is, with what and when it was
  made and that it is now ported, without separating "original" and "port". "How it was made" expanded with
  the technical details of each port: IMAN, Proyecto Nutria, The Electric Eel Attack, Que No Caiga,
  Retro Penal, Da Car, Super Happy Jump, One Button Prototype, The Meaning of Christmas, Go Ball
  and Neuro Battle (step 6 of the template for the next ones).
- [x] Seed randomness with the time in the ports that use it without seeding it (Que No Caiga, Da Car,
  Super Happy Jump, The Meaning of Christmas and Go Ball, because of the `crates/unity` particles;
  Da Car also because of the `crates/construct2` ones): with macroquad's fixed seed, every run
  came out the same. The originals changed on every run (`Math.random` in Construct 2, `Random`
  seeded with the time in Unity and in Java). IMAN and Neuro Battle already did it, and the XNA
  ports use `xna::Random::con_reloj`. Native and web smoke tests of the five pass; two runs of Super
  Happy Jump already have the cacti at different heights. (Go Ball failed once on the desktop web:
  with another test running at the same time, the level took longer than the 5 s of the step and the
  capture came out black; repeated alone, it passes.)

## Phase 4 — Publishing on itch.io

Each itch.io page now offers the port to play in the browser. The games that already had a
web version (Que No Caiga, Retro Penal and Da Car in HTML5; One Button Prototype and Neuro Battle in
WebGL; Super Happy Jump in the Unity Web Player, which no longer runs) replace it with the port's, and
the others premiere one. Nothing is uploaded without confirmation from the account owner (rule 4).

- [x] **4.1** Prepare the publishing: butler 15.32 with the owner's session (their credentials do not
  go into the repo) and `tools/publicar_itch.sh <game>`, which builds the web version with `tools/build_web.sh` and
  uploads it with `butler push dist/<game> elvisbrevi/<project>:html5` (One Button Prototype goes to
  `prototipo-2000`), with the commit version.
- [x] **4.2** What was already on each page: the owner asked to publish without further decisions, so
  the original downloads and the old web versions stay as they are; the port is added as
  one more file (`html5` channel).
- [x] **4.3** (Done by the owner on 21-Oct-2026; the eleven pages load the published port and
  start without errors in Chromium.) Configure each page to play in the browser, from the itch.io panel (butler
  does not do it): HTML type, frame the size of the game (640×480 IMAN and The Electric Eel Attack,
  1280×720 Proyecto Nutria, 400×500 Que No Caiga and Retro Penal, 800×450 Da Car, 960×600 Super
  Happy Jump, One Button Prototype and Neuro Battle, 1024×600 Go Ball, 320×480 portrait The
  Meaning of Christmas), "Mobile friendly" with its orientation and a fullscreen button.
- [x] **4.4** Uploaded on 21-Oct-2026, with the owner's confirmation, the eleven ports (commit
  `fc81ec8`; butler processed them all). Before that, in the plan: publish the already finished ports one by one: Proyecto Nutria, IMAN, The
  Electric Eel Attack, Go Ball, The Meaning of Christmas, Que No Caiga, Retro Penal, Da Car,
  Super Happy Jump, One Button Prototype and Neuro Battle. Test each published page
  on desktop and on mobile (`tools/humo/web.mjs` adapted to a URL, or by hand).
- [ ] **4.5** From then on, every new port is published when its checkpoint closes (step 8 of the
  template), also with confirmation.
