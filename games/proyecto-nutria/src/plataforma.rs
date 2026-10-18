// Port de Proyecto_Nutria.Plataforma.

use macroquad::prelude::*;
use xna::{Rectangle, int};

use crate::assets::Assets;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Movimiento {
    #[default]
    Ninguno,
    Vertical,
    Horizontal,
}

/// Textura de la plataforma ("Graficos/Plataformas/" + Diseño). Pasto existe pero el nivel no la usa.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Diseno {
    #[default]
    Null,
    Roca,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Colision {
    #[default]
    Superior,
    Lateral,
    /// Ninguna plataforma del nivel la usa, pero Fisica la contempla.
    #[allow(dead_code)]
    Inferior,
    Total,
}

#[derive(Default)]
pub struct Plataforma {
    pub cuerpo: Rectangle,
    pub posicion: Vec2,
    pub movimiento: Movimiento,
    pub posicion_final: f32,
    pub posicion_inicial: f32,
    pub velocidad_movimiento: f32,
    pub ida: bool,
    pub vuelta: bool,
    pub diseno: Diseno,
    pub colision: Colision,
}

impl Plataforma {
    /// Plataforma.cargar.
    pub fn cargar(&mut self) {
        self.ida = true;
        self.vuelta = false;
    }

    fn textura<'a>(&self, assets: &'a Assets) -> &'a Texture2D {
        match self.diseno {
            Diseno::Null => &assets.plataforma_null,
            Diseno::Roca => &assets.plataforma_roca,
        }
    }

    pub fn actualizar(&mut self, assets: &Assets) {
        if self.movimiento != Movimiento::Ninguno {
            self.mover();
            let t = self.textura(assets);
            self.cuerpo = Rectangle::new(
                int(self.posicion.x),
                int(self.posicion.y),
                t.width() as i32,
                t.height() as i32,
            );
        }
    }

    /// Solo se llama con plataformas que no son Null (las Null son invisibles).
    pub fn dibujar(&self, assets: &Assets) {
        xna::dibujar_en(self.textura(assets), self.cuerpo, WHITE);
    }

    fn mover(&mut self) {
        let eje = match self.movimiento {
            Movimiento::Horizontal => &mut self.posicion.x,
            Movimiento::Vertical => &mut self.posicion.y,
            Movimiento::Ninguno => return,
        };
        if self.ida {
            *eje += self.velocidad_movimiento;
            if *eje >= self.posicion_final {
                self.ida = false;
                self.vuelta = true;
            }
        } else if self.vuelta {
            *eje -= self.velocidad_movimiento;
            if *eje <= self.posicion_inicial {
                self.ida = true;
                self.vuelta = false;
            }
        }
    }
}
