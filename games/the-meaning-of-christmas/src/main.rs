// The Meaning of Christmas — port a macroquad del juego de Android (Java + libGDX 0.9.2) de Elvis
// Brevi (2011).

mod gdx;
mod juego;

use macroquad::audio::load_sound_from_bytes;
use macroquad::prelude::*;
use museo::{ConfigTactil, Cruceta, Pantalla, Tactil};

use crate::gdx::{ALTO, ANCHO, BitmapFont, Entrada, Musica, textura};
use crate::juego::{Juego, Recursos};

fn window_conf() -> Conf {
    Conf {
        window_title: "The Meaning of Christmas".to_owned(),
        // La pantalla de 320x480 del juego, a una vez y media.
        window_width: (ANCHO * 1.5) as i32,
        window_height: (ALTO * 1.5) as i32,
        ..Default::default()
    }
}

/// Sin controles en pantalla: se juega tocando, como en Android.
const TACTIL: ConfigTactil = ConfigTactil {
    cruceta: Cruceta::Ninguna,
    botones: &[],
    start: None,
    zonas: &[],
};

macro_rules! imagen {
    ($nombre:literal) => {
        textura(include_bytes!(concat!(
            "../assets/imagenes/",
            $nombre,
            ".png"
        )))
    };
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut recursos = Recursos {
        titulo: imagen!("titulo"),
        imagen_titulo: imagen!("imagen titulo"),
        pixel: imagen!("pixel"),
        dinero: imagen!("dinero"),
        franja: imagen!("franja"),
        noche: imagen!("noche"),
        luna: imagen!("luna"),
        regalos: [imagen!("regalo1"), imagen!("regalo2"), imagen!("regalo3")],
        fuente: BitmapFont::nueva(
            include_str!("../assets/fuente/arial-15.fnt"),
            include_bytes!("../assets/fuente/arial-15.png"),
        ),
    };
    let pantalla = Pantalla::nueva(ANCHO, ALTO);
    clear_background(BLACK);
    next_frame().await;
    let musica = |datos: &'static [u8]| async move {
        Musica::nueva(load_sound_from_bytes(datos).await.expect("música"))
    };
    let noche = musica(include_bytes!("../assets/musica/noche de paz.ogg")).await;
    let angel = musica(include_bytes!("../assets/musica/angel.ogg")).await;
    let mut tactil = Tactil::nuevo(TACTIL);
    let mut juego = Juego::nuevo(&recursos, noche, angel);

    loop {
        let pulsados = tactil.actualizar(&pantalla);
        let p = pantalla.a_virtual(mouse_position().into(), false);
        let entrada = Entrada {
            tocado: is_mouse_button_pressed(MouseButton::Left),
            x: p.x,
            y: p.y,
        };
        pantalla.vista(Vec2::ZERO);
        juego.render(&mut recursos, get_frame_time(), entrada);
        museo::presentar(&pantalla, &tactil, &pulsados);
        next_frame().await
    }
}
