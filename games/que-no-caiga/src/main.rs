// Que No Caiga — port a macroquad del juego de Construct 2 de Elvis Brevi (2014-2015).

mod juego;

use std::collections::HashMap;

use construct2::{Audio, Clase, Fuentes, Mundo, Proyecto, Toques, dt_del_runtime};
use macroquad::prelude::*;
use museo::{ConfigTactil, Cruceta, Pantalla, Tactil};

use crate::juego::Juego;

fn window_conf() -> Conf {
    Conf {
        window_title: "Que No Caiga".to_owned(),
        // El canvas del original: 400x500.
        window_width: 400,
        window_height: 500,
        ..Default::default()
    }
}

/// Sin controles en pantalla: el juego se juega tocándolo (el objeto Touch del original).
const TACTIL: ConfigTactil = ConfigTactil {
    cruceta: Cruceta::Ninguna,
    botones: &[],
    start: None,
    zonas: &[],
};

macro_rules! imagenes {
    ($($nombre:literal),* $(,)?) => {
        [$(($nombre, include_bytes!(concat!("../assets/", $nombre)).as_slice())),*]
    };
}

#[macroquad::main(window_conf)]
async fn main() {
    // Construct 2 usa Math.random, que cambia en cada partida.
    rand::srand(miniquad::date::now() as u64);
    // El Touch del juego usa también el ratón: los toques no deben generar clics además.
    simulate_mouse_with_touch(false);
    // El comportamiento Platform es el constructor 8 de la tabla de referencias del runtime.
    let proyecto = Proyecto::leer(include_str!("../assets/data.js"), &[(8, Clase::Plataforma)]);
    let texturas: HashMap<String, Texture2D> = imagenes!(
        "images/bomba-sheet0.png",
        "images/bomba-sheet1.png",
        "images/botonjugar-sheet0.png",
        "images/cuadrogameover-sheet0.png",
        "images/fondo-sheet0.png",
        "images/gameover-sheet0.png",
        "images/jugador1-sheet0.png",
        "images/jugador1-sheet1.png",
        "images/limitecorrecto-sheet0.png",
        "images/limite-sheet0.png",
        "images/pelota-sheet0.png",
        "images/reiniciar-sheet0.png",
        "images/sprite3-sheet0.png",
        "images/sprite4-sheet0.png",
        "images/sprite5-sheet0.png",
        "images/sprite6-sheet0.png",
    )
    .into_iter()
    .map(|(n, b)| (n.to_owned(), Texture2D::from_file_with_format(b, None)))
    .collect();
    let fuentes = Fuentes::liberation();
    clear_background(BLACK);
    next_frame().await;
    let audio = Audio::cargar(&[
        ("aparicion", include_bytes!("../assets/media/aparicion.ogg")),
        ("caida", include_bytes!("../assets/media/caida.ogg")),
        ("click", include_bytes!("../assets/media/click.ogg")),
        ("explosion", include_bytes!("../assets/media/explosion.ogg")),
        ("pegada", include_bytes!("../assets/media/pegada.ogg")),
    ])
    .await;
    let pantalla = Pantalla::nueva(proyecto.ancho as f32, proyecto.alto as f32);
    let mut tactil = Tactil::nuevo(TACTIL);
    let mut juego = Juego::nuevo(Mundo::nuevo(proyecto, texturas, fuentes), audio);
    let mut toques = Toques::default();
    let mut primero = true;

    loop {
        let pulsados = tactil.actualizar(&pantalla);
        let arriba = tactil.juego_arriba();
        // Toques y ratón, entre ticks (como los eventos del DOM).
        juego.entrada(&toques.leer(|p| pantalla.a_virtual(p, arriba), true));

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
