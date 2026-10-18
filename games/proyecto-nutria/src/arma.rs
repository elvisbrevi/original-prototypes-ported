// Port de Arma, Pistola y Bala. Las cajas siempre traen una Pistola, cuyo nombre es "Laser"; el
// Lanzallamas existe en el código pero ninguna caja lo crea, así que no se portó.

use macroquad::prelude::*;
use xna::{Rectangle, int};

use crate::assets::Assets;

#[derive(Clone, Copy, Default)]
pub struct Bala {
    pub posicion: Vec2,
    pub direccion: Vec2,
    pub origen: Vec2,
    pub cuerpo: Rectangle,
    pub activa: bool,
    pub rotacion: f32,
}

pub struct Arma {
    pub balas: Vec<Bala>,
    /// direccion == SpriteEffects.FlipHorizontally.
    pub voltear: bool,
    pub posicion: Vec2,
}

impl Arma {
    /// new Pistola() + cargar: 20 balas de "laser".
    pub fn pistola(posicion: Vec2) -> Self {
        Self {
            balas: vec![Bala::default(); 20],
            voltear: false,
            posicion,
        }
    }

    /// Pistola.disparar: mueve una bala activa. Ninguna bala se desactiva nunca.
    pub fn disparar(&mut self, i: usize, laser: &Texture2D) {
        let bala = &mut self.balas[i];
        if bala.activa {
            bala.posicion += bala.direccion;
            bala.cuerpo = Rectangle::new(
                int(bala.posicion.x),
                int(bala.posicion.y),
                laser.width() as i32,
                laser.height() as i32,
            );
            bala.origen = vec2(
                (bala.cuerpo.width / 2) as f32,
                (bala.cuerpo.height / 2) as f32,
            );
        }
    }

    pub fn dibujar(&self, assets: &Assets) {
        xna::dibujar(
            &assets.pistola,
            self.posicion,
            None,
            WHITE,
            0.0,
            Vec2::ZERO,
            0.4,
            self.voltear,
        );
        for bala in self.balas.iter().filter(|b| b.activa) {
            xna::dibujar(
                &assets.laser,
                bala.posicion + bala.origen,
                None,
                WHITE,
                bala.rotacion,
                bala.origen,
                1.0,
                false,
            );
        }
    }
}
