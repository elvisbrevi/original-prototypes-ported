// Da Car — port a macroquad del juego de Construct 2 de Elvis Brevi (2015).

mod juego;

use std::collections::HashMap;

use construct2::{Audio, Clase, Fuentes, Mundo, Proyecto, Toques, dt_del_runtime};
use macroquad::prelude::*;
use museo::{ConfigTactil, Cruceta, Pantalla, Tactil};

use crate::juego::{Juego, Teclas};

fn window_conf() -> Conf {
    Conf {
        window_title: "Da Car".to_owned(),
        // El canvas del original: 800x450.
        window_width: 800,
        window_height: 450,
        ..Default::default()
    }
}

/// Sin controles en pantalla: en el móvil se acelera tocando la pantalla y se gira inclinando el
/// dispositivo (Touch.Beta), como en el original.
const TACTIL: ConfigTactil = ConfigTactil {
    cruceta: Cruceta::Ninguna,
    botones: &[],
    start: None,
    zonas: &[],
};

/// Los comportamientos, por su índice en la tabla de referencias del runtime (lc()).
const COMPORTAMIENTOS: [(u64, Clase); 3] = [
    (7, Clase::Fisica),
    (8, Clase::Envolver),
    (9, Clase::Desplazar),
];

macro_rules! imagenes {
    ($($nombre:literal),* $(,)?) => {
        [$(($nombre, include_bytes!(concat!("../assets/", $nombre)).as_slice())),*]
    };
}

fn teclas() -> Teclas {
    Teclas {
        arriba: is_key_down(KeyCode::Up),
        izquierda: is_key_down(KeyCode::Left),
        derecha: is_key_down(KeyCode::Right),
        w: is_key_down(KeyCode::W),
        a: is_key_down(KeyCode::A),
        d: is_key_down(KeyCode::D),
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Construct 2 usa Math.random, que cambia en cada partida.
    rand::srand(miniquad::date::now() as u64);
    // El Touch del juego usa también el ratón: los toques no deben generar clics además.
    simulate_mouse_with_touch(false);
    let proyecto = Proyecto::leer(include_str!("../assets/data.js"), &COMPORTAMIENTOS);
    let texturas: HashMap<String, Texture2D> = imagenes!(
        "images/botoncompartir-sheet0.png",
        "images/botonreintentar-sheet0.png",
        "images/cabeza-sheet0.png",
        "images/cerro1-sheet0.png",
        "images/cerro2-sheet0.png",
        "images/cerro3-sheet0.png",
        "images/cerro4-sheet0.png",
        "images/ground-sheet0.png",
        "images/mapameta-sheet0.png",
        "images/meta-sheet0.png",
        "images/motomapa-sheet0.png",
        "images/moto-sheet0.png",
        "images/particulaspolvo.png",
        "images/rueda1-sheet0.png",
        "images/spawn-sheet0.png",
    )
    .into_iter()
    .map(|(n, b)| (n.to_owned(), Texture2D::from_file_with_format(b, None)))
    .collect();
    let fuentes = Fuentes::liberation();
    clear_background(BLACK);
    next_frame().await;
    let audio = Audio::cargar(&[
        ("acelerar", include_bytes!("../assets/media/acelerar.ogg")),
        ("motor2", include_bytes!("../assets/media/motor2.ogg")),
    ])
    .await;
    let pantalla = Pantalla::nueva(proyecto.ancho as f32, proyecto.alto as f32);
    let mut tactil = Tactil::nuevo(TACTIL);
    // System.IsOnMobile: el original miraba el navegador; aquí, si hay pantalla táctil.
    let movil = tactil.activo;
    let mut juego = Juego::nuevo(Mundo::nuevo(proyecto, texturas, fuentes), audio, movil);
    let mut toques = Toques::default();
    let mut primero = true;

    loop {
        let pulsados = tactil.actualizar(&pantalla);
        let arriba = tactil.juego_arriba();
        // Toques, ratón, teclado y la inclinación, entre ticks (como los eventos del DOM).
        let entrada = toques.leer(|p| pantalla.a_virtual(p, arriba), true);
        juego.entrada(&entrada, teclas(), museo::sensores::beta());

        let dt = dt_del_runtime(get_frame_time() as f64, primero);
        primero = false;
        juego.tick(dt);

        pantalla.vista(Vec2::ZERO);
        clear_background(BLACK);
        juego.dibujar();
        museo::presentar(&pantalla, &tactil, &pulsados);
        next_frame().await
    }
}
