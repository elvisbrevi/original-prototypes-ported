// SpriteFont de XNA 3.1 (exportada por xnb_extract.py a PNG + texto), dibujada con el mismo
// algoritmo de SpriteFont.InternalDraw (kerning, recorte y espaciado).

use std::collections::HashMap;

use macroquad::prelude::*;

use crate::Rectangle;

struct Glifo {
    glifo: Rect,
    recorte: Rectangle,
    kerning: Vec3,
}

pub struct SpriteFont {
    textura: Texture2D,
    line_spacing: f32,
    spacing: f32,
    glifos: HashMap<char, Glifo>,
    por_defecto: Option<char>,
}

impl SpriteFont {
    /// `datos` es el .txt que escribe xnb_extract.py junto a la textura.
    pub fn nueva(textura: Texture2D, datos: &str) -> Self {
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
