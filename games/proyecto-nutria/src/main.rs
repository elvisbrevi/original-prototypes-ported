// Proyecto Nutria (Version Prototipo) — port a macroquad del juego XNA 3.1 de Elvis Brevi (2009).

mod arma;
mod assets;
mod caja;
mod camara;
mod control;
mod fisica;
mod iconos;
mod menu;
mod nivel;
mod particulas;
mod personaje;
mod plataforma;
mod shader;
mod tiempo;

use macroquad::prelude::*;
use museo::{Boton, ConfigTactil, Cruceta, Pantalla, PasoFijo, Tactil, WEB};
use xna::colores::CORNFLOWER_BLUE;

use crate::assets::Assets;
use crate::camara::Camara;
use crate::control::Control;
use crate::iconos::Posicion;
use crate::nivel::Nivel;
use crate::tiempo::GameTime;

/// Back buffer del original: 1280x720.
const ANCHO: f32 = 1280.0;
const ALTO: f32 = 720.0;
/// Game1.areaJuego: el viewport donde se dibuja el nivel, dentro del marco.
const AREA: Rect = Rect::new(100.0, 100.0, 1080.0, 520.0);

fn window_conf() -> Conf {
    Conf {
        window_title: "Proyecto Nutria (Version Prototipo)".to_owned(),
        window_width: ANCHO as i32,
        window_height: ALTO as i32,
        ..Default::default()
    }
}

/// Las teclas del original: cruceta (flechas), A dispara (Z), B salta (X) y START pausa (Enter).
const TACTIL: ConfigTactil = ConfigTactil {
    cruceta: Cruceta::Completa,
    botones: &[
        Boton {
            letra: "A",
            etiqueta: "DISPARO",
        },
        Boton {
            letra: "B",
            etiqueta: "SALTO",
        },
    ],
    start: Some("START"),
    zonas: &[],
};

/// Cámara del área de juego: con Some((posición, escala)) aplica la transformación de Camera2D
/// (escala y luego traslación de -posición * escala); con None dibuja sin transformar.
fn vista_area(area: &RenderTarget, transform: Option<(Vec2, f32)>) {
    let (origen, escala) = transform.unwrap_or((Vec2::ZERO, 1.0));
    set_camera(&Camera2D {
        target: origen + AREA.size() / (2.0 * escala),
        zoom: vec2(2.0 * escala / AREA.w, 2.0 * escala / AREA.h),
        render_target: Some(area.clone()),
        ..Default::default()
    });
}

#[macroquad::main(window_conf)]
async fn main() {
    let assets = Assets::cargar();
    let pantalla = Pantalla::nueva(ANCHO, ALTO);
    let area = render_target(AREA.w as u32, AREA.h as u32);
    area.texture.set_filter(FilterMode::Linear);
    let mut paso = PasoFijo::xna();
    let mut tactil = Tactil::nuevo(TACTIL);
    // Sin mando DirectInput el original crea un único control: el teclado.
    let mut controles = [Control::default()];
    let mut tiempo = GameTime::default();
    let mut camara = Camara::nueva();
    let mut nivel = Nivel::cargar_nivel_1(&assets);

    loop {
        if is_key_down(KeyCode::Escape) && !WEB {
            break;
        }
        let pulsados = tactil.actualizar(&pantalla);
        for _ in 0..paso.pasos() {
            // Game1.Update (Game1.pause nunca se activa).
            tiempo.avanzar();
            for control in &mut controles {
                control.actualizar(&pulsados);
            }
            camara.actualizar(&nivel.personajes, nivel.limites);
            nivel.actualizar(&tiempo, &controles, &mut camara, &assets);
            // base.Update: Camera2D es un GameComponent y calcula su matriz al final.
            camara.actualizar_transform();
        }

        // Game1.Draw: marco e iconos en la pantalla completa...
        pantalla.vista(Vec2::ZERO);
        clear_background(CORNFLOWER_BLUE);
        draw_texture(&assets.marco_coipo, 0.0, 0.0, WHITE);
        for (i, (icono, personaje)) in nivel.icon_per.iter().zip(&nivel.personajes).enumerate() {
            let posicion = if i == 0 {
                Posicion::SuperiorIzquierda
            } else {
                Posicion::SuperiorDerecha
            };
            icono.dibujar(personaje, posicion, &assets);
        }
        // ...y el nivel en el viewport areaJuego, que se borra en negro.
        vista_area(&area, None);
        clear_background(BLACK);
        nivel.dibujar(&camara, &assets, |t| vista_area(&area, t));
        pantalla.vista(Vec2::ZERO);
        pantalla.copiar_opaco(&area.texture, AREA);

        museo::presentar(&pantalla, &tactil, &pulsados);
        next_frame().await
    }
}
