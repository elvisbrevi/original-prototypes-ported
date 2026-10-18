// Port de Iman.Menus.Menu: menú/pausa, transición entre escenarios y créditos.

use macroquad::prelude::*;

use crate::assets::Assets;
use crate::control::Control;
use crate::escenarios::{Camara, Escenario};
use crate::shader::NonPhotoRealistic;
use crate::xna::{ALTO, AMARILLO, ANCHO, VERDE};

pub struct Menu {
    shader: NonPhotoRealistic,
    opciones: [&'static str; 2],
    opcion_seleccionada: usize,
    opcion: &'static str,
    mapa: Escenario,
    pub activo: bool,
    pub exit: bool,
    creditos: bool,
    alfa_cortina: u8,
    alfa_creditos: u8,
}

impl Menu {
    pub fn nuevo(assets: &Assets, camara: &mut Camara) -> Self {
        Self {
            shader: NonPhotoRealistic::nuevo(&assets.sketch),
            opciones: ["Play", "Quit"],
            opcion_seleccionada: 0,
            opcion: "",
            mapa: Escenario::escenario_01(assets, camara),
            activo: true,
            exit: false,
            creditos: false,
            alfa_cortina: 255,
            alfa_creditos: 0,
        }
    }

    pub fn actualizar(&mut self, control: &Control, camara: &mut Camara, assets: &Assets) {
        if control.start_ && !self.activo {
            self.activo = true;
        }
        if self.activo {
            self.shader.actualizar();
            self.recorrer_menu(control);
            self.seleccionar_opciones(control, camara, assets);
            return;
        }
        if self.mapa.activo {
            if self.alfa_cortina > 1 {
                self.alfa_cortina -= 1;
            }
            self.mapa.actualizar(camara, control, assets);
            return;
        }
        if self.alfa_cortina < 254 {
            self.alfa_cortina += 1;
            return;
        }
        if self.mapa.id == 0 {
            self.mapa = Escenario::escenario_02(assets, camara);
            self.mapa.activo = true;
        }
        // Sin "else" en el original: al cargar el escenario 2 ya se marcan los créditos
        // (con alfa 1, invisibles) y se completan al terminarlo.
        if self.mapa.id == 1 {
            self.creditos = true;
            if self.alfa_creditos < 254 {
                self.alfa_creditos += 1;
            }
        }
    }

    /// `vista` fija la cámara: con Some(posición) en coordenadas de mundo, con None en pantalla.
    pub fn dibujar(&self, camara: &Camara, assets: &Assets, vista: impl Fn(Option<Vec2>)) {
        if self.activo {
            vista(None);
            self.shader
                .dibujar(|| draw_texture(&assets.titulo, 0.0, 0.0, WHITE));
            for (i, opcion) in self.opciones.iter().enumerate() {
                let (texto, color) = if self.opcion_seleccionada == i {
                    (format!("*{opcion}"), VERDE)
                } else {
                    (opcion.to_string(), AMARILLO)
                };
                assets
                    .fuente
                    .dibujar(&texto, vec2(250.0, (150 + i * 50) as f32), color);
            }
        } else {
            vista(Some(camara.position));
            self.mapa.dibujar(camara);
            let cam = camara.position;
            draw_texture_ex(
                &assets.cortina,
                cam.x as i32 as f32,
                cam.y as i32 as f32,
                Color::from_rgba(0, 0, 0, self.alfa_cortina),
                DrawTextureParams {
                    dest_size: Some(vec2(ANCHO, ALTO)),
                    ..Default::default()
                },
            );
            if self.creditos {
                assets.fuente.dibujar(
                    "FIN...",
                    cam + vec2(260.0, 200.0),
                    Color::from_rgba(255, 255, 255, self.alfa_creditos),
                );
            }
        }
    }

    fn seleccionar_opciones(&mut self, control: &Control, camara: &mut Camara, assets: &Assets) {
        if !control.a_ {
            return;
        }
        match self.opcion {
            "Play" => {
                self.mapa = Escenario::escenario_01(assets, camara);
                self.mapa.activo = true;
                self.opciones = ["Resume", "Menu"];
                self.alfa_cortina = 255;
                self.opcion_seleccionada = 0;
                self.activo = false;
            }
            "Quit" => self.exit = true,
            "Resume" => {
                self.mapa.activo = true;
                self.activo = false;
            }
            "Menu" => {
                self.mapa.activo = false;
                self.opciones = ["Play", "Quit"];
                self.opcion_seleccionada = 0;
                self.activo = true;
            }
            _ => {}
        }
    }

    fn recorrer_menu(&mut self, control: &Control) {
        let ultima = self.opciones.len() - 1;
        if control.down_ {
            self.opcion_seleccionada = if self.opcion_seleccionada == ultima {
                0
            } else {
                self.opcion_seleccionada + 1
            };
        }
        if control.up_ {
            self.opcion_seleccionada = if self.opcion_seleccionada == 0 {
                ultima
            } else {
                self.opcion_seleccionada - 1
            };
        }
        self.opcion = self.opciones[self.opcion_seleccionada];
    }
}
