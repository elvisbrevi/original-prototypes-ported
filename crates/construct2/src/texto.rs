// Instancia de Text. Construct 2 la dibuja con la fuente del sistema en un canvas 2D (línea base
// "top"); el port usa Liberation Sans, con las métricas de Arial.

use macroquad::math::DVec2;
use macroquad::prelude::*;

use crate::datos::Instancia;

pub struct Fuentes {
    pub normal: Font,
    pub negrita: Font,
}

impl Fuentes {
    /// Liberation Sans 2.1.5 (normal y negrita, SIL OFL 1.1, ver fuentes/LICENCIA-OFL.txt): tiene
    /// las métricas de Arial, la fuente del sistema que usaban los juegos.
    pub fn liberation() -> Self {
        Self {
            normal: load_ttf_font_from_bytes(include_bytes!(
                "../fuentes/LiberationSans-Regular.ttf"
            ))
            .expect("fuente"),
            negrita: load_ttf_font_from_bytes(include_bytes!("../fuentes/LiberationSans-Bold.ttf"))
                .expect("fuente"),
        }
    }
}

pub struct Texto {
    pub uid: u32,
    pub tipo: usize,
    pub capa: usize,
    pub texto: String,
    pub visible: bool,
    pub x: f64,
    pub y: f64,
    pub ancho: f64,
    pub alto: f64,
    /// Origen (hotspot) en fracciones del tamaño.
    origen: DVec2,
    /// Tamaño en px (pt * 96 / 72) y negrita.
    px: f32,
    negrita: bool,
    color: Color,
    alineacion_h: i32,
    alineacion_v: i32,
    alto_linea: f32,
    pub destruido: bool,
}

impl Texto {
    /// Text.Instance.onCreate: [texto, visibilidad inicial (0 = visible), fuente, color,
    /// alineación horizontal, vertical, origen, ajuste, separación de líneas].
    pub fn nuevo(instancia: &Instancia, capa: usize, uid: u32, x: f64, y: f64) -> Self {
        let p = &instancia.propiedades;
        let fuente = p[2].as_str().unwrap_or_default();
        let pt: f32 = fuente
            .split(' ')
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
        Self {
            uid,
            tipo: instancia.tipo,
            capa,
            texto: p[0].as_str().unwrap_or_default().to_owned(),
            visible: p[1].as_u64() == Some(0),
            x,
            y,
            ancho: instancia.ancho,
            alto: instancia.alto,
            origen: instancia.origen,
            px: pt / 72.0 * 96.0,
            // "bold" o la familia "Arial Black".
            negrita: fuente.contains("bold") || fuente.contains("Black"),
            color,
            alineacion_h: p[4].as_i64().unwrap_or(0) as i32,
            alineacion_v: p[5].as_i64().unwrap_or(0) as i32,
            // Math.ceil(pt / 72 * 96) + 4 + separación.
            alto_linea: (pt / 72.0 * 96.0).ceil() + 4.0 + p[8].as_f64().unwrap_or(0.0) as f32,
            destruido: false,
        }
    }

    /// Set text con un número.
    pub fn set_texto_numero(&mut self, n: f64) {
        self.texto = numero_a_texto(n);
    }

    /// La caja del texto: el origen cae en (x, y).
    pub fn caja(&self) -> Rect {
        Rect::new(
            (self.x - self.origen.x * self.ancho) as f32,
            (self.y - self.origen.y * self.alto) as f32,
            self.ancho as f32,
            self.alto as f32,
        )
    }

    /// Text.draw: líneas ajustadas a la caja, alineadas y con la línea base arriba.
    pub fn dibujar(&self, fuentes: &Fuentes) {
        if !self.visible || self.ancho == 0.0 || self.alto == 0.0 {
            return;
        }
        let caja = self.caja();
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
                if !linea.is_empty() && ancho(&prueba) > caja.w {
                    lineas.push(std::mem::replace(&mut linea, palabra.to_owned()));
                } else {
                    linea = prueba;
                }
            }
            lineas.push(linea);
        }
        let mut y = caja.y;
        let n = lineas.len() as f32;
        match self.alineacion_v {
            1 => y += (caja.h / 2.0 - n * self.alto_linea / 2.0).max(0.0),
            2 => y += (caja.h - n * self.alto_linea - 2.0).max(0.0),
            _ => {}
        }
        let fondo = caja.y + caja.h;
        for linea in &lineas {
            let w = ancho(linea);
            let x = match self.alineacion_h {
                1 => caja.x + (caja.w - w) / 2.0,
                2 => caja.x + (caja.w - w),
                _ => caja.x,
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

/// SetText con un número: si es menor que 1e9 se redondea a 10 decimales, y se escribe como en
/// JavaScript (sin decimales si es entero).
pub fn numero_a_texto(n: f64) -> String {
    let n = if n < 1e9 {
        crate::geometria::redondear(n * 1e10) / 1e10
    } else {
        n
    };
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}
