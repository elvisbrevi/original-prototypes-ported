// One Button Prototype — port a macroquad del prototipo de Unity 5.1 de Elvis Brevi (2015).

mod juego;

use std::collections::HashMap;

use macroquad::prelude::*;
use museo::{ConfigTactil, Cruceta, Pantalla, Tactil};
use unity::datos::Proyecto;
use unity::gui::{FuenteGui, Gui};
use unity::render::Recursos;

use crate::juego::{ALTO, ANCHO, Entrada, Juego};

fn window_conf() -> Conf {
    Conf {
        window_title: "One Button Prototype".to_owned(),
        // El lienzo de la versión WebGL: 960x600.
        window_width: ANCHO as i32,
        window_height: ALTO as i32,
        ..Default::default()
    }
}

/// Sin controles en pantalla: el único botón es el clic, y en el móvil, el toque.
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

/// Las fuentes con las métricas verticales que Unity guardó (en fracciones del tamaño): "Arial"
/// (la fuente integrada de la GUI, del sistema) es Liberation Sans, con sus mismas métricas;
/// "OpenSansRegular", la del contador de FPS, viene en el build.
fn fuentes(proyecto: &Proyecto) -> HashMap<String, FuenteGui> {
    let cargar = |b: &[u8]| load_ttf_font_from_bytes(b).expect("fuente");
    let mut f = HashMap::new();
    let arial = &proyecto.fuentes["Arial"];
    f.insert(
        "Arial".to_owned(),
        FuenteGui {
            normal: cargar(include_bytes!(
                "../../../crates/construct2/fuentes/LiberationSans-Regular.ttf"
            )),
            negrita: Some(cargar(include_bytes!(
                "../../../crates/construct2/fuentes/LiberationSans-Bold.ttf"
            ))),
            ascenso: arial.ascenso / arial.tamano,
            alto_linea: arial.interlineado / arial.tamano,
            tamano_defecto: arial.tamano,
        },
    );
    let open = &proyecto.fuentes["OpenSansRegular"];
    f.insert(
        "OpenSansRegular".to_owned(),
        FuenteGui {
            normal: cargar(include_bytes!("../assets/fuentes/OpenSans-Regular.ttf")),
            negrita: None,
            ascenso: open.ascenso / open.tamano,
            alto_linea: open.interlineado / open.tamano,
            tamano_defecto: open.tamano,
        },
    );
    f
}

#[macroquad::main(window_conf)]
async fn main() {
    let proyecto = Proyecto::leer(include_str!("../assets/escena.json"));
    let texturas: HashMap<String, Texture2D> = texturas!(
        "Back 1",
        "Back-sun",
        "Floor_1d",
        "Floor_1u",
        "Floor_2d",
        "Floor_2u",
        "Floor_3d",
        "Floor_3u",
        "Flowers",
        "Grass",
        "RobotBoyCrouchSprite",
        "RobotBoyIdleSprite",
        "RobotBoyJumpSprite",
        "RobotBoyRunSprite",
        "RobotBoyWalkSprite",
        "Roots_1-1",
        "Roots_1-1m",
        "Roots_2-2",
        "Roots_2-3",
        "Roots_3-1",
        "Roots_3-1m",
        "Roots_4-1",
        "Roots_4-2",
        "Roots_4-2m",
        "box",
        "bridge",
        "bridge-c1",
        "bridge-c2",
        "bridge-c3",
        "bridge-c4",
        "bridge-c5",
        "bridge-c6",
        "bridgeL",
        "bridgeR",
        "button",
        "button active",
        "button hover",
        "button on",
        "button on hover",
        "death trap asset sheet",
        "dragon_walk",
        "foliage",
        "foliage1",
        "lianas",
        "star-md",
        "window",
        "window on",
    )
    .into_iter()
    .map(|(n, b)| (n.to_owned(), Texture2D::from_file_with_format(b, None)))
    .collect();
    let mut gui = Gui::nueva(fuentes(&proyecto), texturas.clone());
    let recursos = Recursos::nuevos(texturas);
    let pantalla = Pantalla::nueva(ANCHO, ALTO);
    let mut tactil = Tactil::nuevo(TACTIL);
    let mut juego = Juego::nuevo(proyecto);
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
            tecla_p: is_key_pressed(KeyCode::P),
            control: is_key_down(KeyCode::LeftControl),
        };
        juego.frame(get_frame_time(), entrada);

        pantalla.vista(Vec2::ZERO);
        juego.dibujar(&recursos, &mut gui, entrada);
        museo::presentar(&pantalla, &tactil, &pulsados);
        next_frame().await
    }
}
