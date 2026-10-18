// Controles táctiles para móviles y tablets, configurables por juego: cruceta, de 1 a 4 botones,
// un botón START y zonas de toque sobre la pantalla del juego. Se muestran si la página web informa
// de un puntero táctil, o en cuanto llega el primer toque.

use macroquad::prelude::*;

use crate::Pantalla;

pub const MAX_BOTONES: usize = 4;
pub const MAX_ZONAS: usize = 4;

/// Qué direcciones ofrece la cruceta.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Cruceta {
    Ninguna,
    /// Solo izquierda y derecha.
    Horizontal,
    /// Cuatro direcciones y diagonales (ocho sectores de 45°).
    Completa,
}

/// Botón redondo: una letra grande y una etiqueta pequeña debajo (por ejemplo "B" / "SALTO").
#[derive(Clone, Copy)]
pub struct Boton {
    pub letra: &'static str,
    pub etiqueta: &'static str,
}

/// Controles de un juego. Los botones se colocan abajo a la derecha: el primero en el extremo
/// derecho, el segundo abajo a su izquierda, el tercero más a la izquierda y el cuarto encima,
/// formando un rombo. START va encima del botón más alto.
#[derive(Clone, Copy)]
pub struct ConfigTactil {
    pub cruceta: Cruceta,
    pub botones: &'static [Boton],
    /// Texto del botón START, si el juego lo usa (pausa, menú...).
    pub start: Option<&'static str>,
    /// Zonas de la pantalla virtual (en sus coordenadas) que cuentan como pulsadas al tocarlas.
    /// Un toque que cae sobre un control no cuenta para las zonas.
    pub zonas: &'static [Rect],
}

/// Controles virtuales pulsados en este frame.
#[derive(Clone, Copy, Default)]
pub struct Pulsados {
    pub arriba: bool,
    pub abajo: bool,
    pub izquierda: bool,
    pub derecha: bool,
    pub botones: [bool; MAX_BOTONES],
    pub start: bool,
    pub zonas: [bool; MAX_ZONAS],
}

pub struct Tactil {
    pub activo: bool,
    config: ConfigTactil,
    /// Último valor de `activo` comunicado a la página web.
    avisado: Option<bool>,
}

struct Disposicion {
    cruceta: Vec2,
    radio_cruceta: f32,
    botones: [Vec2; MAX_BOTONES],
    radio_boton: f32,
    start: Rect,
}

#[cfg(target_arch = "wasm32")]
unsafe extern "C" {
    /// Definida en el index.html: `matchMedia('(pointer: coarse)')`.
    fn museo_es_tactil() -> i32;
    /// Definida en el index.html: avisa a la página (y al test de humo) de si los controles se ven.
    fn museo_tactil_visible(visible: i32);
}

fn es_dispositivo_tactil() -> bool {
    #[cfg(target_arch = "wasm32")]
    return unsafe { museo_es_tactil() } != 0;
    #[cfg(not(target_arch = "wasm32"))]
    false
}

fn avisar_visible(_visible: bool) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        museo_tactil_visible(_visible as i32)
    };
}

/// Posición de cada botón respecto al primero, en radios de botón.
const POSICIONES: [Vec2; MAX_BOTONES] = [
    vec2(0.0, 0.0),
    vec2(-1.8, 1.5),
    vec2(-3.6, 0.0),
    vec2(-1.8, -1.5),
];

impl Disposicion {
    fn calcular(config: &ConfigTactil) -> Self {
        let (ancho, alto) = (screen_width(), screen_height());
        let u = ancho.min(alto);
        let margen = 0.05 * u;
        let radio_cruceta = 0.2 * u;
        let radio_boton = 0.095 * u;
        let base = alto - margen - radio_boton;
        // Con un solo botón queda abajo; con dos o más, el primero sube para dejar sitio al segundo.
        let primero = vec2(
            ancho - margen - radio_boton,
            if config.botones.len() > 1 {
                base - 1.5 * radio_boton
            } else {
                base
            },
        );
        let botones = POSICIONES.map(|p| primero + p * radio_boton);
        let mas_alto = botones[..config.botones.len()]
            .iter()
            .map(|b| b.y)
            .fold(base, f32::min);
        let (ancho_start, alto_start) = (0.22 * u, 0.085 * u);
        Self {
            cruceta: vec2(margen + radio_cruceta, alto - margen - radio_cruceta),
            radio_cruceta,
            botones,
            radio_boton,
            // START va encima de los botones: así no tapa el juego ni en horizontal ni en vertical.
            start: Rect::new(
                ancho - margen - ancho_start,
                mas_alto - radio_boton - margen - alto_start,
                ancho_start,
                alto_start,
            ),
        }
    }
}

impl Tactil {
    pub fn nuevo(config: ConfigTactil) -> Self {
        assert!(config.botones.len() <= MAX_BOTONES && config.zonas.len() <= MAX_ZONAS);
        Self {
            activo: es_dispositivo_tactil(),
            config,
            avisado: None,
        }
    }

    /// En vertical con controles táctiles el juego se coloca arriba y los controles debajo (si el
    /// juego no tiene controles en pantalla, porque se juega tocándolo, queda centrado).
    pub fn juego_arriba(&self) -> bool {
        self.activo && screen_height() > screen_width() && self.hay_controles()
    }

    fn hay_controles(&self) -> bool {
        let c = &self.config;
        c.cruceta != Cruceta::Ninguna || !c.botones.is_empty() || c.start.is_some()
    }

    /// Lee los toques del frame. Admite varios dedos a la vez (andar y saltar).
    pub fn actualizar(&mut self, pantalla: &Pantalla) -> Pulsados {
        // Un toque que empieza y termina en el mismo frame llega como Ended: cuenta como pulsado.
        let toques: Vec<Vec2> = touches()
            .into_iter()
            .filter(|t| t.phase != TouchPhase::Cancelled)
            .map(|t| t.position)
            .collect();
        if !toques.is_empty() {
            self.activo = true;
        }
        if self.avisado != Some(self.activo) {
            self.avisado = Some(self.activo);
            avisar_visible(self.activo);
        }
        let mut p = Pulsados::default();
        if !self.activo {
            return p;
        }
        let c = &self.config;
        let d = Disposicion::calcular(c);
        'toques: for toque in toques {
            let desde_cruceta = toque - d.cruceta;
            if c.cruceta != Cruceta::Ninguna && desde_cruceta.length() < d.radio_cruceta * 1.3 {
                if desde_cruceta.length() < d.radio_cruceta * 0.25 {
                    continue;
                }
                let (dx, dy) = (desde_cruceta.x, desde_cruceta.y);
                if c.cruceta == Cruceta::Horizontal {
                    p.izquierda |= dx < 0.0;
                    p.derecha |= dx > 0.0;
                    continue;
                }
                // Ocho sectores de 45°: las diagonales solo se activan cerca de la diagonal.
                let inclinacion = 22.5_f32.to_radians().tan();
                if dx.abs() > dy.abs() * inclinacion {
                    p.izquierda |= dx < 0.0;
                    p.derecha |= dx > 0.0;
                }
                if dy.abs() > dx.abs() * inclinacion {
                    p.arriba |= dy < 0.0;
                    p.abajo |= dy > 0.0;
                }
                continue;
            }
            for (i, boton) in d.botones[..c.botones.len()].iter().enumerate() {
                if toque.distance(*boton) < d.radio_boton * 1.25 {
                    p.botones[i] = true;
                    continue 'toques;
                }
            }
            if c.start.is_some() && d.start.contains(toque) {
                p.start = true;
                continue;
            }
            let en_juego = pantalla.a_virtual(toque, self.juego_arriba());
            for (i, zona) in c.zonas.iter().enumerate() {
                p.zonas[i] |= zona.contains(en_juego);
            }
        }
        p
    }

    pub fn dibujar(&self, p: &Pulsados) {
        if !self.activo {
            return;
        }
        let c = &self.config;
        let d = Disposicion::calcular(c);
        if c.cruceta != Cruceta::Ninguna {
            let (r, centro) = (d.radio_cruceta, d.cruceta);
            draw_circle(centro.x, centro.y, r, Color::new(1.0, 1.0, 1.0, 0.12));
            draw_circle_lines(centro.x, centro.y, r, 2.0, Color::new(1.0, 1.0, 1.0, 0.35));
            let (largo, grueso) = (0.42 * r, 0.36 * r);
            let flechas = [
                (p.izquierda, vec2(-1.0, 0.0)),
                (p.derecha, vec2(1.0, 0.0)),
                (p.arriba, vec2(0.0, -1.0)),
                (p.abajo, vec2(0.0, 1.0)),
            ];
            let cuantas = if c.cruceta == Cruceta::Horizontal {
                2
            } else {
                4
            };
            for (pulsado, dir) in &flechas[..cuantas] {
                let medio = centro + *dir * (r * 0.55);
                let tamano = if dir.x == 0.0 {
                    vec2(grueso, largo)
                } else {
                    vec2(largo, grueso)
                };
                draw_rectangle(
                    medio.x - tamano.x / 2.0,
                    medio.y - tamano.y / 2.0,
                    tamano.x,
                    tamano.y,
                    relleno(*pulsado),
                );
            }
        }
        for (i, b) in c.botones.iter().enumerate() {
            boton(d.botones[i], d.radio_boton, b, p.botones[i]);
        }
        if let Some(texto) = c.start {
            let s = d.start;
            draw_rectangle(s.x, s.y, s.w, s.h, relleno(p.start));
            draw_rectangle_lines(s.x, s.y, s.w, s.h, 2.0, Color::new(1.0, 1.0, 1.0, 0.35));
            texto_centrado(texto, s.center(), s.h * 0.5);
        }
    }
}

fn relleno(pulsado: bool) -> Color {
    Color::new(1.0, 1.0, 1.0, if pulsado { 0.5 } else { 0.22 })
}

fn boton(centro: Vec2, radio: f32, b: &Boton, pulsado: bool) {
    draw_circle(centro.x, centro.y, radio, relleno(pulsado));
    draw_circle_lines(
        centro.x,
        centro.y,
        radio,
        2.0,
        Color::new(1.0, 1.0, 1.0, 0.5),
    );
    texto_centrado(b.letra, centro - vec2(0.0, radio * 0.12), radio * 0.8);
    texto_centrado(b.etiqueta, centro + vec2(0.0, radio * 0.55), radio * 0.3);
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
