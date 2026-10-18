// Test de humo de la versión web de un juego del museo: la sirve desde dist/<juego>, la abre en
// Chromium (Playwright) como escritorio, móvil horizontal y móvil vertical, ejecuta los pasos de
// games/<juego>/humo.json y guarda capturas. El mismo humo.json lo usa nativo.mjs.
//
// Uso:  node tools/humo/web.mjs <juego> [carpeta de capturas]
//       (antes: tools/build_web.sh <juego>; una vez: npm install --prefix tools/humo)
//
// Comprueba en cada escenario:
//   - que no haya errores de JavaScript ni de carga del wasm;
//   - que la página detecte bien el puntero táctil y que los controles táctiles se vean solo en
//     móvil (lo avisa crates/museo a través de window.museo.tactilVisible);
//   - que el canvas no quede en negro.
//
// humo.json:
//   {
//     "pantalla": [640, 480],              // tamaño de la pantalla virtual del juego
//     "tactil": { "botones": 2 },          // cuántos botones tiene su ConfigTactil (sin "tactil":
//                                          // el juego no tiene controles y queda centrado)
//     "pasos": [
//       { "esperar": 3000 },
//       { "captura": "menu" },
//       { "tecla": "z", "control": "boton0", "ms": 200 },   // tecla en escritorio, toque en móvil
//       { "tecla": ["ArrowRight", "x"], "control": ["derecha", "boton1"], "ms": 800 },
//       { "zona": [320, 240] }              // clic (escritorio) o toque (móvil) en la pantalla virtual
//     ]
//   }
// Controles: izquierda, derecha, arriba, abajo, boton0..boton3, start.

import { chromium } from "playwright-core";
import { createServer } from "node:http";
import { existsSync, mkdirSync, readFileSync, readdirSync, statSync } from "node:fs";
import { homedir } from "node:os";
import { extname, join, normalize, resolve } from "node:path";

const raiz = resolve(import.meta.dirname, "..", "..");
const juego = process.argv[2];
if (!juego) {
    console.error("uso: node tools/humo/web.mjs <juego> [carpeta de capturas]");
    process.exit(2);
}
const salida = resolve(process.argv[3] ?? join("/tmp", `humo-${juego}`, "web"));
const dist = join(raiz, "dist", juego);
if (!existsSync(join(dist, "index.html"))) {
    console.error(`falta ${dist}/index.html: ejecuta antes tools/build_web.sh ${juego}`);
    process.exit(2);
}
const rutaConfig = join(raiz, "games", juego, "humo.json");
const config = existsSync(rutaConfig)
    ? JSON.parse(readFileSync(rutaConfig, "utf8"))
    : { pasos: [{ esperar: 3000 }, { captura: "inicio" }] };
mkdirSync(salida, { recursive: true });

const escenarios = [
    { nombre: "escritorio", movil: false, viewport: { width: 1280, height: 720 } },
    { nombre: "movil-horizontal", movil: true, viewport: { width: 915, height: 412 } },
    { nombre: "movil-vertical", movil: true, viewport: { width: 412, height: 915 } },
];

// Chromium: $CHROMIUM, el de /opt/pw-browsers o el de la caché de Playwright.
function buscarChromium() {
    if (process.env.CHROMIUM) return process.env.CHROMIUM;
    const candidatos = ["/opt/pw-browsers/chromium"];
    for (const base of ["/opt/pw-browsers", join(homedir(), ".cache", "ms-playwright")]) {
        if (!existsSync(base)) continue;
        for (const d of readdirSync(base).filter((d) => d.startsWith("chromium-")).sort().reverse()) {
            candidatos.push(join(base, d, "chrome-linux64", "chrome"), join(base, d, "chrome-linux", "chrome"));
        }
    }
    const encontrado = candidatos.find((c) => existsSync(c) && statSync(c).isFile());
    if (!encontrado) throw new Error("no encuentro Chromium: define CHROMIUM=/ruta/a/chrome");
    return encontrado;
}

const tipos = { ".html": "text/html", ".js": "text/javascript", ".wasm": "application/wasm", ".png": "image/png" };
const servidor = createServer((req, res) => {
    const ruta = normalize(decodeURIComponent(new URL(req.url, "http://x").pathname)).replace(/^\/+/, "");
    const archivo = join(dist, ruta || "index.html");
    if (!archivo.startsWith(dist) || !existsSync(archivo) || !statSync(archivo).isFile()) {
        res.writeHead(404).end();
        return;
    }
    res.writeHead(200, { "Content-Type": tipos[extname(archivo)] ?? "application/octet-stream" });
    res.end(readFileSync(archivo));
});
await new Promise((ok) => servidor.listen(0, "127.0.0.1", ok));
const url = `http://127.0.0.1:${servidor.address().port}/`;

// Misma disposición que crates/museo/src/tactil.rs (en píxeles CSS del viewport).
function posicionControl(nombre, w, h) {
    const u = Math.min(w, h), m = 0.05 * u, rc = 0.2 * u, rb = 0.095 * u;
    const base = h - m - rb;
    const cruceta = [m + rc, h - m - rc];
    const n = config.tactil?.botones ?? 0;
    const primero = [w - m - rb, n > 1 ? base - 1.5 * rb : base];
    const botones = [[0, 0], [-1.8, 1.5], [-3.6, 0], [-1.8, -1.5]].map(([x, y]) => [primero[0] + x * rb, primero[1] + y * rb]);
    const masAlto = Math.min(base, ...botones.slice(0, n).map((b) => b[1]));
    const direcciones = { izquierda: [-1, 0], derecha: [1, 0], arriba: [0, -1], abajo: [0, 1] };
    if (nombre in direcciones) {
        const [dx, dy] = direcciones[nombre];
        return [cruceta[0] + dx * rc * 0.55, cruceta[1] + dy * rc * 0.55];
    }
    if (nombre === "start") return [w - m - 0.11 * u, masAlto - rb - m - 0.0425 * u];
    const boton = /^boton([0-3])$/.exec(nombre);
    if (boton) return botones[Number(boton[1])];
    throw new Error(`control desconocido: ${nombre}`);
}

// Misma colocación que Pantalla::destino: escalada, centrada o arriba (vertical con táctil).
function posicionZona([x, y], w, h, arriba) {
    const [ancho, alto] = config.pantalla ?? [w, h];
    const escala = Math.min(w / ancho, h / alto);
    const ox = (w - ancho * escala) / 2;
    const oy = arriba ? 0 : (h - alto * escala) / 2;
    return [ox + x * escala, oy + y * escala];
}

const lista = (v) => (v === undefined ? [] : Array.isArray(v) ? v : [v]);
const dormir = (ms) => new Promise((ok) => setTimeout(ok, ms));

// Fracción de píxeles no negros de una captura, medida en la propia página.
async function fraccionNoNegra(page, png) {
    return page.evaluate(async (b64) => {
        const img = new Image();
        img.src = `data:image/png;base64,${b64}`;
        await img.decode();
        const c = document.createElement("canvas");
        c.width = img.width;
        c.height = img.height;
        const ctx = c.getContext("2d");
        ctx.drawImage(img, 0, 0);
        const d = ctx.getImageData(0, 0, c.width, c.height).data;
        let n = 0;
        for (let i = 0; i < d.length; i += 4) if (d[i] + d[i + 1] + d[i + 2] > 24) n++;
        return n / (d.length / 4);
    }, png.toString("base64"));
}

const navegador = await chromium.launch({
    executablePath: buscarChromium(),
    args: ["--use-angle=swiftshader", "--enable-unsafe-swiftshader", "--ignore-gpu-blocklist", "--autoplay-policy=no-user-gesture-required"],
});
let fallos = 0;
for (const e of escenarios) {
    const contexto = await navegador.newContext({
        viewport: e.viewport,
        deviceScaleFactor: e.movil ? 2 : 1,
        isMobile: e.movil,
        hasTouch: e.movil,
    });
    const page = await contexto.newPage();
    const errores = [];
    page.on("pageerror", (err) => errores.push(`pageerror: ${err.message}`));
    // Los 404 se informan con su URL (abajo); el mensaje de consola equivalente no la incluye.
    page.on("console", (m) => m.type() === "error" && !m.text().startsWith("Failed to load resource") && errores.push(`console: ${m.text()}`));
    page.on("requestfailed", (r) => errores.push(`requestfailed: ${r.url()}`));
    page.on("response", (r) => r.status() >= 400 && errores.push(`HTTP ${r.status()}: ${r.url()}`));
    await page.goto(url);
    await page.waitForFunction(() => window.museo?.tactilVisible !== null && window.museo?.tactilVisible !== undefined, null, { timeout: 30000 }).catch(() => {});
    const cdp = e.movil ? await contexto.newCDPSession(page) : null;
    const { width: w, height: h } = e.viewport;
    // Como Tactil::juego_arriba: en vertical, arriba solo si hay controles en pantalla.
    const arriba = e.movil && h > w && Boolean(config.tactil);
    const capturas = [];
    let negro = 1;

    for (const paso of config.pasos) {
        if (paso.esperar) await dormir(paso.esperar);
        const ms = paso.ms ?? 150;
        let puntos = [];
        if (paso.zona) puntos.push(posicionZona(paso.zona, w, h, arriba));
        if (e.movil) puntos.push(...lista(paso.control).map((c) => posicionControl(c, w, h)));
        if (e.movil && puntos.length) {
            const touchPoints = puntos.map(([x, y], id) => ({ x, y, id }));
            await cdp.send("Input.dispatchTouchEvent", { type: "touchStart", touchPoints });
            await dormir(ms);
            await cdp.send("Input.dispatchTouchEvent", { type: "touchEnd", touchPoints: [] });
        } else if (!e.movil) {
            const teclas = lista(paso.tecla);
            for (const t of teclas) await page.keyboard.down(t);
            if (paso.zona) await page.mouse.click(...puntos[0], { delay: ms });
            else if (teclas.length) await dormir(ms);
            for (const t of teclas.reverse()) await page.keyboard.up(t);
        }
        if (paso.despues) await dormir(paso.despues);
        if (paso.captura) {
            const ruta = join(salida, `${e.nombre}-${paso.captura}.png`);
            const png = await page.screenshot({ path: ruta });
            negro = Math.min(negro, await fraccionNoNegra(page, png));
            capturas.push(ruta);
        }
    }

    const estado = await page.evaluate(() => ({
        coarse: matchMedia("(pointer: coarse)").matches,
        tactilVisible: window.museo?.tactilVisible ?? null,
    }));
    const problemas = [...errores];
    if (estado.coarse !== e.movil) problemas.push(`(pointer: coarse) = ${estado.coarse}`);
    if (estado.tactilVisible !== e.movil) problemas.push(`controles táctiles visibles = ${estado.tactilVisible}`);
    if (capturas.length && negro < 0.05) problemas.push(`canvas casi negro (${(negro * 100).toFixed(1)} % de píxeles con color)`);
    fallos += problemas.length;
    console.log(`${problemas.length ? "FALLA" : "ok   "} ${e.nombre.padEnd(17)} táctil=${estado.tactilVisible} capturas=${capturas.length}`);
    for (const p of problemas) console.log(`      ${p}`);
    await contexto.close();
}
await navegador.close();
servidor.close();
console.log(`capturas en ${salida}`);
process.exit(fallos ? 1 : 0);
