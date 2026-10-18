// Port de Proyecto_Nutria.Camara y Camera2D: la cámara encuadra a los dos personajes, alejándose
// si están lejos, y sigue con retraso al que va más a la izquierda.

use macroquad::prelude::*;
use xna::{Rectangle, clamp};

use crate::personaje::Personaje;

pub struct Camara {
    /// Camera2D.position y Camera2D.scale (la rotación siempre es 0).
    pub posicion: Vec2,
    pub escala: f32,
    pub area: Rectangle,
    pub activo: bool,
    velocidad_camara: f32,
    /// Camera2D.transform: se calcula en Camera2D.Update, después de actualizar el nivel, y es la
    /// que usa Draw.
    pub transform: (Vec2, f32),
}

impl Camara {
    /// Camara.inicializar. Game1.Initialize pone la posición en (500, 500), pero la cámara es un
    /// GameComponent y base.Initialize llama después a Camera2D.Initialize, que la deja en (0, 0)
    /// con escala 1 (y desactiva la cámara manual de depuración, que no se portó).
    pub fn nueva() -> Self {
        Self {
            posicion: Vec2::ZERO,
            escala: 1.0,
            area: Rectangle::default(),
            activo: false,
            velocidad_camara: 6.0,
            transform: (Vec2::ZERO, 1.0),
        }
    }

    pub fn actualizar(&mut self, personajes: &[Personaje], limites: Rectangle) {
        let (a, b) = (&personajes[0], &personajes[1]);
        let mut x = a.posicion.x;
        let mut derecha = a.posicion.x + a.cuerpo.width as f32;
        let mut y = a.posicion.y;
        let mut abajo = a.posicion.y + a.cuerpo.height as f32;
        if self.activo {
            x = x.min(b.posicion.x);
            derecha = derecha.max(b.posicion.x + b.cuerpo.width as f32);
            y = y.min(b.posicion.y);
            abajo = abajo.max(b.posicion.y + b.cuerpo.height as f32);
            self.area =
                Rectangle::new(x as i32, y as i32, (derecha - x) as i32, (abajo - y) as i32);
        }
        if self.area.width > 1000 {
            self.escala = 1000.0 / self.area.width as f32;
        }
        let objetivo = self.area.x as f32 - 200.0 * self.escala;
        if self.posicion.x < objetivo - 7.0 {
            self.posicion.x += self.velocidad_camara;
            if self.velocidad_camara > 0.0 {
                self.velocidad_camara -= 0.0001;
            }
        } else if self.posicion.x > objetivo + 7.0 {
            self.posicion.x -= self.velocidad_camara;
            if self.velocidad_camara < 6.0 {
                self.velocidad_camara += 0.01;
            }
        }
        self.escala = clamp(self.escala, 0.38, 0.6);
        self.posicion.x = if self.area.x > 1200 {
            clamp(self.posicion.x, 0.0, 1070.0)
        } else {
            clamp(self.posicion.x, 0.0, limites.right() as f32)
        };
        self.posicion.y = clamp(self.posicion.y, 200.0 * self.escala, 200.0 * self.escala);
    }

    /// Camera2D.Update: transform = Scale(scale) * Translation(-position * scale).
    pub fn actualizar_transform(&mut self) {
        self.transform = (self.posicion, self.escala);
    }
}
