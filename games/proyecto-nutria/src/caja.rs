// Port de Proyecto_Nutria.Caja: la caja que da una pistola. Al recogerla desaparece y vuelve a
// aparecer (con un fundido) unos segundos después.

use macroquad::prelude::*;
use xna::{Rectangle, SEGUNDOS_POR_PASO_F64};

use crate::arma::Arma;
use crate::assets::Assets;

pub struct Caja {
    /// Alfa de `color` (Color.White) como byte de C#.
    alfa: u8,
    pub cuerpo: Rectangle,
    pub posicion: Vec2,
    pub activa: bool,
    tiempo_activacion: f64,
    pub arma: Option<Arma>,
}

impl Caja {
    /// Nivel.crearCaja + Caja.cargar.
    pub fn nueva(posicion: Vec2, assets: &Assets) -> Self {
        Self {
            alfa: 255,
            cuerpo: Rectangle::new(
                posicion.x as i32,
                posicion.y as i32,
                assets.caja.width() as i32,
                assets.caja.height() as i32,
            ),
            posicion,
            activa: true,
            tiempo_activacion: 0.0,
            arma: Some(Arma::pistola(posicion)),
        }
    }

    pub fn actualizar(&mut self) {
        self.aparicion();
        if self.arma.is_none() {
            self.arma = Some(Arma::pistola(self.posicion));
        }
    }

    pub fn dibujar(&self, assets: &Assets) {
        draw_texture(
            &assets.caja,
            self.posicion.x,
            self.posicion.y,
            Color::from_rgba(255, 255, 255, self.alfa),
        );
    }

    /// Caja.aparicion: invisible hasta los 6 s y luego un fundido de dos en dos.
    fn aparicion(&mut self) {
        if self.tiempo_activacion >= 5.0 {
            self.alfa = self.alfa.wrapping_add(1);
            if self.alfa < 254 {
                self.alfa = self.alfa.wrapping_add(1);
            } else {
                self.activa = true;
                self.tiempo_activacion = 0.0;
            }
        }
        if !self.activa && self.tiempo_activacion < 6.0 {
            self.alfa = 0;
            if self.tiempo_activacion < 6.0 {
                self.tiempo_activacion += SEGUNDOS_POR_PASO_F64;
            }
        }
    }
}
