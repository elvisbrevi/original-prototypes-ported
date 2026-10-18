// IMAN (Prototipo) — port a macroquad del juego XNA 3.1 de Elvis Brevi (2009).

mod assets;
mod control;
mod escenarios;
mod fisica;
mod graficos;
mod menu;
mod objetos;
mod shader;

use macroquad::prelude::*;
use museo::{Boton, ConfigTactil, Cruceta, Pantalla, PasoFijo, Tactil, WEB};

use crate::assets::Assets;
use crate::control::Control;
use crate::escenarios::Camara;
use crate::menu::Menu;

/// Resolución del juego original (640x480).
pub const ANCHO: f32 = 640.0;
pub const ALTO: f32 = 480.0;

fn window_conf() -> Conf {
    Conf {
        window_title: "IMAN (Prototipo)".to_owned(),
        window_width: ANCHO as i32,
        window_height: ALTO as i32,
        ..Default::default()
    }
}

/// Cruceta, botones A (imán) y B (salto) y START, como el mando del original.
const TACTIL: ConfigTactil = ConfigTactil {
    cruceta: Cruceta::Completa,
    botones: &[
        Boton {
            letra: "A",
            etiqueta: "IMAN",
        },
        Boton {
            letra: "B",
            etiqueta: "SALTO",
        },
    ],
    start: Some("START"),
    zonas: &[],
};

#[macroquad::main(window_conf)]
async fn main() {
    rand::srand(miniquad::date::now() as u64);
    let assets = Assets::cargar().await;
    let pantalla = Pantalla::nueva(ANCHO, ALTO);
    let mut paso = PasoFijo::xna();
    let mut camara = Camara::default();
    let mut control = Control::default();
    let mut tactil = Tactil::nuevo(TACTIL);
    let mut menu = Menu::nuevo(&assets, &mut camara);
    let mut pantalla_completa = false;

    loop {
        // En la web no hay ventana que cerrar: Esc y "Quit" no hacen nada.
        if is_key_down(KeyCode::Escape) && !WEB {
            break;
        }
        if is_key_pressed(KeyCode::F4) {
            pantalla_completa = !pantalla_completa;
            set_fullscreen(pantalla_completa);
        }

        let pulsados = tactil.actualizar(&pantalla);
        for _ in 0..paso.pasos() {
            control.actualizar(&pulsados);
            menu.actualizar(&control, &mut camara, &assets);
            if menu.exit {
                if !WEB {
                    return;
                }
                menu.exit = false;
            }
        }

        pantalla.vista(Vec2::ZERO);
        clear_background(BLACK);
        menu.dibujar(&camara, &assets, |origen| {
            pantalla.vista(origen.unwrap_or_default())
        });
        museo::presentar(&pantalla, &tactil, &pulsados);
        next_frame().await
    }
}
