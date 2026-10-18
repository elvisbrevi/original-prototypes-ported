// IMAN (Prototipo) — port a macroquad del juego XNA 3.1 de Elvis Brevi (2009).

mod assets;
mod control;
mod escenarios;
mod fisica;
mod graficos;
mod menu;
mod objetos;
mod shader;
mod xna;

use macroquad::prelude::*;

use crate::assets::Assets;
use crate::control::Control;
use crate::escenarios::Camara;
use crate::menu::Menu;
use crate::xna::{ALTO, ANCHO, SEGUNDOS_POR_PASO};

fn window_conf() -> Conf {
    Conf {
        window_title: "IMAN (Prototipo)".to_owned(),
        window_width: ANCHO as i32,
        window_height: ALTO as i32,
        ..Default::default()
    }
}

/// Cámara que dibuja en la pantalla virtual de 640x480 con `origen` en la esquina superior izquierda.
fn vista(pantalla: &RenderTarget, origen: Vec2) {
    set_camera(&Camera2D {
        target: origen + vec2(ANCHO, ALTO) / 2.0,
        zoom: vec2(2.0 / ANCHO, 2.0 / ALTO),
        render_target: Some(pantalla.clone()),
        ..Default::default()
    });
}

#[macroquad::main(window_conf)]
async fn main() {
    rand::srand(miniquad::date::now() as u64);
    let assets = Assets::cargar().await;
    let pantalla = render_target(ANCHO as u32, ALTO as u32);
    pantalla.texture.set_filter(FilterMode::Linear);

    let mut camara = Camara::default();
    let mut control = Control::default();
    let mut menu = Menu::nuevo(&assets, &mut camara);
    let mut acumulado = 0.0_f64;
    let mut pantalla_completa = false;

    loop {
        if is_key_down(KeyCode::Escape) {
            break;
        }
        if is_key_pressed(KeyCode::F4) {
            pantalla_completa = !pantalla_completa;
            set_fullscreen(pantalla_completa);
        }

        // Paso fijo como Game de XNA (IsFixedTimeStep, máximo 500 ms acumulados por frame).
        acumulado += (get_frame_time() as f64).min(0.5);
        while acumulado >= SEGUNDOS_POR_PASO as f64 {
            acumulado -= SEGUNDOS_POR_PASO as f64;
            control.actualizar();
            menu.actualizar(&control, &mut camara, &assets);
            if menu.exit {
                return;
            }
        }

        vista(&pantalla, Vec2::ZERO);
        clear_background(BLACK);
        menu.dibujar(&camara, &assets, |origen| {
            vista(&pantalla, origen.unwrap_or_default())
        });

        // Escala la pantalla virtual a la ventana manteniendo 4:3.
        set_default_camera();
        clear_background(BLACK);
        let escala = (screen_width() / ANCHO).min(screen_height() / ALTO);
        let tamano = vec2(ANCHO, ALTO) * escala;
        draw_texture_ex(
            &pantalla.texture,
            (screen_width() - tamano.x) / 2.0,
            (screen_height() - tamano.y) / 2.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(tamano),
                ..Default::default()
            },
        );
        next_frame().await
    }
}
