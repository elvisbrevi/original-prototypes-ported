// Port de Proyecto_Nutria.Iconos.IconoPersonajes: retrato, barra de vida, nombre y monedas de
// cada personaje en el marco de la pantalla. (El Cronometro del mismo espacio de nombres se
// actualiza en el original pero nunca se dibuja, así que no se portó.)

use macroquad::prelude::*;
use xna::Rectangle;
use xna::colores::{CORNFLOWER_BLUE, ORO, ROJO};

use crate::assets::Assets;
use crate::personaje::Personaje;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Posicion {
    SuperiorIzquierda,
    SuperiorDerecha,
}

/// Solo se actualiza el icono de los personajes que tienen control: con un solo teclado la barra
/// del pollo se queda con ancho 0 y no se ve.
#[derive(Default)]
pub struct IconoPersonajes {
    rectangulo_vida: Rectangle,
}

impl IconoPersonajes {
    pub fn actualizar(&mut self, p: &Personaje) {
        self.rectangulo_vida = Rectangle::new(0, 0, p.vida as i32, 20);
    }

    pub fn dibujar(&self, p: &Personaje, posicion: Posicion, assets: &Assets) {
        // (retrato, esfera, barra, marco, nombre, moneda, número)
        let [retrato, esfera, barra, marco, nombre, moneda, numero] = match posicion {
            Posicion::SuperiorIzquierda => [
                (20.0, 20.0),
                (0.0, 0.0),
                (105.0, 15.0),
                (100.0, 10.0),
                (10.0, 70.0),
                (150.0, 40.0),
                (200.0, 50.0),
            ],
            Posicion::SuperiorDerecha => [
                (1180.0, 20.0),
                (1160.0, 0.0),
                (975.0, 15.0),
                (970.0, 10.0),
                (1180.0, 70.0),
                (1000.0, 40.0),
                (1050.0, 50.0),
            ],
        }
        .map(|(x, y)| vec2(x, y));
        let normal = |textura, en: Vec2, fuente, color| {
            xna::dibujar(textura, en, fuente, color, 0.0, Vec2::ZERO, 1.0, false)
        };
        normal(p.textura(assets), retrato, Some(p.cuadro), WHITE);
        normal(&assets.esfera, esfera, None, WHITE);
        // La barra es la textura "null" de 50x50 recortada a (vida x 20): con el modo Clamp de
        // SpriteBatch se estira su último píxel.
        normal(
            &assets.plataforma_null,
            barra,
            Some(self.rectangulo_vida),
            ROJO,
        );
        normal(&assets.marco_icono, marco, None, WHITE);
        assets.fuente.dibujar(p.nombre, nombre, CORNFLOWER_BLUE);
        xna::dibujar(
            &assets.moneda,
            moneda,
            None,
            ORO,
            0.0,
            Vec2::ZERO,
            0.3,
            false,
        );
        assets.fuente.dibujar(&p.monedas.to_string(), numero, ORO);
    }
}
