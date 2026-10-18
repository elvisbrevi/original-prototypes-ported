// Que No Caiga — port a macroquad del juego de Construct 2 de Elvis Brevi (2014-2015).

mod datos;
mod juego;
mod runtime;

use std::collections::{HashMap, HashSet};

use macroquad::prelude::*;
use museo::{ConfigTactil, Cruceta, Pantalla, Tactil};

use crate::datos::Proyecto;
use crate::juego::Juego;
use crate::runtime::{Audio, Fuentes};

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
    // El Touch del juego usa también el ratón: los toques no deben generar clics además.
    simulate_mouse_with_touch(false);
    let proyecto = Proyecto::leer(include_str!("../assets/data.js"));
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
    let fuentes = Fuentes {
        normal: load_ttf_font_from_bytes(include_bytes!(
            "../assets/fuentes/LiberationSans-Regular.ttf"
        ))
        .expect("fuente"),
        negrita: load_ttf_font_from_bytes(include_bytes!(
            "../assets/fuentes/LiberationSans-Bold.ttf"
        ))
        .expect("fuente"),
    };
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
    let pantalla = Pantalla::nueva(proyecto.ancho, proyecto.alto);
    let mut tactil = Tactil::nuevo(TACTIL);
    let mut juego = Juego::nuevo(proyecto, texturas, fuentes, audio);
    let mut vistos: HashSet<u64> = HashSet::new();
    let mut primero = true;

    loop {
        let pulsados = tactil.actualizar(&pantalla);
        let arriba = tactil.juego_arriba();
        // Toques y ratón, entre ticks (como los eventos del DOM).
        let mut activos = Vec::new();
        let mut nuevos = 0;
        for t in touches() {
            let terminado = matches!(t.phase, TouchPhase::Ended | TouchPhase::Cancelled);
            if vistos.insert(t.id) {
                nuevos += 1;
            }
            if terminado {
                vistos.remove(&t.id);
            } else {
                activos.push(pantalla.a_virtual(t.position, arriba));
            }
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            nuevos += 1;
        }
        if is_mouse_button_down(MouseButton::Left) {
            activos.push(pantalla.a_virtual(mouse_position().into(), arriba));
        }
        juego.entrada(activos, nuevos);

        // dt del runtime: 0 en el primer tick o si pasó más de medio segundo; como mucho 0,1 s.
        let mut dt = get_frame_time() as f64;
        if primero || dt > 0.5 {
            dt = 0.0;
        } else if dt > 0.1 {
            dt = 0.1;
        }
        primero = false;
        juego.tick(dt);

        pantalla.vista(Vec2::ZERO);
        clear_background(BLACK);
        juego.dibujar();
        museo::presentar(&pantalla, &tactil, &pulsados);
        next_frame().await
    }
}
