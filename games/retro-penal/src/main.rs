// Retro Penal — port a macroquad del juego de Construct 2 de Elvis Brevi (2015).

mod juego;

use std::collections::HashMap;

use construct2::{Audio, Clase, Fuentes, Mundo, Proyecto, Toques, dt_del_runtime};
use macroquad::prelude::*;
use museo::{ConfigTactil, Cruceta, Pantalla, Tactil};

use crate::juego::Juego;

fn window_conf() -> Conf {
    Conf {
        window_title: "Retro Penal".to_owned(),
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

/// Los comportamientos, por su índice en la tabla de referencias del runtime (lc()).
const COMPORTAMIENTOS: [(u64, Clase); 5] = [
    (5, Clase::Torreta),
    (6, Clase::Movimiento),
    (7, Clase::Bala),
    (8, Clase::Solido),
    (9, Clase::Envolver),
];

macro_rules! imagenes {
    ($($nombre:literal),* $(,)?) => {
        [$(($nombre, include_bytes!(concat!("../assets/", $nombre)).as_slice())),*]
    };
}

#[macroquad::main(window_conf)]
async fn main() {
    // El Touch del juego usa también el ratón: los toques no deben generar clics además.
    simulate_mouse_with_touch(false);
    let proyecto = Proyecto::leer(include_str!("../assets/data.js"), &COMPORTAMIENTOS);
    let texturas: HashMap<String, Texture2D> = imagenes!(
        "images/afuera-sheet0.png",
        "images/animaciongol-sheet0.png",
        "images/arcoizquierdo-sheet0.png",
        "images/arco-sheet0.png",
        "images/arquero-sheet0.png",
        "images/barra-sheet0.png",
        "images/boton_empezar-sheet0.png",
        "images/cortina-sheet0.png",
        "images/cuadrogol-sheet0.png",
        "images/degradacion-sheet0.png",
        "images/flecha-sheet0.png",
        "images/fondo-sheet0.png",
        "images/icono_intento-sheet0.png",
        "images/icono_null-sheet0.png",
        "images/imagen_gameover-sheet0.png",
        "images/jugador-sheet0.png",
        "images/jugador-sheet1.png",
        "images/logo_juego-sheet0.png",
        "images/pasto-sheet0.png",
        "images/pelota-sheet0.png",
        "images/posicionarquero-sheet0.png",
        "images/punto-sheet0.png",
        "images/tapempezar-sheet0.png",
    )
    .into_iter()
    .map(|(n, b)| (n.to_owned(), Texture2D::from_file_with_format(b, None)))
    .collect();
    let fuentes = Fuentes::liberation();
    clear_background(BLACK);
    next_frame().await;
    let audio = Audio::cargar(&[
        ("click3", include_bytes!("../assets/media/click3.ogg")),
        ("gol3", include_bytes!("../assets/media/gol3.ogg")),
        ("musica3", include_bytes!("../assets/media/musica3.ogg")),
        ("perdio3", include_bytes!("../assets/media/perdio3.ogg")),
        ("publico3", include_bytes!("../assets/media/publico3.ogg")),
        (
            "seloperdio3",
            include_bytes!("../assets/media/seloperdio3.ogg"),
        ),
        ("tiro3", include_bytes!("../assets/media/tiro3.ogg")),
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
