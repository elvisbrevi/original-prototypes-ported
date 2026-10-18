// Port de Proyecto_Nutria.MenuPersonajes. A pesar del nombre es el menú de pausa (Enter):
// CONTINUAR, REINICIAR y SALIR. Además coloca a los personajes al cargar y al reiniciar. Las caras
// para elegir personaje se cargan pero nunca se dibujan, así que no se portaron.

use macroquad::prelude::*;
use xna::colores::ROJO;

use crate::assets::Assets;
use crate::camara::Camara;
use crate::control::Control;
use crate::iconos::IconoPersonajes;
use crate::personaje::{Personaje, Tipo};

const OPCIONES: [&str; 3] = ["CONTINUAR", "REINICIAR", "SALIR"];

#[derive(Default)]
pub struct MenuPersonajes {
    pub activo: bool,
    opcion_seleccionada: usize,
}

impl MenuPersonajes {
    /// MenuPersonajes.cargar: el coipo es el de la lista del menú, que ya se cargó una vez en
    /// (0, 0); el pollo es nuevo.
    pub fn cargar(
        limites: xna::Rectangle,
        assets: &Assets,
    ) -> (Self, Vec<Personaje>, Vec<IconoPersonajes>) {
        let mut coipo = Personaje::nuevo(Tipo::Coipo);
        coipo.cargar(assets);
        let mut personajes = vec![coipo];
        let iconos = elegir_personaje(&mut personajes, limites, assets);
        (Self::default(), personajes, iconos)
    }

    pub fn actualizar(
        &mut self,
        control: &Control,
        personajes: &mut Vec<Personaje>,
        iconos: &mut Vec<IconoPersonajes>,
        limites: xna::Rectangle,
        camara: &mut Camara,
        assets: &Assets,
    ) {
        if self.activo {
            self.desplazarse_en_menu(control);
            self.seleccionar_opciones(control, personajes, iconos, limites, camara, assets);
        }
        if control.start_ {
            self.activo = true;
        }
    }

    /// Se dibuja dentro del área de juego, sin la transformación de la cámara.
    pub fn dibujar(&self, assets: &Assets) {
        draw_texture(&assets.fondo_menu, 300.0, 100.0, WHITE);
        for (i, opcion) in OPCIONES.iter().enumerate() {
            let color = if i == self.opcion_seleccionada {
                ROJO
            } else {
                BLACK
            };
            assets
                .fuente
                .dibujar(opcion, vec2(450.0, (170 + i * 50) as f32), color);
        }
    }

    fn desplazarse_en_menu(&mut self, control: &Control) {
        if control.up_ {
            self.opcion_seleccionada = if self.opcion_seleccionada == 0 {
                2
            } else {
                self.opcion_seleccionada - 1
            };
        }
        if control.down_ {
            self.opcion_seleccionada = if self.opcion_seleccionada == 2 {
                0
            } else {
                self.opcion_seleccionada + 1
            };
        }
    }

    /// SALIR no sale del juego: reinicia igual que REINICIAR, pero sin mover la cámara.
    fn seleccionar_opciones(
        &mut self,
        control: &Control,
        personajes: &mut Vec<Personaje>,
        iconos: &mut Vec<IconoPersonajes>,
        limites: xna::Rectangle,
        camara: &mut Camara,
        assets: &Assets,
    ) {
        if !control.a_ {
            return;
        }
        if self.opcion_seleccionada != 0 {
            *iconos = elegir_personaje(personajes, limites, assets);
        }
        if self.opcion_seleccionada == 1 {
            camara.posicion.x = 0.0;
            camara.escala = 0.36;
        }
        self.activo = false;
        self.opcion_seleccionada = 0;
    }
}

/// MenuPersonajes.elegirPersonaje: el coipo (siempre el mismo objeto, con lo que conserva su
/// pistola) vuelve a (0, 280) y se crea un pollo nuevo en (2600, 800).
fn elegir_personaje(
    personajes: &mut Vec<Personaje>,
    limites: xna::Rectangle,
    assets: &Assets,
) -> Vec<IconoPersonajes> {
    personajes.truncate(1);
    let coipo = &mut personajes[0];
    coipo.posicion = vec2(0.0, 280.0);
    coipo.monedas = 0;
    coipo.limites = limites;
    coipo.cargar(assets);
    let mut pollo = Personaje::nuevo(Tipo::Pollo);
    pollo.posicion = vec2(2600.0, 800.0);
    pollo.limites = limites;
    pollo.cargar(assets);
    personajes.push(pollo);
    personajes
        .iter()
        .map(|_| IconoPersonajes::default())
        .collect()
}
