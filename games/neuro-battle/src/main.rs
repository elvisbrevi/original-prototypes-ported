// Neuro Battle — port a macroquad del prototipo de Unity 5.1 (WebGL con IL2CPP) de Elvis Brevi
// (GameJam Santo Tomás Concepción, 2015).

mod juego;

use std::collections::HashMap;

use macroquad::prelude::*;
use museo::{Boton, ConfigTactil, Cruceta, Pantalla, Pulsados, Tactil};
use unity::audio::Audio;
use unity::datos::Proyecto;
use unity::entrada::Teclas;
use unity::gui::{FuenteGui, Gui};
use unity::render::Recursos;

use crate::juego::{ALTO, ANCHO, EntradaFrame, Juego};

fn window_conf() -> Conf {
    Conf {
        window_title: "Neuro Battle".to_owned(),
        // El lienzo de la versión WebGL: 960x600.
        window_width: ANCHO as i32,
        window_height: ALTO as i32,
        ..Default::default()
    }
}

/// En el móvil: la cruceta mueve la neurona (WASD), los botones disparan (Shift derecho), la
/// hacen girar (las flechas) y sueltan la última neurona unida (Alt izquierdo), y START es el
/// zoom (E). En el título, las instrucciones y el game over, tocar la pantalla es la E.
const TACTIL: ConfigTactil = ConfigTactil {
    cruceta: Cruceta::Completa,
    botones: &[
        Boton {
            letra: "A",
            etiqueta: "DISPARAR",
        },
        Boton {
            letra: ">",
            etiqueta: "GIRAR",
        },
        Boton {
            letra: "<",
            etiqueta: "GIRAR",
        },
        Boton {
            letra: "B",
            etiqueta: "SOLTAR",
        },
    ],
    start: Some("ZOOM"),
    zonas: &[],
};

/// Las teclas del InputManager del juego, con sus nombres de Unity.
const TECLAS: [(KeyCode, &str); 7] = [
    (KeyCode::A, "a"),
    (KeyCode::D, "d"),
    (KeyCode::S, "s"),
    (KeyCode::W, "w"),
    (KeyCode::Left, "left"),
    (KeyCode::Right, "right"),
    (KeyCode::LeftAlt, "left alt"),
];

macro_rules! texturas {
    ($($nombre:literal),* $(,)?) => {
        [$(($nombre, include_bytes!(concat!("../assets/texturas/", $nombre, ".png")).as_slice())),*]
    };
}

/// La fuente del puntaje: «Light Pixel-7» (de Style-7, gratis solo para uso personal) no se
/// copia; la reemplaza Micro 5 (SIL OFL 1.1), con las métricas verticales de la original y 2,1
/// veces más grande, para que el texto mida lo mismo.
fn fuentes(proyecto: &Proyecto) -> HashMap<String, FuenteGui> {
    let f = &proyecto.fuentes["light_pixel-7"];
    let micro = load_ttf_font_from_bytes(include_bytes!("../assets/fuentes/Micro5-Regular.ttf"))
        .expect("fuente");
    HashMap::from([(
        "light_pixel-7".to_owned(),
        FuenteGui {
            normal: micro,
            negrita: None,
            ascenso: f.ascenso / f.tamano,
            alto_linea: f.interlineado / f.tamano,
            tamano_defecto: f.tamano,
            escala: 2.1,
        },
    )])
}

#[macroquad::main(window_conf)]
async fn main() {
    // Unity siembra Random con la hora al arrancar.
    rand::srand(miniquad::date::now() as u64);
    let proyecto = Proyecto::leer(include_str!("../assets/escena.json"));
    let texturas: HashMap<String, Texture2D> = texturas!(
        "atake",
        "background_Static",
        "bala",
        "circulo",
        "cola grande",
        "colas",
        "cuadrado",
        "defensa",
        "enemigo1",
        "esquina_A",
        "game_over",
        "help_screen",
        "lateral_A",
        "pata",
        "splash",
    )
    .into_iter()
    .map(|(n, b)| (n.to_owned(), Texture2D::from_file_with_format(b, None)))
    .collect();
    let audio = Audio::cargar(&[
        ("disparo", include_bytes!("../assets/audio/disparo.ogg")),
        ("union", include_bytes!("../assets/audio/union.ogg")),
    ])
    .await;
    let mut gui = Gui::nueva(fuentes(&proyecto), texturas.clone());
    let recursos = Recursos::nuevos(texturas);
    let pantalla = Pantalla::nueva(ANCHO, ALTO);
    let mut tactil = Tactil::nuevo(TACTIL);
    let mut juego = Juego::nuevo(proyecto, audio);
    let mut antes = Pulsados::default();
    let (mut e_antes, mut disparo_antes) = (false, false);

    loop {
        let pulsados = tactil.actualizar(&pantalla);
        let mut abajo: Vec<&str> = TECLAS
            .iter()
            .filter(|(k, _)| is_key_down(*k))
            .map(|(_, n)| *n)
            .collect();
        let mut pulsadas: Vec<&str> = TECLAS
            .iter()
            .filter(|(k, _)| is_key_pressed(*k))
            .map(|(_, n)| *n)
            .collect();
        // Fire2 también es el botón derecho del ratón («mouse 1»).
        if is_mouse_button_down(MouseButton::Right) {
            abajo.push("mouse 1");
        }
        if is_mouse_button_pressed(MouseButton::Right) {
            pulsadas.push("mouse 1");
        }
        // La cruceta y los botones en pantalla hacen de teclas.
        for (activo, previo, nombre) in [
            (pulsados.izquierda, antes.izquierda, "a"),
            (pulsados.derecha, antes.derecha, "d"),
            (pulsados.arriba, antes.arriba, "w"),
            (pulsados.abajo, antes.abajo, "s"),
            (pulsados.botones[1], antes.botones[1], "right"),
            (pulsados.botones[2], antes.botones[2], "left"),
            (pulsados.botones[3], antes.botones[3], "left alt"),
        ] {
            if activo {
                abajo.push(nombre);
            }
            if activo && !previo {
                pulsadas.push(nombre);
            }
        }
        antes = pulsados;
        // En los menús, un toque fuera de los controles es la E durante un frame.
        let toque_menu = juego.nivel != 2
            && touches()
                .iter()
                .any(|t| t.phase == TouchPhase::Started && !tactil.en_control(t.position));
        let e = is_key_down(KeyCode::E) || pulsados.start || toque_menu;
        let disparo = is_key_down(KeyCode::RightShift) || pulsados.botones[0];
        let entrada = EntradaFrame {
            teclas: Teclas {
                abajo: &abajo,
                pulsadas: &pulsadas,
            },
            e,
            e_soltada: e_antes && !e,
            escape: is_key_down(KeyCode::Escape),
            disparo,
            disparo_soltado: disparo_antes && !disparo,
        };
        (e_antes, disparo_antes) = (e, disparo);
        juego.frame(get_frame_time(), &entrada);

        pantalla.vista(Vec2::ZERO);
        juego.dibujar(&recursos, &mut gui);
        museo::presentar(&pantalla, &tactil, &pulsados);
        next_frame().await
    }
}
