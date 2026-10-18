// Port de Proyecto_Nutria.Fisica. La lista de monedas del nivel siempre está vacía, así que su
// gravedad y sus colisiones (que nunca hacen nada) no se portaron.

use macroquad::prelude::*;
use xna::{MILISEGUNDOS_POR_PASO_F64, Rectangle, SEGUNDOS_POR_PASO_F64, clamp, int};

use crate::caja::Caja;
use crate::personaje::{Estado, Personaje};
use crate::plataforma::{Colision, Movimiento, Plataforma};

#[derive(Default)]
pub struct Fisica {
    gravedad_personaje: [f32; 2],
}

impl Fisica {
    pub fn actualizar(
        &mut self,
        personajes: &mut [Personaje],
        plataformas: &[Plataforma],
        cajas: &mut [Caja],
    ) {
        self.gravedad(personajes);
        self.colision_plataforma(plataformas, personajes);
        colision_cajas(cajas, personajes);
        colision_balas(personajes);
    }

    /// La gravedad solo vuelve a 0 al saltar: al aterrizar sigue sumando hasta 9 px por paso y la
    /// colisión con la plataforma devuelve al personaje a su superficie.
    fn gravedad(&mut self, personajes: &mut [Personaje]) {
        for (i, p) in personajes.iter_mut().enumerate() {
            if p.estado == Estado::Saltando {
                self.gravedad_personaje[i] = 0.0;
            }
            if p.estado != Estado::Saltando {
                p.posicion.y += self.gravedad_personaje[i];
                if self.gravedad_personaje[i] <= 8.0 {
                    self.gravedad_personaje[i] += 1.0;
                }
            }
        }
    }

    fn colision_plataforma(&self, plataformas: &[Plataforma], personajes: &mut [Personaje]) {
        for (i, p) in personajes.iter_mut().enumerate() {
            let g = self.gravedad_personaje[i];
            p.cuerpo = Rectangle::new(
                int(p.posicion.x),
                int(p.posicion.y),
                int(p.cuadro.width as f32 * p.zoom),
                int(p.cuadro.height as f32 * p.zoom),
            );
            p.origen = vec2((p.cuadro.width / 2) as f32, (p.cuadro.height / 2) as f32);
            p.colision_superior = false;
            p.colision_inferior = false;
            p.colision_derecha = false;
            p.colision_izquierda = false;
            for plataforma in plataformas {
                let c = plataforma.cuerpo;
                if !p.cuerpo.intersects(&c) {
                    continue;
                }
                if matches!(plataforma.colision, Colision::Lateral | Colision::Total) {
                    if p.posicion.x
                        >= c.x as f32 + (c.width as f32 - (g + p.velocidad_movimiento)) - 10.0
                        && p.posicion.y + p.cuerpo.height as f32
                            > c.y as f32 + (g + p.velocidad_movimiento)
                    {
                        p.colision_izquierda = true;
                        p.velocidad_movimiento = 0.0;
                        p.posicion.x = int(clamp(
                            p.posicion.x,
                            c.right() as f32,
                            p.cuerpo.right() as f32,
                        )) as f32;
                    }
                    if p.posicion.x + p.cuerpo.width as f32
                        <= c.x as f32 + (g + p.velocidad_movimiento) + 10.0
                        && p.posicion.y + p.cuerpo.height as f32
                            > c.y as f32 + (g + p.velocidad_movimiento) + 10.0
                    {
                        p.colision_derecha = true;
                        p.velocidad_movimiento = 0.0;
                        p.posicion.x =
                            int(clamp(p.posicion.x, 0.0, (c.left() - p.cuerpo.width) as f32))
                                as f32;
                    }
                }
                if matches!(plataforma.colision, Colision::Superior | Colision::Total)
                    && p.posicion.y + p.cuerpo.height as f32 <= c.y as f32 + (g + 1.0)
                    && p.posicion.x + p.cuerpo.width as f32 > c.x as f32
                {
                    p.posicion.y = (c.y - p.cuerpo.height) as f32;
                    p.colision_superior = true;
                    // Las plataformas que se mueven arrastran al personaje.
                    let sentido = if plataforma.ida { 1.0 } else { -1.0 };
                    match plataforma.movimiento {
                        Movimiento::Vertical => {
                            p.posicion.y += sentido * plataforma.velocidad_movimiento
                        }
                        Movimiento::Horizontal => {
                            p.posicion.x += sentido * plataforma.velocidad_movimiento
                        }
                        Movimiento::Ninguno => {}
                    }
                }
                if matches!(plataforma.colision, Colision::Inferior | Colision::Total)
                    && p.posicion.y >= c.y as f32 + (c.height as f32 - (g + p.velocidad_movimiento))
                    && p.posicion.x + p.cuerpo.width as f32 > c.x as f32
                {
                    // Clamp(Y, Bottom + 10, 0): con el Clamp invertido de XNA queda en Bottom + 10.
                    p.posicion.y = int(clamp(p.posicion.y, (c.bottom() + 10) as f32, 0.0)) as f32;
                    p.colision_inferior = true;
                }
            }
        }
    }
}

/// Fisica.colisionCajas: tocar una caja activa da su pistola con 10 balas.
fn colision_cajas(cajas: &mut [Caja], personajes: &mut [Personaje]) {
    for p in personajes.iter_mut() {
        for caja in cajas.iter_mut() {
            if p.cuerpo.intersects(&caja.cuerpo) && caja.activa {
                caja.activa = false;
                p.arma = caja.arma.take();
                p.armado = true;
                p.cantidad_balas = 10;
            }
        }
    }
}

/// Fisica.colisionBalas: las balas de un personaje golpean al otro, que sale despedido hacia
/// arriba girando y en rojo. Al quedarse sin vida la recupera, pero pierde 50 monedas.
fn colision_balas(personajes: &mut [Personaje]) {
    for i in 0..personajes.len() {
        let p = &mut personajes[i];
        if p.golpeado {
            if (p.posicion_golpe as f32) < p.posicion.y {
                p.posicion.y -= MILISEGUNDOS_POR_PASO_F64 as f32 * 1.5;
                p.rotacion += 0.5;
                p.color = xna::colores::ROJO;
            } else {
                p.rotacion = 0.0;
                p.golpeado = false;
            }
            p.duracion_golpe -= SEGUNDOS_POR_PASO_F64 * 1.0;
        } else {
            p.posicion_golpe = 0;
            p.duracion_golpe = 2.0;
            p.color = WHITE;
        }
        if p.vida <= 0.0 {
            p.vida = 200.0;
            p.monedas -= 50;
        }
        // El personaje i solo puede recibir balas del otro.
        let (blanco, tirador) = if i == 0 {
            let (a, b) = personajes.split_at_mut(1);
            (&mut a[0], &b[0])
        } else {
            let (a, b) = personajes.split_at_mut(1);
            (&mut b[0], &a[0])
        };
        let Some(arma) = &tirador.arma else {
            continue;
        };
        for bala in &arma.balas {
            if blanco.cuerpo.intersects(&bala.cuerpo) && bala.activa {
                blanco.vida -= 1.0;
                blanco.posicion_golpe = int(blanco.posicion.y - 250.0);
                blanco.golpeado = true;
            }
        }
    }
}
