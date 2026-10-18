// Controles táctiles para móviles y tablets: cruceta, botones A (imán) y B (salto) y START.
// Se muestran si la página web informa de un puntero táctil, o en cuanto llega el primer toque.

use macroquad::prelude::*;

/// Botones virtuales pulsados en este frame.
#[derive(Clone, Copy, Default)]
pub struct Pulsados {
    pub a: bool,
    pub b: bool,
    pub start: bool,
    pub arriba: bool,
    pub abajo: bool,
    pub izquierda: bool,
    pub derecha: bool,
}

pub struct Tactil {
    pub activo: bool,
}

struct Disposicion {
    cruceta: Vec2,
    radio_cruceta: f32,
    a: Vec2,
    b: Vec2,
    radio_boton: f32,
    start: Rect,
}

#[cfg(target_arch = "wasm32")]
unsafe extern "C" {
    /// Definida en web/index.html: `matchMedia('(pointer: coarse)')`.
    fn iman_es_tactil() -> i32;
}

fn es_dispositivo_tactil() -> bool {
    #[cfg(target_arch = "wasm32")]
    return unsafe { iman_es_tactil() } != 0;
    #[cfg(not(target_arch = "wasm32"))]
    false
}

fn disposicion() -> Disposicion {
    let (ancho, alto) = (screen_width(), screen_height());
    let u = ancho.min(alto);
    let margen = 0.05 * u;
    let radio_cruceta = 0.2 * u;
    let radio_boton = 0.095 * u;
    let base = alto - margen - radio_boton;
    let a = vec2(ancho - margen - radio_boton, base - 1.5 * radio_boton);
    // START va encima de A: así no tapa el juego ni en horizontal ni en vertical.
    let (ancho_start, alto_start) = (0.22 * u, 0.085 * u);
    Disposicion {
        cruceta: vec2(margen + radio_cruceta, alto - margen - radio_cruceta),
        radio_cruceta,
        a,
        b: vec2(ancho - margen - 2.8 * radio_boton, base),
        radio_boton,
        start: Rect::new(
            ancho - margen - ancho_start,
            a.y - radio_boton - margen - alto_start,
            ancho_start,
            alto_start,
        ),
    }
}

impl Tactil {
    pub fn nuevo() -> Self {
        Self {
            activo: es_dispositivo_tactil(),
        }
    }

    pub fn actualizar(&mut self) -> Pulsados {
        // Un toque que empieza y termina en el mismo frame llega como Ended: cuenta como pulsado.
        let toques: Vec<Vec2> = touches()
            .into_iter()
            .filter(|t| t.phase != TouchPhase::Cancelled)
            .map(|t| t.position)
            .collect();
        if !toques.is_empty() {
            self.activo = true;
        }
        let mut p = Pulsados::default();
        if !self.activo {
            return p;
        }
        let d = disposicion();
        for toque in toques {
            let desde_cruceta = toque - d.cruceta;
            if desde_cruceta.length() < d.radio_cruceta * 1.3 {
                // Ocho sectores de 45°: las diagonales solo se activan cerca de la diagonal.
                if desde_cruceta.length() < d.radio_cruceta * 0.25 {
                    continue;
                }
                let (dx, dy) = (desde_cruceta.x, desde_cruceta.y);
                let inclinacion = 22.5_f32.to_radians().tan();
                if dx.abs() > dy.abs() * inclinacion {
                    p.izquierda |= dx < 0.0;
                    p.derecha |= dx > 0.0;
                }
                if dy.abs() > dx.abs() * inclinacion {
                    p.arriba |= dy < 0.0;
                    p.abajo |= dy > 0.0;
                }
            } else if toque.distance(d.a) < d.radio_boton * 1.25 {
                p.a = true;
            } else if toque.distance(d.b) < d.radio_boton * 1.25 {
                p.b = true;
            } else if d.start.contains(toque) {
                p.start = true;
            }
        }
        p
    }

    pub fn dibujar(&self, p: &Pulsados) {
        if !self.activo {
            return;
        }
        let d = disposicion();
        let (r, c) = (d.radio_cruceta, d.cruceta);
        draw_circle(c.x, c.y, r, Color::new(1.0, 1.0, 1.0, 0.12));
        draw_circle_lines(c.x, c.y, r, 2.0, Color::new(1.0, 1.0, 1.0, 0.35));
        let (largo, grueso) = (0.42 * r, 0.36 * r);
        let flechas = [
            (p.arriba, vec2(0.0, -1.0)),
            (p.abajo, vec2(0.0, 1.0)),
            (p.izquierda, vec2(-1.0, 0.0)),
            (p.derecha, vec2(1.0, 0.0)),
        ];
        for (pulsado, dir) in flechas {
            let centro = c + dir * (r * 0.55);
            let tamano = if dir.x == 0.0 {
                vec2(grueso, largo)
            } else {
                vec2(largo, grueso)
            };
            draw_rectangle(
                centro.x - tamano.x / 2.0,
                centro.y - tamano.y / 2.0,
                tamano.x,
                tamano.y,
                relleno(pulsado),
            );
        }
        boton(d.a, d.radio_boton, "A", "IMAN", p.a);
        boton(d.b, d.radio_boton, "B", "SALTO", p.b);
        let s = d.start;
        draw_rectangle(s.x, s.y, s.w, s.h, relleno(p.start));
        draw_rectangle_lines(s.x, s.y, s.w, s.h, 2.0, Color::new(1.0, 1.0, 1.0, 0.35));
        texto_centrado("START", s.center(), s.h * 0.5);
    }
}

fn relleno(pulsado: bool) -> Color {
    Color::new(1.0, 1.0, 1.0, if pulsado { 0.5 } else { 0.22 })
}

fn boton(centro: Vec2, radio: f32, letra: &str, etiqueta: &str, pulsado: bool) {
    draw_circle(centro.x, centro.y, radio, relleno(pulsado));
    draw_circle_lines(
        centro.x,
        centro.y,
        radio,
        2.0,
        Color::new(1.0, 1.0, 1.0, 0.5),
    );
    texto_centrado(letra, centro - vec2(0.0, radio * 0.12), radio * 0.8);
    texto_centrado(etiqueta, centro + vec2(0.0, radio * 0.55), radio * 0.3);
}

fn texto_centrado(texto: &str, centro: Vec2, tamano: f32) {
    let medida = measure_text(texto, None, tamano as u16, 1.0);
    draw_text(
        texto,
        centro.x - medida.width / 2.0,
        centro.y + medida.offset_y / 2.0,
        tamano,
        Color::new(1.0, 1.0, 1.0, 0.85),
    );
}
