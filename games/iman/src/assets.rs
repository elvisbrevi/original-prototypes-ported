// Contenido extraído de los .xnb originales con tools/xnb_extract.py, embebido en el binario.

use std::collections::HashMap;

use macroquad::audio::{Sound, load_sound_from_bytes};
use macroquad::prelude::*;

use crate::xna::Rectangle;

macro_rules! textura {
    ($ruta:literal) => {
        Texture2D::from_file_with_format(include_bytes!(concat!("../assets/", $ruta)), None)
    };
}

pub struct Assets {
    pub nino: Texture2D,
    pub iman: Texture2D,
    pub titulo: Texture2D,
    pub cortina: Texture2D,
    pub ramas: Texture2D,
    pub sketch: Texture2D,
    pub cielos: [Texture2D; 2],
    pub backgrounds: [Texture2D; 2],
    pub suelos: [Texture2D; 2],
    pub fronts: [Texture2D; 2],
    /// Indexado por Diseno: NULL, _01.._04 (_05 existe en el contenido pero ningún escenario la usa).
    pub plataformas: [Texture2D; 5],
    pub fuente: SpriteFont,
    pub salto: Sound,
}

impl Assets {
    pub async fn cargar() -> Self {
        let sketch = textura!("shaders/sketch.png");
        // El sampler SketchSampler del efecto no define filtro: D3D9 usa POINT por defecto.
        sketch.set_filter(FilterMode::Nearest);
        Self {
            nino: textura!("personajes/nino.png"),
            iman: textura!("personajes/iman.png"),
            titulo: textura!("menu/titulo.png"),
            cortina: textura!("menu/cortina.png"),
            ramas: textura!("ramas/01.png"),
            sketch,
            cielos: [textura!("cielos/01.png"), textura!("cielos/02.png")],
            backgrounds: [
                textura!("backgrounds/01.png"),
                textura!("backgrounds/02.png"),
            ],
            suelos: [textura!("suelos/01.png"), textura!("suelos/02.png")],
            fronts: [textura!("fronts/01.png"), textura!("fronts/02.png")],
            plataformas: [
                textura!("plataformas/null.png"),
                textura!("plataformas/01.png"),
                textura!("plataformas/02.png"),
                textura!("plataformas/03.png"),
                textura!("plataformas/04.png"),
            ],
            fuente: SpriteFont::nueva(
                textura!("fuente/fuente.png"),
                include_str!("../assets/fuente/fuente.txt"),
            ),
            salto: load_sound_from_bytes(include_bytes!("../assets/audio/jump.wav"))
                .await
                .expect("jump.wav"),
        }
    }
}

struct Glifo {
    glifo: Rect,
    recorte: Rectangle,
    kerning: Vec3,
}

/// SpriteFont de XNA 3.1 (Textos/fuente) dibujada con el mismo algoritmo de SpriteFont.InternalDraw.
pub struct SpriteFont {
    textura: Texture2D,
    line_spacing: f32,
    spacing: f32,
    glifos: HashMap<char, Glifo>,
    por_defecto: Option<char>,
}

impl SpriteFont {
    fn nueva(textura: Texture2D, datos: &str) -> Self {
        let mut lineas = datos.lines().filter(|l| !l.starts_with('#'));
        let cabecera: Vec<f32> = numeros(lineas.next().expect("cabecera de fuente"));
        let glifos = lineas
            .map(|linea| {
                let n = numeros(linea);
                let c = char::from_u32(n[0] as u32).expect("carácter de fuente");
                let glifo = Glifo {
                    glifo: Rect::new(n[1], n[2], n[3], n[4]),
                    recorte: Rectangle::new(n[5] as i32, n[6] as i32, n[7] as i32, n[8] as i32),
                    kerning: vec3(n[9], n[10], n[11]),
                };
                (c, glifo)
            })
            .collect();
        Self {
            textura,
            line_spacing: cabecera[0],
            spacing: cabecera[1],
            glifos,
            por_defecto: char::from_u32(cabecera[2] as u32).filter(|_| cabecera[2] >= 0.0),
        }
    }

    pub fn dibujar(&self, texto: &str, posicion: Vec2, color: Color) {
        let mut cursor = Vec2::ZERO;
        let mut primero = true;
        for c in texto.chars() {
            match c {
                '\r' => continue,
                '\n' => {
                    cursor.x = 0.0;
                    cursor.y += self.line_spacing;
                    primero = true;
                    continue;
                }
                _ => {}
            }
            let Some(g) = self
                .glifos
                .get(&c)
                .or_else(|| self.glifos.get(&self.por_defecto?))
            else {
                continue;
            };
            let mut kerning = g.kerning;
            if primero {
                kerning.x = kerning.x.max(0.0);
            } else {
                cursor.x += self.spacing;
            }
            cursor.x += kerning.x;
            let destino = posicion + cursor + vec2(g.recorte.x as f32, g.recorte.y as f32);
            draw_texture_ex(
                &self.textura,
                destino.x,
                destino.y,
                color,
                DrawTextureParams {
                    source: Some(g.glifo),
                    ..Default::default()
                },
            );
            primero = false;
            cursor.x += kerning.y + kerning.z;
        }
    }
}

fn numeros(linea: &str) -> Vec<f32> {
    linea
        .split_whitespace()
        .map(|n| n.parse().expect("número en fuente.txt"))
        .collect()
}
