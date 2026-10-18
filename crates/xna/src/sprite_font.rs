// SpriteFont de XNA 3.1 (exportada por xnb_extract.py a PNG + texto), dibujada con el mismo
// algoritmo de SpriteFont.InternalDraw (kerning, recorte y espaciado).

use std::collections::HashMap;

use macroquad::prelude::*;

use crate::Rectangle;

struct Glifo {
    glifo: Rectangle,
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
                let rect = |i: usize| {
                    Rectangle::new(
                        n[i] as i32,
                        n[i + 1] as i32,
                        n[i + 2] as i32,
                        n[i + 3] as i32,
                    )
                };
                let glifo = Glifo {
                    glifo: rect(1),
                    recorte: rect(5),
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

    /// GetIndexForCharacter: el carácter, o el de por defecto si la fuente no lo tiene.
    fn glifo(&self, c: char) -> Option<&Glifo> {
        self.glifos
            .get(&c)
            .or_else(|| self.glifos.get(&self.por_defecto?))
    }

    /// SpriteBatch.DrawString(fuente, texto, posición, color).
    pub fn dibujar(&self, texto: &str, posicion: Vec2, color: Color) {
        self.dibujar_ex(
            texto,
            posicion,
            color,
            0.0,
            Vec2::ZERO,
            1.0,
            crate::Efectos::NINGUNO,
        );
    }

    /// SpriteFont.MeasureString (InternalMeasure).
    pub fn medir(&self, texto: &str) -> Vec2 {
        if texto.is_empty() {
            return Vec2::ZERO;
        }
        let mut tamano = vec2(0.0, self.line_spacing);
        let mut ancho_maximo = 0.0_f32;
        let mut lineas = 0;
        let mut derecha = 0.0_f32;
        let mut primero = true;
        for c in texto.chars() {
            match c {
                '\r' => continue,
                '\n' => {
                    tamano.x += derecha.max(0.0);
                    derecha = 0.0;
                    ancho_maximo = ancho_maximo.max(tamano.x);
                    tamano = vec2(0.0, self.line_spacing);
                    primero = true;
                    lineas += 1;
                    continue;
                }
                _ => {}
            }
            let Some(g) = self.glifo(c) else { continue };
            let mut kerning = g.kerning;
            if primero {
                kerning.x = kerning.x.max(0.0);
            } else {
                tamano.x += self.spacing + derecha;
            }
            tamano.x += kerning.x + kerning.y;
            derecha = kerning.z;
            tamano.y = tamano.y.max(g.recorte.height as f32);
            primero = false;
        }
        tamano.x += derecha.max(0.0);
        tamano.y += lineas as f32 * self.line_spacing;
        tamano.x = tamano.x.max(ancho_maximo);
        tamano
    }

    /// SpriteBatch.DrawString con rotación, origen, escala y efectos: el mismo algoritmo de
    /// SpriteFont.InternalDraw de XNA 3.1 (al voltear se recorre el texto al revés).
    #[allow(clippy::too_many_arguments)]
    pub fn dibujar_ex(
        &self,
        texto: &str,
        posicion: Vec2,
        color: Color,
        rotacion: f32,
        origen: Vec2,
        escala: f32,
        efectos: crate::Efectos,
    ) {
        let (seno, coseno) = rotacion.sin_cos();
        // Translation(-origen * escala) seguida de RotationZ(rotación).
        let transformar = |p: Vec2| {
            let p = p - origen * escala;
            vec2(p.x * coseno - p.y * seno, p.x * seno + p.y * coseno)
        };
        let mut signo = 1.0;
        let mut inicio_x = 0.0;
        if efectos.horizontal {
            inicio_x = self.medir(texto).x * escala;
            signo = -1.0;
        }
        let mut cursor = vec2(inicio_x, 0.0);
        if efectos.vertical {
            cursor.y = (self.medir(texto).y - self.line_spacing) * escala;
        }
        let mut primero = true;
        for c in texto.chars() {
            match c {
                '\n' => {
                    primero = true;
                    cursor.x = inicio_x;
                    cursor.y +=
                        if efectos.vertical { -1.0 } else { 1.0 } * self.line_spacing * escala;
                    continue;
                }
                '\r' => continue,
                _ => {}
            }
            let Some(g) = self.glifo(c) else { continue };
            let mut kerning = g.kerning;
            if primero {
                kerning.x = kerning.x.max(0.0);
            } else {
                cursor.x += self.spacing * escala * signo;
            }
            cursor.x += kerning.x * escala * signo;
            let mut recorte = g.recorte;
            if efectos.vertical {
                recorte.y = self.line_spacing as i32 - g.glifo.height - recorte.y;
            }
            if efectos.horizontal {
                recorte.x -= recorte.width;
            }
            let destino = transformar(cursor + vec2(recorte.x as f32, recorte.y as f32) * escala);
            crate::dibujar(
                &self.textura,
                posicion + destino,
                Some(g.glifo),
                color,
                rotacion,
                Vec2::ZERO,
                escala,
                efectos,
            );
            primero = false;
            cursor.x += (kerning.y + kerning.z) * escala * signo;
        }
    }
}

fn numeros(linea: &str) -> Vec<f32> {
    linea
        .split_whitespace()
        .map(|n| n.parse().expect("número en fuente.txt"))
        .collect()
}
