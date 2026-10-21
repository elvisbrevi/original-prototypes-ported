// Go Ball — port a macroquad del juego de Unity 3.4 de Elvis Brevi (2011).

mod itween;
mod juego;
mod recursos;

use std::collections::HashMap;

use macroquad::prelude::*;
use museo::{Boton, ConfigTactil, Cruceta, Pantalla, Pulsados, Tactil};
use unity::audio3d::Audio3d;
use unity::datos3d::Proyecto3d;
use unity::gui::Gui;
use unity::malla::Malla;
use unity::render3d::Render3d;

use crate::juego::{ALTO, ANCHO, EntradaFrame, Juego};

fn window_conf() -> Conf {
    Conf {
        window_title: "Go Ball".to_owned(),
        // La resolución por defecto del build de escritorio: 1024x600.
        window_width: ANCHO as i32,
        window_height: ALTO as i32,
        ..Default::default()
    }
}

/// Sin teclado: la cruceta mueve la bola y el botón la hace saltar (las flechas y la de arriba).
/// Fuera de los controles, la pantalla se toca como con el ratón: los botones de la GUI y
/// arrastrar las cajas.
const TACTIL: ConfigTactil = ConfigTactil {
    cruceta: Cruceta::Horizontal,
    botones: &[Boton {
        letra: "A",
        etiqueta: "SALTAR",
    }],
    start: None,
    zonas: &[],
};

/// Las teclas que usa el InputManager del juego, con sus nombres de Unity.
const TECLAS: [(KeyCode, &str); 6] = [
    (KeyCode::Left, "left"),
    (KeyCode::Right, "right"),
    (KeyCode::A, "a"),
    (KeyCode::D, "d"),
    (KeyCode::Up, "up"),
    (KeyCode::W, "w"),
];

/// Las texturas que dibuja la GUI: la piel, las fuentes y el GUITexture de la cámara.
fn texturas_gui(proyecto: &Proyecto3d) -> HashMap<String, Texture2D> {
    let mut nombres: Vec<String> = proyecto
        .piel
        .estilos
        .values()
        .flat_map(|e| {
            [
                &e.normal,
                &e.hover,
                &e.active,
                &e.on_normal,
                &e.on_hover,
                &e.on_active,
            ]
            .into_iter()
            .filter_map(|s| s.fondo.clone())
        })
        .collect();
    nombres.extend(proyecto.fuentes.values().filter_map(|f| f.textura.clone()));
    nombres.push("pixel".to_owned());
    recursos::TEXTURAS
        .iter()
        .filter(|(n, _)| nombres.iter().any(|m| m == n))
        .map(|(n, b)| {
            (
                n.to_string(),
                Texture2D::from_file_with_format(b, Some(ImageFormat::Png)),
            )
        })
        .collect()
}

#[macroquad::main(window_conf)]
async fn main() {
    // Unity siembra Random con la hora al arrancar (las partículas lo usan).
    rand::srand(miniquad::date::now() as u64);
    let proyecto = Proyecto3d::leer(include_str!("../assets/escena.json"));
    let mallas: HashMap<String, Malla> = recursos::MALLAS
        .iter()
        .map(|(n, b)| (n.to_string(), Malla::leer(b)))
        .collect();
    let archivos: HashMap<String, &[u8]> = recursos::TEXTURAS
        .iter()
        .map(|(n, b)| (n.to_string(), *b))
        .collect();
    let pantalla = Pantalla::nueva_3d(ANCHO, ALTO);
    let mut render = Render3d::nuevo(&proyecto, &mallas, &archivos, ANCHO, ALTO);
    let clips: Vec<(&str, &[u8], f32)> = proyecto
        .audio
        .iter()
        .filter_map(|(n, c)| {
            let (_, b) = recursos::AUDIO.iter().find(|(a, _)| *a == c.archivo)?;
            Some((n.as_str(), *b, c.duracion))
        })
        .collect();
    let audio = Audio3d::cargar(&clips).await;
    let texturas = texturas_gui(&proyecto);
    let fuentes_mapa = proyecto
        .fuentes
        .iter()
        .filter_map(|(n, f)| {
            let t = texturas.get(f.textura.as_ref()?)?.clone();
            Some((n.clone(), (t, f.clone())))
        })
        .collect();
    let mut gui = Gui::nueva(HashMap::new(), texturas.clone()).con_fuentes_mapa(fuentes_mapa);
    let pixel = texturas.get("pixel").cloned();
    let mut juego = Juego::nuevo(proyecto, &mallas, audio, pixel);
    let mut tactil = Tactil::nuevo(TACTIL);
    let mut antes = Pulsados::default();
    // El dedo que hace de ratón (el primero que no cae sobre un control) y si estaba apretado.
    let mut dedo: Option<u64> = None;
    let mut dedo_apretado = false;

    loop {
        let pulsados = tactil.actualizar(&pantalla);
        let toques = touches();
        let arriba = tactil.juego_arriba();
        let (raton, pulsado, apretado, soltado) = if toques.is_empty() && dedo.is_none() {
            (
                Some(pantalla.a_virtual(mouse_position().into(), arriba)),
                is_mouse_button_pressed(MouseButton::Left),
                is_mouse_button_down(MouseButton::Left),
                is_mouse_button_released(MouseButton::Left),
            )
        } else {
            // Con los dedos, el ratón es el dedo que empezó fuera de los controles.
            if dedo.is_none_or(|d| !toques.iter().any(|t| t.id == d)) {
                dedo = toques
                    .iter()
                    .find(|t| t.phase == TouchPhase::Started && !tactil.en_control(t.position))
                    .map(|t| t.id);
            }
            match dedo.and_then(|d| toques.iter().find(|t| t.id == d)) {
                Some(t) => {
                    let terminado = matches!(t.phase, TouchPhase::Ended | TouchPhase::Cancelled);
                    let r = (
                        Some(pantalla.a_virtual(t.position, arriba)),
                        t.phase == TouchPhase::Started,
                        !terminado,
                        terminado && dedo_apretado,
                    );
                    if terminado {
                        dedo = None;
                    }
                    r
                }
                None => {
                    dedo = None;
                    (None, false, false, dedo_apretado)
                }
            }
        };
        dedo_apretado = apretado;
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
        // La cruceta y el botón en pantalla hacen de flechas.
        for (activo, previo, nombre) in [
            (pulsados.izquierda, antes.izquierda, "left"),
            (pulsados.derecha, antes.derecha, "right"),
            (pulsados.botones[0], antes.botones[0], "up"),
        ] {
            if activo {
                abajo.push(nombre);
            }
            if activo && !previo {
                pulsadas.push(nombre);
            }
        }
        antes = pulsados;
        let entrada = EntradaFrame {
            teclas_abajo: abajo,
            teclas_pulsadas: pulsadas,
            pulsado,
            apretado,
            soltado,
            raton,
        };
        juego.frame(get_frame_time(), &entrada);
        if let Some(completa) = juego.pedidos.pantalla_completa.take() {
            set_fullscreen(completa);
        }
        if juego.pedidos.salir && !museo::WEB {
            break;
        }
        juego.pedidos.salir = false;

        juego.dibujar(&mut render, pantalla.pase());
        pantalla.vista(Vec2::ZERO);
        juego.gui(&mut gui, &entrada);
        museo::presentar(&pantalla, &tactil, &pulsados);
        next_frame().await
    }
}
