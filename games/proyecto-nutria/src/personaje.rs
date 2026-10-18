// Port de Personaje, Coipo y Pollo (las dos subclases son idénticas salvo el nombre y la textura,
// y en este prototipo la textura del pollo es la misma del coipo).

use macroquad::prelude::*;
use xna::{Efectos, Rectangle, SEGUNDOS_POR_PASO_F64, clamp, int};

use crate::arma::Arma;
use crate::assets::Assets;
use crate::control::Control;
use crate::particulas::{self, Explosion};
use crate::tiempo::GameTime;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tipo {
    Coipo,
    Pollo,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Estado {
    Parado,
    Caminando,
    Saltando,
    Cayendo,
}

pub struct Personaje {
    pub tipo: Tipo,
    pub nombre: &'static str,
    pub posicion: Vec2,
    pub origen: Vec2,
    pub cuadro: Rectangle,
    pub rotacion: f32,
    pub zoom: f32,
    /// direccion == SpriteEffects.FlipHorizontally (mira a la izquierda).
    voltear: bool,
    pub color: Color,
    pub cuerpo: Rectangle,
    pub colision_superior: bool,
    pub colision_inferior: bool,
    pub colision_derecha: bool,
    pub colision_izquierda: bool,
    pub velocidad_movimiento: f32,
    velocidad_rotacion: f32,
    limite_salto: f32,
    velocidad_salto: f32,
    distancia_salto: f32,
    pub vida: f32,
    girando: bool,
    resbalando: bool,
    pub limites: Rectangle,
    pub monedas: i32,
    pub estado: Estado,
    max_frame_rate: f32,
    frame_rate: f32,
    frame_per_seconds: Vec2,
    pub polvo: Option<Explosion>,
    pub golpeado: bool,
    pub duracion_golpe: f64,
    pub posicion_golpe: i32,
    pub arma: Option<Arma>,
    pub armado: bool,
    pub cantidad_balas: i32,
}

impl Personaje {
    /// new Coipo() / new Pollo(), con los valores iniciales de los campos de Personaje.
    pub fn nuevo(tipo: Tipo) -> Self {
        Self {
            tipo,
            nombre: match tipo {
                Tipo::Coipo => "Coipo",
                Tipo::Pollo => "Pollo",
            },
            posicion: Vec2::ZERO,
            origen: Vec2::ZERO,
            cuadro: Rectangle::default(),
            rotacion: 0.0,
            zoom: 0.0,
            voltear: false,
            color: WHITE,
            cuerpo: Rectangle::default(),
            colision_superior: false,
            colision_inferior: false,
            colision_derecha: false,
            colision_izquierda: false,
            velocidad_movimiento: 0.0,
            velocidad_rotacion: 0.0,
            limite_salto: 0.0,
            velocidad_salto: 0.0,
            distancia_salto: 0.0,
            vida: 0.0,
            girando: false,
            resbalando: false,
            limites: Rectangle::default(),
            monedas: 0,
            estado: Estado::Parado,
            max_frame_rate: 0.0,
            frame_rate: 0.0,
            frame_per_seconds: Vec2::ZERO,
            polvo: None,
            golpeado: false,
            duracion_golpe: 0.5,
            posicion_golpe: 0,
            arma: None,
            armado: false,
            cantidad_balas: 20,
        }
    }

    pub fn textura<'a>(&self, assets: &'a Assets) -> &'a Texture2D {
        match self.tipo {
            Tipo::Coipo => &assets.coipo,
            Tipo::Pollo => &assets.pollo,
        }
    }

    /// Coipo.cargar / Pollo.cargar. No reinicia el arma, el color ni el estado del golpe: al
    /// reiniciar el nivel el coipo (que es el mismo objeto) conserva su pistola.
    pub fn cargar(&mut self, assets: &Assets) {
        self.cuadro = Rectangle::new(0, 0, 70, 70);
        self.rotacion = 0.0;
        self.voltear = true;
        // El origen se calcula con el Cuerpo anterior (vacío la primera vez).
        self.origen = vec2(
            (self.cuerpo.width / 2) as f32,
            (self.cuerpo.height / 2) as f32,
        );
        self.zoom = 1.0;
        self.estado = Estado::Parado;
        self.velocidad_movimiento = 0.0;
        self.frame_per_seconds = vec2(200.0, 0.0);
        self.max_frame_rate = self.textura(assets).width();
        self.polvo = Some(Explosion::cargar(10, &assets.hoja));
        self.vida = 200.0;
        self.cuerpo = Rectangle::new(
            int(self.posicion.x),
            int(self.posicion.y),
            int(self.cuadro.width as f32 * self.zoom),
            int(self.cuadro.height as f32 * self.zoom),
        );
    }

    pub fn actualizar(&mut self, tiempo: &GameTime, control: &Control, assets: &Assets) {
        self.limitar_movimiento();
        self.correr(control);
        self.animar(tiempo, control, assets);
        self.cambiar_estados(control);
        self.saltar(control);
        self.disparar(control, assets);
    }

    pub fn dibujar(&self, assets: &Assets) {
        self.dibujar_personaje(assets);
        if let Some(polvo) = &self.polvo {
            polvo.dibujar(&assets.hoja);
        }
    }

    fn limitar_movimiento(&mut self) {
        let l = self.limites;
        self.posicion.x = clamp(self.posicion.x, l.left() as f32, l.right() as f32);
        self.posicion.y = clamp(self.posicion.y, l.top() as f32, l.bottom() as f32);
        if self.posicion.x <= l.left() as f32 {
            self.velocidad_movimiento = 0.0;
            self.posicion.x = (l.left() + 1) as f32;
        }
        if self.posicion.x >= l.right() as f32 {
            self.velocidad_movimiento = 0.0;
            self.posicion.x = (l.right() - 1) as f32;
        }
    }

    fn correr(&mut self, control: &Control) {
        if self.velocidad_movimiento <= 0.0 && control.left_hold {
            self.voltear = true;
        } else if self.velocidad_movimiento >= 0.0 && control.right_hold {
            self.voltear = false;
        }
        if control.right_hold
            && !self.colision_derecha
            && self.velocidad_movimiento < 6.0
            && !self.resbalando
        {
            self.velocidad_movimiento += 0.1;
        }
        if control.left_hold
            && !self.colision_izquierda
            && self.velocidad_movimiento > -6.0
            && !self.resbalando
        {
            self.velocidad_movimiento -= 0.1;
        }
        if self.resbalando && !self.colision_derecha && !self.colision_izquierda {
            if self.velocidad_movimiento > 0.1 {
                self.velocidad_movimiento -= 0.2;
            } else if self.velocidad_movimiento < -0.1 {
                self.velocidad_movimiento += 0.2;
            } else {
                self.velocidad_movimiento = 0.0;
            }
        }
        // Hacia la derecha avanza aunque esté chocando; hacia la izquierda, no.
        if self.velocidad_movimiento > 1.0
            || (self.velocidad_movimiento < -1.0
                && !self.colision_izquierda
                && !self.colision_derecha)
        {
            self.posicion.x += self.velocidad_movimiento;
        }
    }

    fn animar(&mut self, tiempo: &GameTime, control: &Control, assets: &Assets) {
        let corriendo = control.left_hold || control.right_hold;
        if let Some(polvo) = &mut self.polvo {
            polvo.estado = if self.colision_superior && corriendo {
                particulas::Estado::Activo
            } else {
                particulas::Estado::Inactivo
            };
            polvo.actualizar(
                vec2(self.posicion.x, self.cuerpo.bottom() as f32),
                tiempo,
                &assets.hoja,
            );
        }
        if self.resbalando {
            self.cuadro.x = 0;
            self.cuadro.y = 70;
        }
        if self.estado == Estado::Saltando
            || (self.estado == Estado::Cayendo && !self.colision_superior)
        {
            if self.girando {
                self.rotacion += self.velocidad_rotacion;
                self.cuadro.x = 140;
                self.cuadro.y = 70;
                if control.right_hold {
                    self.velocidad_rotacion = 0.5;
                } else if control.left_hold {
                    self.velocidad_rotacion = -0.5;
                }
            } else {
                // Cuadro.X = 70; Cuadro.X = 70; (la Y queda como estaba)
                self.cuadro.x = 70;
            }
        }
        if self.colision_superior {
            self.rotacion = 0.0;
            self.girando = false;
        }
        if self.colision_superior && !corriendo {
            self.cuadro.x = 210;
            self.cuadro.y = 70;
        }
        if !self.colision_superior || !corriendo || self.resbalando {
            return;
        }
        self.cuadro.y = 0;
        if self.frame_rate > 70.0 {
            self.frame_rate = 0.0;
            if self.cuadro.x as f32 >= self.max_frame_rate - 70.0 {
                self.cuadro.x = -70;
            }
            self.cuadro.x += 70;
        }
        self.frame_rate += self.frame_per_seconds.x * (SEGUNDOS_POR_PASO_F64 as f32 * 10.0);
    }

    fn dibujar_personaje(&self, assets: &Assets) {
        xna::dibujar(
            self.textura(assets),
            self.posicion + self.origen,
            Some(self.cuadro),
            self.color,
            self.rotacion,
            self.origen,
            self.zoom,
            Efectos::horizontal_si(self.voltear),
        );
        if self.armado
            && let Some(arma) = &self.arma
        {
            arma.dibujar(assets);
        }
    }

    fn cambiar_estados(&mut self, control: &Control) {
        self.resbalando = (self.velocidad_movimiento > 0.0 && !control.right_hold)
            || (self.velocidad_movimiento < 0.0 && !control.left_hold);
        if (control.left_hold || control.right_hold) && self.estado != Estado::Cayendo {
            self.girando = true;
        }
        if self.estado != Estado::Saltando {
            if control.left_hold || control.right_hold {
                self.estado = Estado::Caminando;
            }
            if !control.right_hold && !control.left_hold && !self.resbalando {
                self.estado = Estado::Parado;
            }
        }
        if !self.colision_superior && self.estado != Estado::Saltando {
            self.estado = Estado::Cayendo;
        }
    }

    /// Mientras se mantiene B sube hasta 250 px; al soltarlo pasa a "Cayendo" en cada paso.
    fn saltar(&mut self, control: &Control) {
        if control.b_ {
            self.limite_salto = self.posicion.y - 250.0;
            self.distancia_salto = self.posicion.y;
            self.velocidad_salto = 8.0;
        }
        if control.b_hold && self.estado != Estado::Cayendo && !self.colision_inferior {
            if self.distancia_salto > self.limite_salto {
                self.distancia_salto -= 8.0;
                self.estado = Estado::Saltando;
            }
            if self.estado == Estado::Saltando && self.posicion.y > self.distancia_salto {
                self.posicion.y -= self.velocidad_salto;
                if self.velocidad_salto >= 0.3 {
                    self.velocidad_salto -= 0.3;
                }
                if self.velocidad_salto <= 0.5 {
                    self.estado = Estado::Parado;
                }
            }
        } else {
            self.distancia_salto = 0.0;
            self.velocidad_salto = 0.0;
            self.estado = Estado::Cayendo;
        }
    }

    /// Personaje.disparar con la pistola ("Laser"): las balas inactivas esperan junto al arma,
    /// que se coloca según hacia dónde mira el personaje; Z dispara la siguiente (de la 10 a la 1).
    fn disparar(&mut self, control: &Control, assets: &Assets) {
        if !self.armado {
            return;
        }
        let Some(arma) = &mut self.arma else {
            return;
        };
        arma.voltear = self.voltear;
        let ancho_bala = assets.laser.width();
        for i in 0..arma.balas.len() {
            if arma.balas[i].activa {
                arma.disparar(i, &assets.laser);
                continue;
            }
            // La bala toma la posición que tenía el arma y después el arma sigue al personaje.
            let junto_al_arma = arma.posicion;
            let bala = &mut arma.balas[i];
            if !self.voltear {
                bala.direccion = vec2(20.0, 0.0);
                bala.posicion = vec2(junto_al_arma.x + 80.0, junto_al_arma.y + 30.0);
                arma.posicion = vec2(self.posicion.x, self.posicion.y - 10.0);
            } else {
                bala.direccion = vec2(-20.0, 0.0);
                bala.posicion = vec2(junto_al_arma.x - ancho_bala + 10.0, junto_al_arma.y + 30.0);
                arma.posicion = vec2(self.posicion.x - 50.0, self.posicion.y - 10.0);
            }
            arma.balas[i].rotacion = 0.0;
        }
        if control.a_ && self.cantidad_balas > 0 && !arma.balas.is_empty() {
            arma.balas[(self.cantidad_balas - 1) as usize].activa = true;
            self.cantidad_balas -= 1;
        }
    }
}
