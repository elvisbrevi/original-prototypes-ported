// Super Happy Jump — port a macroquad del juego de Unity 4.6 (Web Player) de Elvis Brevi (2014).

mod juego;

use std::collections::HashMap;

use macroquad::prelude::*;
use museo::{ConfigTactil, Cruceta, Pantalla, Preferencias, Tactil};
use unity::audio::Audio;
use unity::datos::Proyecto;
use unity::gui::{FuenteGui, Gui};
use unity::render::Recursos;

use crate::juego::{ALTO, ANCHO, Entrada, Juego};

fn window_conf() -> Conf {
    Conf {
        window_title: "Super Happy Jump".to_owned(),
        // El tamaño del Web Player: 960x600.
        window_width: ANCHO as i32,
        window_height: ALTO as i32,
        ..Default::default()
    }
}

/// Sin controles en pantalla: se juega tocando (el toque es el clic del original).
const TACTIL: ConfigTactil = ConfigTactil {
    cruceta: Cruceta::Ninguna,
    botones: &[],
    start: None,
    zonas: &[],
};

macro_rules! texturas {
    ($($nombre:literal),* $(,)?) => {
        [$(($nombre, include_bytes!(concat!("../assets/texturas/", $nombre, ".png")).as_slice())),*]
    };
}

/// Las fuentes de la GUI con las métricas verticales de FreeType (en fracciones del tamaño):
/// "Arial" (la fuente incorporada de Unity, del sistema) es Liberation Sans, con sus mismas
/// métricas; "Arial Rounded Bold" (Arial Rounded MT Bold, incrustada en el original) es Nunito,
/// con el ascenso y el interlineado que Unity guardó de la original.
fn fuentes(proyecto: &Proyecto) -> HashMap<String, FuenteGui> {
    let cargar = |b: &[u8]| load_ttf_font_from_bytes(b).expect("fuente");
    let mut f = HashMap::new();
    f.insert(
        "Arial".to_owned(),
        FuenteGui {
            normal: cargar(include_bytes!(
                "../../../crates/construct2/fuentes/LiberationSans-Regular.ttf"
            )),
            negrita: Some(cargar(include_bytes!(
                "../../../crates/construct2/fuentes/LiberationSans-Bold.ttf"
            ))),
            // hhea de Arial: ascenso 1854, descenso 434 e interlínea 67 de 2048.
            ascenso: 1854.0 / 2048.0,
            alto_linea: (1854.0 + 434.0 + 67.0) / 2048.0,
            // La Arial integrada de Unity es de 13.
            tamano_defecto: 13.0,
            escala: 1.0,
        },
    );
    let redondeada = &proyecto.fuentes["Arial Rounded Bold"];
    f.insert(
        "Arial Rounded Bold".to_owned(),
        FuenteGui {
            normal: cargar(include_bytes!("../fuentes/Nunito-ExtraBold.ttf")),
            negrita: Some(cargar(include_bytes!("../fuentes/Nunito-Black.ttf"))),
            ascenso: redondeada.ascenso / redondeada.tamano,
            alto_linea: redondeada.interlineado / redondeada.tamano,
            tamano_defecto: redondeada.tamano,
            escala: 1.0,
        },
    );
    f
}

#[macroquad::main(window_conf)]
async fn main() {
    // Unity siembra Random con la hora al arrancar.
    rand::srand(miniquad::date::now() as u64);
    let proyecto = Proyecto::leer(include_str!("../assets/escena.json"));
    let texturas: HashMap<String, Texture2D> = texturas!(
        "bach",
        "base_titulo",
        "button_play",
        "cerca",
        "CFX_T_Anim4_Triangle3",
        "CFX_T_Smoke_4Frames",
        "cinta",
        "coin",
        "hero",
        "label_record",
        "muro",
        "murob",
        "pixel",
        "pixel-large",
        "suelo",
        "titulo",
        "window",
    )
    .into_iter()
    .map(|(n, b)| (n.to_owned(), Texture2D::from_file_with_format(b, None)))
    .collect();
    let mut gui = Gui::nueva(fuentes(&proyecto), texturas.clone());
    let recursos = Recursos::nuevos(texturas);
    clear_background(BLACK);
    next_frame().await;
    let audio = Audio::cargar(&[
        ("click", include_bytes!("../assets/audio/click.wav")),
        ("coin", include_bytes!("../assets/audio/coin.wav")),
        ("dead", include_bytes!("../assets/audio/dead.wav")),
        ("jump", include_bytes!("../assets/audio/jump.wav")),
        ("step", include_bytes!("../assets/audio/step.wav")),
    ])
    .await;
    let pantalla = Pantalla::nueva(ANCHO, ALTO);
    let mut tactil = Tactil::nuevo(TACTIL);
    let mut juego = Juego::nuevo(proyecto, audio, Preferencias::de("super-happy-jump"));
    // Si el último puntero fue un dedo que ya se levantó, no hay hover.
    let mut con_dedo = false;

    loop {
        let pulsados = tactil.actualizar(&pantalla);
        let toques = touches();
        if !toques.is_empty() {
            con_dedo = true;
        } else if mouse_delta_position() != Vec2::ZERO {
            con_dedo = false;
        }
        let raton = (!toques.is_empty() || !con_dedo)
            .then(|| pantalla.a_virtual(mouse_position().into(), false));
        let entrada = Entrada {
            pulsado: is_mouse_button_pressed(MouseButton::Left),
            soltado: is_mouse_button_released(MouseButton::Left),
            raton,
        };
        juego.frame(get_frame_time(), entrada);

        pantalla.vista(Vec2::ZERO);
        juego.dibujar(&recursos, &mut gui, entrada);
        museo::presentar(&pantalla, &tactil, &pulsados);
        next_frame().await
    }
}
