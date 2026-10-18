// Test de humo de la versión nativa de un juego del museo: ejecuta target/release/<juego> en un
// servidor X virtual (Xvfb), sigue los pasos de games/<juego>/humo.json con xdotool y guarda
// capturas de la ventana con ImageMagick. Es el mismo humo.json que usa web.mjs (ver allí el
// formato); aquí cuentan "esperar", "tecla", "zona" (clic), "despues" y "captura".
//
// Uso:  node tools/humo/nativo.mjs <juego> [carpeta de capturas]
//       (antes: cargo build --release -p <juego>; hacen falta Xvfb, xdotool e ImageMagick)
//
// Comprueba que la ventana aparezca, que el juego siga vivo al terminar los pasos y que las
// capturas no queden en negro. El título de la ventana se lee de [package.metadata.museo] titulo.

import { execFileSync, spawn } from "node:child_process";
import { existsSync, mkdirSync, openSync, readFileSync } from "node:fs";
import { join, resolve } from "node:path";

const raiz = resolve(import.meta.dirname, "..", "..");
const juego = process.argv[2];
if (!juego) {
    console.error("uso: node tools/humo/nativo.mjs <juego> [carpeta de capturas]");
    process.exit(2);
}
const salida = resolve(process.argv[3] ?? join("/tmp", `humo-${juego}`, "nativo"));
const binario = join(raiz, "target", "release", juego);
if (!existsSync(binario)) {
    console.error(`falta ${binario}: ejecuta antes cargo build --release -p ${juego}`);
    process.exit(2);
}
const rutaConfig = join(raiz, "games", juego, "humo.json");
const config = existsSync(rutaConfig)
    ? JSON.parse(readFileSync(rutaConfig, "utf8"))
    : { pasos: [{ esperar: 3000 }, { captura: "inicio" }] };
const titulo = /^titulo\s*=\s*"(.*)"$/m.exec(readFileSync(join(raiz, "games", juego, "Cargo.toml"), "utf8"))?.[1];
if (!titulo) {
    console.error(`falta [package.metadata.museo] titulo en games/${juego}/Cargo.toml`);
    process.exit(2);
}
mkdirSync(salida, { recursive: true });

const dormir = (ms) => new Promise((ok) => setTimeout(ok, ms));
const lista = (v) => (v === undefined ? [] : Array.isArray(v) ? v : [v]);
const display = `:${90 + Math.floor(Math.random() * 400)}`;
const entorno = { ...process.env, DISPLAY: display };
const x = (...args) => execFileSync(args[0], args.slice(1), { env: entorno, encoding: "utf8" }).trim();

// Nombres de tecla de Playwright (los de humo.json) a keysyms de X11.
const TECLAS = {
    ArrowLeft: "Left", ArrowRight: "Right", ArrowUp: "Up", ArrowDown: "Down", Enter: "Return",
    Escape: "Escape", " ": "space", Space: "space", Shift: "Shift_L", Control: "Control_L",
    Alt: "Alt_L", Tab: "Tab", Backspace: "BackSpace",
};
const keysym = (t) => TECLAS[t] ?? (t.length === 1 ? t.toLowerCase() : t);

const xvfb = spawn("Xvfb", [display, "-screen", "0", "1280x960x24", "-nolisten", "tcp"], { stdio: "ignore" });
await dormir(1000);
const log = join(salida, "juego.log");
const proceso = spawn(binario, [], { env: entorno, stdio: ["ignore", openSync(log, "w"), openSync(log, "a")] });
let terminado = null;
proceso.on("exit", (codigo, senal) => (terminado = senal ?? codigo));

const problemas = [];
const capturas = [];
let ventana = null;
try {
    const patron = "^" + titulo.replace(/[.*+?^${}()|[\]\\]/g, "\\$&") + "$";
    for (let i = 0; i < 100 && !ventana && terminado === null; i++) {
        try {
            ventana = x("xdotool", "search", "--name", patron).split("\n")[0];
        } catch {
            await dormir(200);
        }
    }
    if (!ventana) throw new Error(`no aparece la ventana «${titulo}» (salida: ${terminado})`);
    const geometria = x("xdotool", "getwindowgeometry", ventana);
    const [w, h] = /Geometry: (\d+)x(\d+)/.exec(geometria).slice(1).map(Number);
    try {
        x("xdotool", "windowfocus", ventana);
    } catch {
        // Sin gestor de ventanas el foco sigue al puntero: basta con ponerlo encima.
    }
    x("xdotool", "mousemove", "--window", ventana, String(Math.floor(w / 2)), String(Math.floor(h / 2)));

    for (const paso of config.pasos) {
        if (paso.esperar) await dormir(paso.esperar);
        const ms = paso.ms ?? 150;
        const teclas = lista(paso.tecla).map(keysym);
        if (teclas.length) x("xdotool", "keydown", ...teclas);
        if (paso.zona) {
            // Misma colocación que Pantalla::destino (centrada, sin deformar).
            const [ancho, alto] = config.pantalla ?? [w, h];
            const escala = Math.min(w / ancho, h / alto);
            const px = (w - ancho * escala) / 2 + paso.zona[0] * escala;
            const py = (h - alto * escala) / 2 + paso.zona[1] * escala;
            x("xdotool", "mousemove", "--window", ventana, String(Math.round(px)), String(Math.round(py)));
            x("xdotool", "mousedown", "1");
            await dormir(ms);
            x("xdotool", "mouseup", "1");
        } else if (teclas.length) {
            await dormir(ms);
        }
        if (teclas.length) x("xdotool", "keyup", ...teclas.reverse());
        if (paso.despues) await dormir(paso.despues);
        if (paso.captura) {
            const ruta = join(salida, `${paso.captura}.png`);
            x("import", "-window", ventana, ruta);
            const media = Number(x("convert", ruta, "-format", "%[fx:mean]", "info:"));
            if (media < 0.02) problemas.push(`captura ${paso.captura} casi negra (media ${media.toFixed(3)})`);
            capturas.push(ruta);
        }
        if (terminado !== null) throw new Error(`el juego terminó durante los pasos (salida: ${terminado})`);
    }
} catch (e) {
    problemas.push(e.message);
} finally {
    if (terminado === null) proceso.kill();
    await dormir(300);
    xvfb.kill();
}
console.log(`${problemas.length ? "FALLA" : "ok   "} nativo  ventana=${ventana ?? "-"} capturas=${capturas.length}`);
for (const p of problemas) console.log(`      ${p}`);
console.log(`capturas y juego.log en ${salida}`);
process.exit(problemas.length ? 1 : 0);
