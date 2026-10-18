// Port de Iman.Graficos: capas con parallax (Cielo, Background, Suelo, Front) y Ramas.

use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tipo {
    Suelo,
    Background,
    Front,
    Cielo,
}

pub struct Grafico {
    tipo: Tipo,
    textura: Texture2D,
    posicion: Vec2,
    zoom: f32,
    inicial: Vec2,
}

impl Grafico {
    pub fn nuevo(tipo: Tipo, textura: Texture2D, posicion: Vec2, zoom: f32) -> Self {
        Self {
            tipo,
            textura,
            posicion,
            zoom,
            inicial: posicion,
        }
    }

    pub fn actualizar(&mut self, camara: Vec2) {
        match self.tipo {
            Tipo::Background => self.posicion = camara * 0.5 + self.inicial,
            Tipo::Front => self.posicion = camara * -0.5 + self.inicial,
            Tipo::Suelo | Tipo::Cielo => {}
        }
    }

    pub fn dibujar(&self, camara: Vec2) {
        // El cielo va pegado a la cámara.
        let posicion = if self.tipo == Tipo::Cielo {
            camara
        } else {
            self.posicion
        };
        draw_texture_ex(
            &self.textura,
            posicion.x,
            posicion.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(self.textura.size() * self.zoom),
                ..Default::default()
            },
        );
    }
}

pub struct Ramas {
    textura: Texture2D,
    pub rotacion: f32,
    pub posicion: Vec2,
    pub voltear_x: bool,
    pub voltear_y: bool,
    ida: bool,
}

impl Ramas {
    pub fn nueva(
        textura: Texture2D,
        posicion: Vec2,
        rotacion: f32,
        voltear_x: bool,
        voltear_y: bool,
    ) -> Self {
        Self {
            textura,
            rotacion,
            posicion,
            voltear_x,
            voltear_y,
            ida: false,
        }
    }

    pub fn actualizar(&mut self) {
        if self.ida {
            self.rotacion += 0.001;
            if self.rotacion >= 0.1 {
                self.ida = false;
            }
        } else {
            self.rotacion -= 0.001;
            if self.rotacion <= -0.1 {
                self.ida = true;
            }
        }
    }

    /// Dibujada con origen en el centro de la textura (gira alrededor de `posicion`).
    pub fn dibujar(&self) {
        let origen = vec2(
            (self.textura.width() as i32 / 2) as f32,
            (self.textura.height() as i32 / 2) as f32,
        );
        let esquina = self.posicion - origen;
        draw_texture_ex(
            &self.textura,
            esquina.x,
            esquina.y,
            WHITE,
            DrawTextureParams {
                rotation: self.rotacion,
                flip_x: self.voltear_x,
                flip_y: self.voltear_y,
                pivot: Some(self.posicion),
                ..Default::default()
            },
        );
    }
}
