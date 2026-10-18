// Port de Generador, Explosion y Particula: el "polvo" de hojas que levanta un personaje al
// correr sobre una plataforma.

use macroquad::prelude::*;
use xna::{Random, Rectangle, int};

use crate::tiempo::GameTime;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Estado {
    Activo,
    Inactivo,
}

#[derive(Clone, Copy, Default)]
struct Particula {
    posicion: Vec2,
    direccion: Vec2,
    origen: Vec2,
    cuerpo: Rectangle,
    zoom: f32,
    rotacion: f32,
    tiempo: f32,
    /// Color.White con el alfa como byte de C# (las restas dan la vuelta).
    alfa: u8,
    activa: bool,
}

impl Particula {
    /// Particula.actualizar: el cuerpo no se usa para nada más.
    fn actualizar(&mut self, textura: &Texture2D) {
        self.cuerpo = Rectangle::new(
            int(self.posicion.x),
            int(self.posicion.y),
            int(textura.width() * self.zoom),
            int(textura.height() * self.zoom),
        );
    }
}

/// Explosion (un Generador). Cada personaje tiene uno con 10 partículas de "hoja".
pub struct Explosion {
    particulas: Vec<Particula>,
    pub estado: Estado,
    random: Random,
    posicion: Vec2,
}

impl Explosion {
    /// Explosion.cargar(content, cantidad, textura).
    pub fn cargar(cantidad: usize, textura: &Texture2D) -> Self {
        let mut random = Random::con_reloj();
        let mut particulas: Vec<Particula> = (0..cantidad)
            .map(|_| Particula {
                alfa: 255,
                zoom: 0.1,
                direccion: vec2(random.next(-2, 2) as f32, random.next(-2, 2) as f32),
                // Particula.cargar: origen en el centro, con división entera.
                origen: vec2(
                    (textura.width() as i32 / 2) as f32,
                    (textura.height() as i32 / 2) as f32,
                ),
                ..Default::default()
            })
            .collect();
        particulas[cantidad - 1].tiempo = 5.0;
        Self {
            particulas,
            // Generador.Estado_ empieza en su primer valor, Activo.
            estado: Estado::Activo,
            random,
            posicion: Vec2::ZERO,
        }
    }

    pub fn actualizar(&mut self, posicion: Vec2, tiempo: &GameTime, textura: &Texture2D) {
        self.posicion = posicion;
        let n = self.particulas.len();
        for i in 0..n {
            // Particulas[i - 1] con i = 0 lanza IndexOutOfRangeException, y el catch mira la última.
            let anterior = if i == 0 { n - 1 } else { i - 1 };
            if self.particulas[anterior].tiempo >= 3.0 {
                self.mover_particula(i, tiempo, textura);
            }
            self.reiniciar_particula(i);
        }
    }

    pub fn dibujar(&self, textura: &Texture2D) {
        for p in self.particulas.iter().filter(|p| p.activa) {
            xna::dibujar(
                textura,
                p.posicion,
                None,
                Color::from_rgba(255, 255, 255, p.alfa),
                p.rotacion,
                p.origen,
                p.zoom,
                false,
            );
        }
    }

    /// Generador.moverParticula. El tiempo de cada partícula suma TotalGameTime (no el tiempo
    /// del paso), así que crece cada vez más rápido.
    fn mover_particula(&mut self, i: usize, tiempo: &GameTime, textura: &Texture2D) {
        let resta = if self.estado == Estado::Inactivo {
            self.random.next(5, 5)
        } else {
            self.random.next(3, 5)
        };
        let p = &mut self.particulas[i];
        p.tiempo += tiempo.total_segundos() as f32;
        p.actualizar(textura);
        p.posicion += p.direccion;
        p.alfa = p.alfa.wrapping_sub(resta as u8);
        if self.estado == Estado::Activo {
            p.zoom += 0.01;
        }
        p.rotacion += 0.1;
    }

    fn reiniciar_particula(&mut self, i: usize) {
        let p = &mut self.particulas[i];
        if p.alfa <= 5 {
            p.tiempo = 0.0;
            p.posicion = self.posicion;
            p.alfa = 254;
            p.zoom = 0.1;
            p.activa = self.estado == Estado::Activo;
        }
    }
}
