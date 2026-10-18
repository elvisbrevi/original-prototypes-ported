// Port de Iman.Objetos: Objeto (clase base), Plataforma y Niño.

use macroquad::audio::play_sound_once;
use macroquad::prelude::*;

use crate::assets::Assets;
use crate::control::Control;
use crate::xna::{Rectangle, SEGUNDOS_POR_PASO, int};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Diseno {
    #[default]
    Null,
    D01,
    D02,
    D03,
    D04,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Orientacion {
    #[default]
    Arriba,
    Abajo,
    Derecha,
    Izquierda,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Estado {
    #[default]
    Parado,
    Corriendo,
    Saltando,
    Cayendo,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Colision {
    #[default]
    Superior,
    Lateral,
    Inferior,
    Total,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Movimiento {
    #[default]
    Ninguno,
    Vertical,
    Horizontal,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TipoObjeto {
    #[default]
    Plataforma,
    Personaje,
}

pub enum Clase {
    Plataforma,
    Nino(Box<Nino>),
}

/// Campos públicos de Iman.Objetos.Objeto que lee y modifica la física.
pub struct Objeto {
    pub clase: Clase,
    pub diseno: Diseno,
    pub orientacion: Orientacion,
    pub tipo_objeto: TipoObjeto,
    pub colision: Colision,
    pub movimiento: Movimiento,
    pub posicion_inicial: i32,
    pub posicion_final: i32,
    pub estado: Estado,
    pub textura: Texture2D,
    pub posicion: Vec2,
    pub origen: Vec2,
    pub cuerpo: Rectangle,
    pub colision_superior: bool,
    pub colision_inferior: bool,
    pub colision_izquierda: bool,
    pub colision_derecha: bool,
    pub velocidad: i32,
    pub ida: bool,
    pub vuelta: bool,
    pub gravedad: bool,
    pub fuerza_gravedad: f32,
    pub rango_iman: f32,
    pub usando_iman: bool,
    pub iman: Rectangle,
}

/// Estado propio de Iman.Objetos.Niño.
pub struct Nino {
    cuadro: Rectangle,
    zoom: f32,
    voltear: bool,
    distancia_salto: f32,
    limite_salto: f32,
    velocidad_salto: f32,
    altura_salto: f32,
    max_frame_rate: f32,
    frame_rate: f32,
    frame_per_seconds: f32,
    textura_iman: Texture2D,
    cuadro_iman: Rectangle,
    frame_iman: f32,
    voltear_iman: bool,
    rotacion_iman: f32,
    posicion_iman: Vec2,
}

impl Objeto {
    fn vacio(clase: Clase, textura: Texture2D) -> Self {
        Self {
            clase,
            diseno: Diseno::Null,
            orientacion: Orientacion::Arriba,
            tipo_objeto: TipoObjeto::Plataforma,
            colision: Colision::Superior,
            movimiento: Movimiento::Ninguno,
            posicion_inicial: 0,
            posicion_final: 0,
            estado: Estado::Parado,
            textura,
            posicion: Vec2::ZERO,
            origen: Vec2::ZERO,
            cuerpo: Rectangle::default(),
            colision_superior: false,
            colision_inferior: false,
            colision_izquierda: false,
            colision_derecha: false,
            velocidad: 0,
            ida: false,
            vuelta: false,
            gravedad: false,
            fuerza_gravedad: 0.0,
            rango_iman: 0.0,
            usando_iman: false,
            iman: Rectangle::default(),
        }
    }

    /// new Plataforma() con los campos que fija cada Escenario.crear*; `configurar` va antes de cargar().
    pub fn plataforma(
        assets: &Assets,
        diseno: Diseno,
        configurar: impl FnOnce(&mut Objeto),
    ) -> Self {
        let mut p = Self::vacio(
            Clase::Plataforma,
            assets.plataformas[diseno as usize].clone(),
        );
        p.diseno = diseno;
        configurar(&mut p);
        // Plataforma.cargar
        p.tipo_objeto = TipoObjeto::Plataforma;
        p.ida = true;
        if p.diseno != Diseno::Null {
            p.cuerpo = p.cuerpo_textura();
        }
        p
    }

    /// new Niño() + Niño.cargar.
    pub fn nino(assets: &Assets, posicion: Vec2) -> Self {
        let textura = assets.nino.clone();
        let cuadro = Rectangle::new(
            0,
            0,
            textura.width() as i32 / 8,
            textura.height() as i32 / 2,
        );
        let zoom = 1.0;
        let max_frame_rate = (textura.width() as i32 - cuadro.width) as f32;
        let mut n = Self::vacio(
            Clase::Nino(Box::new(Nino {
                cuadro,
                zoom,
                voltear: false,
                distancia_salto: 0.0,
                limite_salto: 0.0,
                velocidad_salto: 0.0,
                altura_salto: 7.0,
                max_frame_rate,
                frame_rate: 0.0,
                frame_per_seconds: 60.0,
                textura_iman: assets.iman.clone(),
                cuadro_iman: Rectangle::new(0, 0, 5, 10),
                frame_iman: 60.0,
                voltear_iman: false,
                rotacion_iman: 0.0,
                posicion_iman: Vec2::ZERO,
            })),
            textura,
        );
        n.posicion = posicion;
        n.tipo_objeto = TipoObjeto::Personaje;
        n.diseno = Diseno::D01;
        n.gravedad = true;
        n.cuerpo = Rectangle::new(
            int(posicion.x),
            int(posicion.y),
            int(cuadro.width as f32 * zoom),
            int(cuadro.height as f32 * zoom),
        );
        n.origen = vec2((n.cuerpo.width / 2) as f32, (n.cuerpo.height / 2) as f32);
        n.velocidad = 3;
        n.rango_iman = 50.0;
        n
    }

    fn cuerpo_textura(&self) -> Rectangle {
        Rectangle::new(
            int(self.posicion.x),
            int(self.posicion.y),
            self.textura.width() as i32,
            self.textura.height() as i32,
        )
    }

    pub fn actualizar(&mut self, control: &Control, assets: &Assets) {
        match self.clase {
            Clase::Plataforma => {
                self.mover_plataforma();
                if self.diseno != Diseno::Null {
                    self.cuerpo = self.cuerpo_textura();
                }
            }
            Clase::Nino(_) => {
                self.mover(control);
                self.saltar(control, assets);
                self.cambiar_estados(control);
                self.animar();
                self.usar_magnetismo(control);
            }
        }
    }

    pub fn dibujar(&self) {
        match &self.clase {
            Clase::Plataforma => {
                // Las plataformas NULL nunca se dibujan (Escenario.dibujarObjetos las salta).
                draw_texture(&self.textura, self.posicion.x, self.posicion.y, WHITE);
            }
            Clase::Nino(n) => {
                if self.usando_iman {
                    n.dibujar_iman(self.rango_iman);
                }
                // Draw(textura, posicion + origen, cuadro, ..., origen, zoom, ...): la esquina queda en posicion.
                draw_texture_ex(
                    &self.textura,
                    self.posicion.x,
                    self.posicion.y,
                    WHITE,
                    DrawTextureParams {
                        source: Some(n.cuadro.as_rect()),
                        dest_size: Some(
                            vec2(n.cuadro.width as f32, n.cuadro.height as f32) * n.zoom,
                        ),
                        flip_x: n.voltear,
                        ..Default::default()
                    },
                );
            }
        }
    }

    // Plataforma.mover
    fn mover_plataforma(&mut self) {
        let eje = match self.movimiento {
            Movimiento::Ninguno => return,
            Movimiento::Horizontal => &mut self.posicion.x,
            Movimiento::Vertical => &mut self.posicion.y,
        };
        if self.ida {
            *eje += self.velocidad as f32;
            if *eje >= self.posicion_final as f32 {
                self.ida = false;
                self.vuelta = true;
            }
        } else if self.vuelta {
            *eje -= self.velocidad as f32;
            if *eje <= self.posicion_inicial as f32 {
                self.ida = true;
                self.vuelta = false;
            }
        }
    }

    fn datos_nino(&mut self) -> &mut Nino {
        match &mut self.clase {
            Clase::Nino(n) => n,
            Clase::Plataforma => unreachable!("el objeto no es el niño"),
        }
    }

    // Niño.mover
    fn mover(&mut self, control: &Control) {
        if control.left_hold {
            self.posicion.x -= self.velocidad as f32;
            self.datos_nino().voltear = false;
            self.orientacion = Orientacion::Izquierda;
        }
        if control.right_hold {
            self.posicion.x += self.velocidad as f32;
            self.datos_nino().voltear = true;
            self.orientacion = Orientacion::Derecha;
        }
        if control.up_hold {
            self.orientacion = Orientacion::Arriba;
        }
        if control.down_hold {
            self.orientacion = Orientacion::Abajo;
        }
        let cuadro = self.datos_nino().cuadro;
        self.cuerpo = Rectangle::new(
            int(self.posicion.x),
            int(self.posicion.y),
            cuadro.width,
            cuadro.height,
        );
        self.origen = vec2(
            self.posicion.x + (self.cuerpo.width / 2) as f32,
            self.posicion.y + (self.cuerpo.height / 2) as f32,
        );
    }

    // Niño.saltar
    fn saltar(&mut self, control: &Control, assets: &Assets) {
        let (posicion, colision_superior, colision_inferior) = (
            self.posicion,
            self.colision_superior,
            self.colision_inferior,
        );
        let mut estado = self.estado;
        let n = self.datos_nino();
        let mut y = posicion.y;
        if control.b_ {
            if colision_superior {
                play_sound_once(&assets.salto);
            }
            n.limite_salto = y - 600.0;
            n.distancia_salto = y;
            n.velocidad_salto = n.altura_salto;
        }
        if control.b_hold && estado != Estado::Cayendo && !colision_inferior {
            if n.distancia_salto > n.limite_salto {
                n.distancia_salto -= 8.0;
                estado = Estado::Saltando;
            }
            if estado == Estado::Saltando && y > n.distancia_salto {
                y -= n.velocidad_salto;
                if n.velocidad_salto >= 0.2 {
                    n.velocidad_salto -= 0.2;
                }
                if n.velocidad_salto < 0.3 {
                    estado = Estado::Parado;
                }
            }
        } else {
            n.distancia_salto = 0.0;
            n.velocidad_salto = 0.0;
            estado = Estado::Cayendo;
        }
        self.posicion.y = y;
        self.estado = estado;
    }

    // Niño.cambiarEstados
    fn cambiar_estados(&mut self, control: &Control) {
        if self.estado != Estado::Saltando {
            if control.left_hold || control.right_hold {
                self.estado = Estado::Corriendo;
            }
            if !control.right_hold && !control.left_hold {
                self.estado = Estado::Parado;
            }
        }
        if !self.colision_superior && self.estado != Estado::Saltando {
            self.estado = Estado::Cayendo;
        }
    }

    // Niño.animar
    fn animar(&mut self) {
        let estado = self.estado;
        let n = self.datos_nino();
        let c = &mut n.cuadro;
        match estado {
            Estado::Saltando => (c.x, c.y) = (c.width * 5, c.height),
            Estado::Cayendo => (c.x, c.y) = (c.width * 6, c.height),
            Estado::Parado => (c.x, c.y) = (c.width * 3, c.height),
            Estado::Corriendo => {
                if n.frame_rate > c.width as f32 {
                    n.frame_rate = 0.0;
                    if c.x as f32 >= n.max_frame_rate {
                        c.x = -c.width;
                    }
                    c.y = 0;
                    c.x += c.width;
                }
                n.frame_rate += n.frame_per_seconds * (SEGUNDOS_POR_PASO * 10.0);
            }
        }
        if n.frame_iman > 5.0 {
            if n.cuadro_iman.x == 45 {
                n.cuadro_iman.x = 0;
            } else {
                n.cuadro_iman.x += 5;
            }
            n.frame_iman = 0.0;
        }
        n.frame_iman += SEGUNDOS_POR_PASO * 100.0;
    }

    // Niño.usarMagnetismo: el imán sale de la mochila, por eso apunta al lado contrario de la mirada.
    fn usar_magnetismo(&mut self, control: &Control) {
        self.usando_iman = control.a_hold;
        let (cuerpo, posicion, rango, orientacion) = (
            self.cuerpo,
            self.posicion,
            self.rango_iman,
            self.orientacion,
        );
        let mut iman = self.iman;
        let n = self.datos_nino();
        match orientacion {
            Orientacion::Izquierda => {
                iman = Rectangle::new(
                    cuerpo.right() + 10,
                    int(posicion.y),
                    int(rango),
                    cuerpo.height,
                );
                n.posicion_iman = vec2(cuerpo.right() as f32, posicion.y);
                n.voltear_iman = false;
                n.rotacion_iman = 0.0;
            }
            Orientacion::Derecha => {
                iman = Rectangle::new(
                    int(posicion.x - rango) - 10,
                    int(posicion.y),
                    int(rango),
                    cuerpo.height,
                );
                n.posicion_iman = vec2(cuerpo.left() as f32 - rango, posicion.y);
                n.voltear_iman = true;
                n.rotacion_iman = 0.0;
            }
            Orientacion::Arriba => {
                iman = Rectangle::new(
                    int(posicion.x),
                    int(posicion.y - rango) - 10,
                    cuerpo.width,
                    int(rango),
                );
                n.posicion_iman = posicion;
                n.rotacion_iman = -1.57;
            }
            Orientacion::Abajo => {}
        }
        self.iman = iman;
    }
}

impl Nino {
    // Niño.dibujarIman: Draw(texturaIman, Rectangle(x, y, rangoIman, 100), cuadroIman, ..., rotacion, Vector2.Zero, ...)
    fn dibujar_iman(&self, rango_iman: f32) {
        let x = int(self.posicion_iman.x) as f32;
        let y = int(self.posicion_iman.y) as f32;
        draw_texture_ex(
            &self.textura_iman,
            x,
            y,
            WHITE,
            DrawTextureParams {
                source: Some(self.cuadro_iman.as_rect()),
                dest_size: Some(vec2(int(rango_iman) as f32, 100.0)),
                rotation: self.rotacion_iman,
                flip_x: self.voltear_iman,
                pivot: Some(vec2(x, y)),
                ..Default::default()
            },
        );
    }
}
