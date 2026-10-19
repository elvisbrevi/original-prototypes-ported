// Plugin Particles: un emisor que lanza partículas (la misma textura, cuadrada) en un cono, con
// velocidad, aceleración, gravedad, crecimiento y desvanecimiento.

use std::collections::HashMap;

use macroquad::prelude::*;

use crate::datos::Instancia;
use crate::geometria::a_radianes;

fn aleatorio() -> f64 {
    rand::gen_range(0.0, 1.0)
}

#[derive(Clone, Copy, Debug)]
struct Particula {
    x: f64,
    y: f64,
    velocidad: f64,
    angulo: f64,
    opacidad: f64,
    crecimiento: f64,
    tamano: f64,
    caida: f64,
    edad: f64,
    viva: bool,
}

pub struct Particulas {
    pub uid: u32,
    pub tipo: usize,
    pub capa: usize,
    pub x: f64,
    pub y: f64,
    /// Dirección del chorro (radianes).
    pub angulo: f64,
    pub opacidad: f64,
    pub visible: bool,
    pub destruido: bool,
    pub textura: String,
    ritmo: f64,
    cono: f64,
    /// 0: continuo; 1: una ráfaga.
    tipo_disparo: i32,
    pub pulverizando: bool,
    velocidad: f64,
    tamano: f64,
    opacidad_inicial: f64,
    crecimiento: f64,
    aleatorio_x: f64,
    aleatorio_y: f64,
    aleatorio_velocidad: f64,
    aleatorio_tamano: f64,
    aleatorio_crecimiento: f64,
    aceleracion: f64,
    gravedad: f64,
    aleatorio_angulo: f64,
    aleatorio_velocidad_vuelo: f64,
    aleatorio_opacidad: f64,
    /// 0: se desvanece hasta el tiempo de vida; 1: desaparece al cumplirlo; 2: al pararse.
    modo_destruir: i32,
    tiempo_vida: f64,
    acumulado: f64,
    primera: bool,
    particulas: Vec<Particula>,
}

impl Particulas {
    /// onCreate: [ritmo, cono (grados), tipo, velocidad, tamaño, opacidad (%), crecimiento,
    /// aleatoriedad de x, de y, de velocidad, de tamaño, de crecimiento, aceleración, gravedad,
    /// aleatoriedad del ángulo, de la velocidad y de la opacidad en vuelo, modo de destruir,
    /// tiempo de vida].
    pub fn nueva(
        instancia: &Instancia,
        textura: &str,
        capa: usize,
        uid: u32,
        x: f64,
        y: f64,
    ) -> Self {
        let p = &instancia.propiedades;
        let g = |i: usize| p.get(i).and_then(|v| v.as_f64()).unwrap_or_default();
        let mut s = Self {
            uid,
            tipo: instancia.tipo,
            capa,
            x,
            y,
            angulo: instancia.angulo,
            opacidad: instancia.opacidad,
            visible: true,
            destruido: false,
            textura: textura.to_owned(),
            ritmo: g(0),
            cono: a_radianes(g(1)),
            tipo_disparo: g(2) as i32,
            pulverizando: true,
            velocidad: g(3),
            tamano: g(4),
            opacidad_inicial: g(5) / 100.0,
            crecimiento: g(6),
            aleatorio_x: g(7),
            aleatorio_y: g(8),
            aleatorio_velocidad: g(9),
            aleatorio_tamano: g(10),
            aleatorio_crecimiento: g(11),
            aceleracion: g(12),
            gravedad: g(13),
            aleatorio_angulo: g(14),
            aleatorio_velocidad_vuelo: g(15),
            aleatorio_opacidad: g(16),
            modo_destruir: g(17) as i32,
            tiempo_vida: g(18),
            acumulado: 0.0,
            primera: true,
            particulas: Vec::new(),
        };
        if s.tipo_disparo == 1 {
            for _ in 0..s.ritmo as usize {
                let mut q = s.nueva_particula();
                q.opacidad = 0.0;
                s.particulas.push(q);
            }
        }
        s
    }

    /// particle.init: los aleatorios en el orden del original.
    fn nueva_particula(&self) -> Particula {
        let x = self.x - self.aleatorio_x / 2.0 + aleatorio() * self.aleatorio_x;
        let y = self.y - self.aleatorio_y / 2.0 + aleatorio() * self.aleatorio_y;
        let velocidad = self.velocidad - self.aleatorio_velocidad / 2.0
            + aleatorio() * self.aleatorio_velocidad;
        let angulo = self.angulo - self.cono / 2.0 + aleatorio() * self.cono;
        let tamano =
            self.tamano - self.aleatorio_tamano / 2.0 + aleatorio() * self.aleatorio_tamano;
        let crecimiento = self.crecimiento - self.aleatorio_crecimiento / 2.0
            + aleatorio() * self.aleatorio_crecimiento;
        Particula {
            x,
            y,
            velocidad,
            angulo,
            opacidad: self.opacidad_inicial,
            crecimiento,
            tamano,
            caida: 0.0,
            edad: 0.0,
            viva: true,
        }
    }

    /// tick. Devuelve si una ráfaga ya terminó (el emisor se destruye).
    pub fn tick(&mut self, dt: f64) -> bool {
        if self.tipo_disparo == 0 && self.pulverizando {
            self.acumulado += dt * self.ritmo;
            let n = self.acumulado.floor();
            self.acumulado -= n;
            for _ in 0..n as usize {
                let q = self.nueva_particula();
                self.particulas.push(q);
            }
        }
        let primera = self.primera;
        let mut vivas = Vec::with_capacity(self.particulas.len());
        for mut q in std::mem::take(&mut self.particulas) {
            if self.tipo_disparo == 1 && primera {
                q = self.nueva_particula();
            }
            self.mover(&mut q, dt);
            if q.viva {
                vivas.push(q);
            }
        }
        self.particulas = vivas;
        self.primera = false;
        self.tipo_disparo == 1 && self.particulas.is_empty()
    }

    fn mover(&self, q: &mut Particula, dt: f64) {
        q.x += q.angulo.cos() * q.velocidad * dt;
        q.y += q.angulo.sin() * q.velocidad * dt;
        q.y += q.caida * dt;
        q.velocidad += self.aceleracion * dt;
        q.tamano += q.crecimiento * dt;
        q.caida += self.gravedad * dt;
        q.edad += dt;
        if q.tamano < 1.0 {
            q.viva = false;
            return;
        }
        if self.aleatorio_angulo != 0.0 {
            q.angulo += aleatorio() * self.aleatorio_angulo * dt - self.aleatorio_angulo * dt / 2.0;
        }
        if self.aleatorio_velocidad_vuelo != 0.0 {
            q.velocidad += aleatorio() * self.aleatorio_velocidad_vuelo * dt
                - self.aleatorio_velocidad_vuelo * dt / 2.0;
        }
        if self.aleatorio_opacidad != 0.0 {
            q.opacidad +=
                aleatorio() * self.aleatorio_opacidad * dt - self.aleatorio_opacidad * dt / 2.0;
            q.opacidad = q.opacidad.clamp(0.0, 1.0);
        }
        if self.modo_destruir <= 1 && q.edad >= self.tiempo_vida {
            q.viva = false;
        }
        if self.modo_destruir == 2 && q.velocidad <= 0.0 {
            q.viva = false;
        }
    }

    /// drawGL: cada partícula, cuadrada y centrada en su posición.
    pub fn dibujar(
        &self,
        texturas: &HashMap<String, Texture2D>,
        desplazamiento: macroquad::math::DVec2,
    ) {
        if !self.visible {
            return;
        }
        let textura = &texturas[&self.textura];
        for q in &self.particulas {
            let mut alfa = self.opacidad * q.opacidad;
            if self.modo_destruir == 0 {
                alfa *= 1.0 - q.edad / self.tiempo_vida;
            }
            if q.tamano < 1.0 || alfa == 0.0 {
                continue;
            }
            draw_texture_ex(
                textura,
                (q.x - q.tamano / 2.0 + desplazamiento.x) as f32,
                (q.y - q.tamano / 2.0 + desplazamiento.y) as f32,
                Color::new(1.0, 1.0, 1.0, alfa as f32),
                DrawTextureParams {
                    dest_size: Some(vec2(q.tamano as f32, q.tamano as f32)),
                    ..Default::default()
                },
            );
        }
    }
}
