// Las piezas del runtime de Construct 2 (c2runtime.js, minificado) que usa Que No Caiga: Sprite
// con sus animaciones y polígonos de colisión, el comportamiento Platform, Text y Audio. Se
// tradujeron del código minificado (ver reference/runtime.txt y reference/aces.txt).

use std::collections::HashMap;

use macroquad::audio::{PlaySoundParams, Sound, load_sound_from_bytes, play_sound};
use macroquad::prelude::*;

use crate::datos::{Animacion, Cuadro, Instancia, Tipo};

/// Comportamiento Platform (behaviors/platform). Sin sólidos en el proyecto nunca hay suelo, así
/// que de su posttick solo queda la caída: dy += gravedad·dt (hasta la velocidad máxima) y
/// y += dy·dt. La velocidad máxima horizontal y la fuerza de salto son 0: los controles por
/// defecto (flechas) no hacen nada.
#[derive(Clone)]
pub struct Plataforma {
    pub gravedad: f64,
    pub caida_maxima: f64,
    pub dy: f64,
    pub habilitada: bool,
}

impl Plataforma {
    /// onCreate: propiedades [velocidad máxima, aceleración, deceleración, salto, gravedad,
    /// caída máxima, doble salto, sostener salto, controles por defecto, habilitada].
    pub fn nueva(propiedades: &serde_json::Value) -> Self {
        let p = |i: usize| propiedades[i].as_f64().unwrap_or_default();
        Self {
            gravedad: p(4),
            caida_maxima: p(5),
            dy: 0.0,
            habilitada: p(9) != 0.0,
        }
    }

    pub fn posttick(&mut self, y: &mut f32, dt: f64) {
        if !self.habilitada {
            return;
        }
        self.dy += self.gravedad * dt;
        if self.dy > self.caida_maxima {
            self.dy = self.caida_maxima;
        }
        if self.dy != 0.0 {
            *y += (self.dy * dt) as f32;
        }
    }

    /// IsJumping: se mueve hacia arriba.
    pub fn subiendo(&self) -> bool {
        self.dy < 0.0
    }

    /// IsFalling.
    pub fn cayendo(&self) -> bool {
        self.dy > 0.0
    }
}

/// Instancia de Sprite.
pub struct Sprite {
    pub tipo: usize,
    pub capa: usize,
    pub x: f32,
    pub y: f32,
    pub ancho: f32,
    pub alto: f32,
    pub opacidad: f32,
    pub visible: bool,
    anim: usize,
    pub cuadro: usize,
    /// Reloj de la animación (animTimer), en tiempo de juego.
    reloj: f64,
    inicio_cuadro: f64,
    reproduciendo: bool,
    adelante: bool,
    repeticiones: i32,
    velocidad: f32,
    /// SetAnimation pendiente (changeAnimName, changeAnimFrom).
    pendiente: String,
    desde: i32,
    /// Dentro de un disparador de animación: los cambios de animación esperan al siguiente tick.
    pub en_disparo: bool,
    /// Nombre de la última animación terminada (OnAnimationFinished).
    pub terminada: String,
    /// Si el Sprite avanza su animación en cada tick (tickMe).
    avanza: bool,
    pub plataforma: Option<Plataforma>,
    pub destruido: bool,
}

/// Comparación de nombres sin distinguir mayúsculas (cr.equals_nocase).
fn igual(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

impl Sprite {
    /// Sprite.Instance.onCreate: [visibilidad inicial (0 = visible), animación inicial, cuadro
    /// inicial, colisiones].
    pub fn nuevo(instancia: &Instancia, tipo: &Tipo, capa: usize) -> Self {
        let p = &instancia.propiedades;
        let nombre = p.get(1).and_then(|v| v.as_str()).unwrap_or_default();
        let anim = tipo
            .animaciones
            .iter()
            .position(|a| igual(&a.nombre, nombre))
            .unwrap_or(0);
        let a = &tipo.animaciones[anim];
        let cuadro =
            (p.get(2).and_then(|v| v.as_u64()).unwrap_or(0) as usize).min(a.cuadros.len() - 1);
        Self {
            tipo: instancia.tipo,
            capa,
            x: instancia.x,
            y: instancia.y,
            ancho: instancia.ancho,
            alto: instancia.alto,
            opacidad: instancia.opacidad,
            visible: p.first().and_then(|v| v.as_u64()) == Some(0),
            anim,
            cuadro,
            reloj: 0.0,
            inicio_cuadro: 0.0,
            reproduciendo: true,
            adelante: a.velocidad >= 0.0,
            repeticiones: 0,
            velocidad: a.velocidad,
            pendiente: String::new(),
            desde: 0,
            en_disparo: false,
            terminada: String::new(),
            avanza: !(tipo.animaciones.len() == 1 && a.cuadros.len() == 1) && a.velocidad != 0.0,
            plataforma: instancia.comportamientos.first().map(Plataforma::nueva),
            destruido: false,
        }
    }

    pub fn cuadro_actual<'a>(&self, tipo: &'a Tipo) -> &'a Cuadro {
        &tipo.animaciones[self.anim].cuadros[self.cuadro]
    }

    /// Sprite.tick: avanza la animación. Devuelve el nombre de la animación si terminó en este
    /// tick (para el disparador OnAnimationFinished).
    pub fn tick(&mut self, tipo: &Tipo, dt: f64) -> Option<String> {
        if !self.avanza {
            return None;
        }
        self.reloj += dt;
        if !self.pendiente.is_empty() {
            self.cambiar_animacion(tipo);
        }
        let ahora = self.reloj;
        let anim = &tipo.animaciones[self.anim];
        let anterior = self.cuadro;
        let duracion = (anim.cuadros[self.cuadro].duracion / self.velocidad) as f64;
        let mut terminada = None;
        if self.reproduciendo && ahora >= self.inicio_cuadro + duracion {
            let n = anim.cuadros.len() as i64;
            let mut c = self.cuadro as i64 + if self.adelante { 1 } else { -1 };
            self.inicio_cuadro += duracion;
            if c >= n {
                if anim.ida_y_vuelta {
                    self.adelante = false;
                    c = n - 2;
                } else if anim.bucle {
                    c = anim.repetir_desde as i64;
                } else {
                    self.repeticiones += 1;
                    if self.repeticiones >= anim.repeticiones {
                        c = n - 1;
                        terminada = Some(self.terminar(anim));
                    } else {
                        c = anim.repetir_desde as i64;
                    }
                }
            }
            if c < 0 {
                if anim.ida_y_vuelta {
                    c = 1;
                    self.adelante = true;
                    if !anim.bucle {
                        self.repeticiones += 1;
                        if self.repeticiones >= anim.repeticiones {
                            c = 0;
                            terminada = Some(self.terminar(anim));
                        }
                    }
                } else if anim.bucle {
                    c = anim.repetir_desde as i64;
                } else {
                    self.repeticiones += 1;
                    if self.repeticiones >= anim.repeticiones {
                        c = 0;
                        terminada = Some(self.terminar(anim));
                    } else {
                        c = anim.repetir_desde as i64;
                    }
                }
            }
            self.cuadro = c.clamp(0, n - 1) as usize;
            if ahora
                > self.inicio_cuadro + (anim.cuadros[self.cuadro].duracion / self.velocidad) as f64
            {
                self.inicio_cuadro = ahora;
            }
            self.cambiar_cuadro(&anim.cuadros[anterior], &anim.cuadros[self.cuadro]);
        }
        terminada
    }

    /// onAnimFinish: se detiene en el último cuadro (el disparador lo lanza quien llama).
    fn terminar(&mut self, anim: &Animacion) -> String {
        self.reproduciendo = false;
        self.terminada = anim.nombre.clone();
        anim.nombre.clone()
    }

    /// Después del disparador: animRepeats = 0.
    pub fn despues_de_terminar(&mut self) {
        self.repeticiones = 0;
    }

    /// doChangeAnim: no hace nada si ya está reproduciendo esa misma animación.
    fn cambiar_animacion(&mut self, tipo: &Tipo) {
        let anterior = &tipo.animaciones[self.anim].cuadros[self.cuadro];
        let nombre = std::mem::take(&mut self.pendiente);
        let Some(nueva) = tipo
            .animaciones
            .iter()
            .position(|a| igual(&a.nombre, &nombre))
        else {
            return;
        };
        if igual(
            &tipo.animaciones[nueva].nombre,
            &tipo.animaciones[self.anim].nombre,
        ) && self.reproduciendo
        {
            return;
        }
        self.anim = nueva;
        let a = &tipo.animaciones[nueva];
        self.velocidad = a.velocidad;
        self.cuadro = self.cuadro.min(a.cuadros.len() - 1);
        if self.desde == 1 {
            self.cuadro = 0;
        }
        self.reproduciendo = true;
        self.inicio_cuadro = self.reloj;
        self.adelante = true;
        self.cambiar_cuadro(anterior, &a.cuadros[self.cuadro]);
    }

    /// OnFrameChanged interno: el tamaño sigue a la proporción del nuevo cuadro.
    fn cambiar_cuadro(&mut self, anterior: &Cuadro, nuevo: &Cuadro) {
        if anterior.recorte.w != nuevo.recorte.w {
            self.ancho *= nuevo.recorte.w / anterior.recorte.w;
        }
        if anterior.recorte.h != nuevo.recorte.h {
            self.alto *= nuevo.recorte.h / anterior.recorte.h;
        }
    }

    /// Set animation (desde: 0 el cuadro actual, 1 el principio). Dentro de un disparador de
    /// animación se aplica al empezar el siguiente tick.
    pub fn set_anim(&mut self, tipo: &Tipo, nombre: &str, desde: i32) {
        self.pendiente = nombre.to_owned();
        self.desde = desde;
        self.avanza = true;
        if !self.en_disparo {
            self.cambiar_animacion(tipo);
        }
    }

    /// Is animation playing: mira la animación pendiente si la hay.
    pub fn anim_es(&self, tipo: &Tipo, nombre: &str) -> bool {
        if !self.pendiente.is_empty() {
            igual(&self.pendiente, nombre)
        } else {
            igual(&tipo.animaciones[self.anim].nombre, nombre)
        }
    }

    /// Set scale: el tamaño del cuadro actual por la escala.
    pub fn set_escala(&mut self, tipo: &Tipo, escala: f32) {
        let c = self.cuadro_actual(tipo);
        self.ancho = c.recorte.w * escala * self.ancho.signum();
        self.alto = c.recorte.h * escala * self.alto.signum();
    }

    /// Caja (sin rotación): el origen del cuadro cae en (x, y).
    pub fn caja(&self, tipo: &Tipo) -> Rect {
        let o = self.cuadro_actual(tipo).origen;
        Rect::new(
            self.x - o.x * self.ancho,
            self.y - o.y * self.alto,
            self.ancho,
            self.alto,
        )
    }

    /// Polígono de colisión en coordenadas del mundo (None si es el rectángulo).
    fn poligono(&self, tipo: &Tipo) -> Option<Vec<Vec2>> {
        let c = self.cuadro_actual(tipo);
        (!c.poligono.is_empty()).then(|| {
            c.poligono
                .iter()
                .map(|p| vec2(self.x + p.x * self.ancho, self.y + p.y * self.alto))
                .collect()
        })
    }

    fn poligono_o_caja(&self, tipo: &Tipo) -> Vec<Vec2> {
        self.poligono(tipo).unwrap_or_else(|| {
            let r = self.caja(tipo);
            vec![
                r.point(),
                vec2(r.right(), r.top()),
                vec2(r.right(), r.bottom()),
                vec2(r.left(), r.bottom()),
            ]
        })
    }

    /// runtime.testOverlap: cajas (con bordes incluidos) y, si alguno tiene polígono propio,
    /// polígonos.
    pub fn solapa(&self, tipo: &Tipo, otro: &Sprite, tipo_otro: &Tipo) -> bool {
        if std::ptr::eq(self, otro) {
            return false;
        }
        if !cajas_se_tocan(self.caja(tipo), otro.caja(tipo_otro)) {
            return false;
        }
        if self.poligono(tipo).is_none() && otro.poligono(tipo_otro).is_none() {
            return true;
        }
        poligonos_se_cortan(
            &self.poligono_o_caja(tipo),
            &otro.poligono_o_caja(tipo_otro),
        )
    }

    /// contains_pt: el punto está en la caja y, si lo hay, en el polígono.
    pub fn contiene(&self, tipo: &Tipo, p: Vec2) -> bool {
        let r = self.caja(tipo);
        if !(p.x >= r.left() && p.x <= r.right() && p.y >= r.top() && p.y <= r.bottom()) {
            return false;
        }
        match self.poligono(tipo) {
            Some(poly) => contiene_punto(&poly, p),
            None => true,
        }
    }

    /// Sprite.draw (sin rotación): el cuadro estirado a la caja.
    pub fn dibujar(&self, tipo: &Tipo, texturas: &HashMap<String, Texture2D>) {
        if !self.visible || self.opacidad <= 0.0 {
            return;
        }
        let c = self.cuadro_actual(tipo);
        let r = self.caja(tipo);
        draw_texture_ex(
            &texturas[&c.imagen],
            r.x,
            r.y,
            Color::new(1.0, 1.0, 1.0, self.opacidad),
            DrawTextureParams {
                source: Some(c.recorte),
                dest_size: Some(r.size()),
                ..Default::default()
            },
        );
    }
}

/// cr.rect.intersects_rect: los bordes que se tocan cuentan.
fn cajas_se_tocan(a: Rect, b: Rect) -> bool {
    !(b.right() < a.left() || b.bottom() < a.top() || b.left() > a.right() || b.top() > a.bottom())
}

/// cr.segments_intersect.
fn segmentos_se_cortan(a1: Vec2, a2: Vec2, b1: Vec2, b2: Vec2) -> bool {
    let (max_ax, min_ax) = (a1.x.max(a2.x), a1.x.min(a2.x));
    let (max_ay, min_ay) = (a1.y.max(a2.y), a1.y.min(a2.y));
    let (max_bx, min_bx) = (b1.x.max(b2.x), b1.x.min(b2.x));
    let (max_by, min_by) = (b1.y.max(b2.y), b1.y.min(b2.y));
    if max_ax < min_bx || min_ax > max_bx || max_ay < min_by || min_ay > max_by {
        return false;
    }
    let dpx = b1.x - a1.x + b2.x - a2.x;
    let dpy = b1.y - a1.y + b2.y - a2.y;
    let (qax, qay) = (a2.x - a1.x, a2.y - a1.y);
    let (qbx, qby) = (b2.x - b1.x, b2.y - b1.y);
    let d = (qay * qbx - qby * qax).abs();
    let la = qbx * dpy - qby * dpx;
    if la.abs() > d {
        return false;
    }
    let lb = qax * dpy - qay * dpx;
    lb.abs() <= d
}

/// CollisionPoly_.contains_pt: dos rayos desde fuera de la caja del polígono; dentro si alguno
/// cruza un número impar de lados.
fn contiene_punto(poly: &[Vec2], p: Vec2) -> bool {
    if p == poly[0] {
        return true;
    }
    let (mut izq, mut arr, mut der, mut aba) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for q in poly {
        izq = izq.min(q.x);
        arr = arr.min(q.y);
        der = der.max(q.x);
        aba = aba.max(q.y);
    }
    let r1 = vec2(izq - 110.0, arr - 101.0);
    let r2 = vec2(der + 131.0, aba + 120.0);
    let (mut c1, mut c2) = (0, 0);
    for i in 0..poly.len() {
        let j = (i + 1) % poly.len();
        if segmentos_se_cortan(r1, p, poly[i], poly[j]) {
            c1 += 1;
        }
        if segmentos_se_cortan(r2, p, poly[i], poly[j]) {
            c2 += 1;
        }
    }
    c1 % 2 == 1 || c2 % 2 == 1
}

/// CollisionPoly_.intersects_poly.
fn poligonos_se_cortan(a: &[Vec2], b: &[Vec2]) -> bool {
    if contiene_punto(a, b[0]) || contiene_punto(b, a[0]) {
        return true;
    }
    for i in 0..a.len() {
        let i2 = (i + 1) % a.len();
        for j in 0..b.len() {
            let j2 = (j + 1) % b.len();
            if segmentos_se_cortan(a[i], a[i2], b[j], b[j2]) {
                return true;
            }
        }
    }
    false
}

/// Instancia de Text. Construct 2 la dibuja con la fuente del sistema en un canvas 2D (línea base
/// "top"); el port usa Liberation Sans, con las métricas de Arial.
pub struct Texto {
    pub tipo: usize,
    pub capa: usize,
    pub texto: String,
    pub visible: bool,
    caja: Rect,
    /// Tamaño en px (pt * 96 / 72) y negrita.
    px: f32,
    negrita: bool,
    color: Color,
    alineacion_h: i32,
    alineacion_v: i32,
    alto_linea: f32,
}

impl Texto {
    /// Text.Instance.onCreate: [texto, visibilidad inicial (0 = visible), fuente, color,
    /// alineación horizontal, vertical, origen, ajuste, separación de líneas].
    pub fn nuevo(instancia: &Instancia, capa: usize) -> Self {
        let p = &instancia.propiedades;
        let fuente = p[2].as_str().unwrap_or_default();
        let palabras: Vec<&str> = fuente.split(' ').collect();
        let pt: f32 = palabras
            .iter()
            .find_map(|w| w.strip_suffix("pt")?.parse().ok())
            .unwrap_or(12.0);
        let color = p[3]
            .as_str()
            .and_then(|c| c.strip_prefix("rgb(")?.strip_suffix(')').map(str::to_owned))
            .map(|c| {
                let v: Vec<u8> = c
                    .split(',')
                    .map(|n| n.trim().parse().unwrap_or(0))
                    .collect();
                Color::from_rgba(v[0], v[1], v[2], 255)
            })
            .unwrap_or(BLACK);
        let caja = Rect::new(
            instancia.x - instancia.origen.x * instancia.ancho,
            instancia.y - instancia.origen.y * instancia.alto,
            instancia.ancho,
            instancia.alto,
        );
        Self {
            tipo: instancia.tipo,
            capa,
            texto: p[0].as_str().unwrap_or_default().to_owned(),
            visible: p[1].as_u64() == Some(0),
            caja,
            px: pt / 72.0 * 96.0,
            // "bold" o la familia "Arial Black".
            negrita: fuente.contains("bold") || fuente.contains("Black"),
            color,
            alineacion_h: p[4].as_i64().unwrap_or(0) as i32,
            alineacion_v: p[5].as_i64().unwrap_or(0) as i32,
            // Math.ceil(pt / 72 * 96) + 4 + separación.
            alto_linea: (pt / 72.0 * 96.0).ceil() + 4.0 + p[8].as_f64().unwrap_or(0.0) as f32,
        }
    }

    /// Set text: los números se escriben como en JavaScript.
    pub fn set_texto_numero(&mut self, n: f64) {
        self.texto = if n.fract() == 0.0 && n.abs() < 1e15 {
            format!("{}", n as i64)
        } else {
            format!("{n}")
        };
    }

    /// Text.draw: líneas ajustadas a la caja, alineadas y con la línea base arriba.
    pub fn dibujar(&self, fuentes: &Fuentes) {
        if !self.visible {
            return;
        }
        let fuente = if self.negrita {
            &fuentes.negrita
        } else {
            &fuentes.normal
        };
        let tam = self.px.round() as u16;
        let escala = self.px / tam as f32;
        let ancho = |t: &str| measure_text(t, Some(fuente), tam, escala).width;
        let mut lineas: Vec<String> = Vec::new();
        for parrafo in self.texto.split('\n') {
            let mut linea = String::new();
            for palabra in parrafo.split(' ') {
                let prueba = if linea.is_empty() {
                    palabra.to_owned()
                } else {
                    format!("{linea} {palabra}")
                };
                if !linea.is_empty() && ancho(&prueba) > self.caja.w {
                    lineas.push(std::mem::replace(&mut linea, palabra.to_owned()));
                } else {
                    linea = prueba;
                }
            }
            lineas.push(linea);
        }
        let mut y = self.caja.y;
        let n = lineas.len() as f32;
        match self.alineacion_v {
            1 => y += (self.caja.h / 2.0 - n * self.alto_linea / 2.0).max(0.0),
            2 => y += (self.caja.h - n * self.alto_linea - 2.0).max(0.0),
            _ => {}
        }
        let fondo = self.caja.y + self.caja.h;
        for linea in &lineas {
            let w = ancho(linea);
            let x = match self.alineacion_h {
                1 => self.caja.x + (self.caja.w - w) / 2.0,
                2 => self.caja.x + (self.caja.w - w),
                _ => self.caja.x,
            };
            // textBaseline "top": la línea base queda un ascendente (0,905 em en Arial) más abajo.
            draw_text_ex(
                linea,
                x,
                y + self.px * 0.905,
                TextParams {
                    font: Some(fuente),
                    font_size: tam,
                    font_scale: escala,
                    color: self.color,
                    ..Default::default()
                },
            );
            y += self.alto_linea;
            if y >= fondo - self.alto_linea {
                break;
            }
        }
    }
}

pub struct Fuentes {
    pub normal: Font,
    pub negrita: Font,
}

/// Plugin Audio: cada Play crea (o reutiliza, si terminó) una instancia con su etiqueta. El
/// volumen está en dB (1 dB se recorta a volumen 1).
pub struct Audio {
    sonidos: HashMap<String, (Sound, f64)>,
    sonando: Vec<(String, f64)>,
}

impl Audio {
    pub async fn cargar(archivos: &[(&str, &[u8])]) -> Self {
        let mut sonidos = HashMap::new();
        for (nombre, datos) in archivos {
            let sonido = load_sound_from_bytes(datos).await.expect("ogg");
            sonidos.insert((*nombre).to_owned(), (sonido, duracion_ogg(datos)));
        }
        Self {
            sonidos,
            sonando: Vec::new(),
        }
    }

    pub fn play(&mut self, archivo: &str, volumen_db: f64, etiqueta: &str) {
        let volumen = 10f64.powf(volumen_db / 20.0).clamp(0.0, 1.0) as f32;
        let (sonido, duracion) = &self.sonidos[archivo];
        play_sound(
            sonido,
            PlaySoundParams {
                looped: false,
                volume: volumen,
            },
        );
        let ahora = get_time();
        self.sonando.retain(|(_, fin)| *fin > ahora);
        self.sonando.push((etiqueta.to_owned(), ahora + duracion));
    }

    /// Is tag playing.
    pub fn etiqueta_sonando(&self, etiqueta: &str) -> bool {
        let ahora = get_time();
        self.sonando
            .iter()
            .any(|(e, fin)| igual(e, etiqueta) && *fin > ahora)
    }
}

/// Duración de un Ogg Vorbis: la última posición de granulo entre la frecuencia de muestreo.
fn duracion_ogg(datos: &[u8]) -> f64 {
    let frecuencia = datos
        .windows(7)
        .position(|w| w == b"\x01vorbis")
        .map(|i| u32::from_le_bytes(datos[i + 12..i + 16].try_into().unwrap()))
        .unwrap_or(44100);
    let granulo = datos
        .windows(4)
        .rposition(|w| w == b"OggS")
        .map(|i| u64::from_le_bytes(datos[i + 6..i + 14].try_into().unwrap()))
        .unwrap_or(0);
    granulo as f64 / frecuencia as f64
}
