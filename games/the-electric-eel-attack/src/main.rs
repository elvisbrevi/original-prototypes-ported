// The Electric Eel Attack — port a macroquad del juego XNA 3.1 de Elvis Brevi (2011).

mod assets;
mod audio;
mod control;
mod motor;
mod niveles;

use std::collections::HashMap;

use macroquad::prelude::*;
use museo::{Boton, ConfigTactil, Cruceta, Pantalla, PasoFijo, Tactil};

use crate::assets::Assets;
use crate::audio::{Banco, Musica};
use crate::control::Control;
use crate::motor::Globales;
use crate::motor::shader::Materiales;
use crate::niveles::{Contexto, Nivel, Tipo};

/// Resolución de los mapas (y del back buffer tras Pantalla.iniciar).
const ANCHO: f32 = 640.0;
const ALTO: f32 = 480.0;

fn window_conf() -> Conf {
    Conf {
        window_title: "The Electric Eel Attack".to_owned(),
        window_width: ANCHO as i32,
        window_height: ALTO as i32,
        ..Default::default()
    }
}

/// Flechas para andar, A dispara (Z), B salta (X), R reinicia el nivel y START empieza (Enter).
const TACTIL: ConfigTactil = ConfigTactil {
    cruceta: Cruceta::Horizontal,
    botones: &[
        Boton {
            letra: "A",
            etiqueta: "DISPARO",
        },
        Boton {
            letra: "B",
            etiqueta: "SALTO",
        },
        Boton {
            letra: "R",
            etiqueta: "REINICIO",
        },
    ],
    start: Some("START"),
    zonas: &[],
};

/// Cuántas instancias de cada sonido pide el nivel que más pide (ver audio::Banco).
fn instancias_necesarias(assets: &Assets) -> HashMap<String, usize> {
    let mut maximo: HashMap<String, usize> = HashMap::new();
    for xml in assets.mapas.values() {
        let documento = roxmltree::Document::parse(xml).expect("mapa XML");
        let mut cuenta: HashMap<String, usize> = HashMap::new();
        for sonido in documento
            .descendants()
            .filter(|n| n.tag_name().name() == "Sonidos")
            .flat_map(|n| n.children().filter(|s| s.is_element()))
        {
            let nombre = sonido.attributes().next().expect("Nombre").value();
            *cuenta.entry(nombre.to_owned()).or_default() += 1;
        }
        for (nombre, n) in cuenta {
            let m = maximo.entry(nombre).or_default();
            *m = (*m).max(n);
        }
    }
    maximo
}

#[macroquad::main(window_conf)]
async fn main() {
    let assets = Assets::cargar();
    let pantalla = Pantalla::nueva(ANCHO, ALTO);
    let materiales = Materiales::cargar();
    // Cargando: el navegador decodifica los sonidos de forma asíncrona.
    clear_background(BLACK);
    next_frame().await;
    let mut banco = Banco::cargar(&assets, &instancias_necesarias(&assets)).await;
    let mut musica = Musica::nueva(&assets);
    let mut glob = Globales::nuevas();
    let mut paso = PasoFijo::xna();
    let mut tactil = Tactil::nuevo(TACTIL);
    let mut control = Control::default();
    let mut pantalla_completa = false;

    // Game1.LoadContent: los cinco niveles, y se carga el título.
    let mut niveles: Vec<Option<Nivel>> = [
        Tipo::Titulo,
        Tipo::Nivel1,
        Tipo::Nivel2,
        Tipo::Nivel3,
        Tipo::Nivel4,
    ]
    .map(|t| Some(Nivel::nuevo(t)))
    .into();
    let mut nivel_activo = 0;
    niveles[0]
        .as_mut()
        .expect("título")
        .cargar(&mut glob, &assets, &mut banco);
    // Para probar el port (no existe en el original): EEL_NIVEL=n empieza en el nivel n.
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(n) = std::env::var("EEL_NIVEL")
        .ok()
        .and_then(|n| n.parse::<usize>().ok())
        .filter(|n| (1..5).contains(n))
    {
        nivel_activo = n;
        niveles[n]
            .as_mut()
            .expect("nivel")
            .cargar(&mut glob, &assets, &mut banco);
    }

    loop {
        // Solo se tiene cargada la canción que toca (cancion1 hasta el nivel 2, cancion2 después).
        musica.preparar(if nivel_activo < 3 { 0 } else { 1 }).await;
        let pulsados = tactil.actualizar(&pantalla);
        for _ in 0..paso.pasos() {
            // Game1.Update.
            if nivel_activo < 3 {
                if !musica.sonando(0) {
                    musica.stop(1);
                    musica.activas[0] = true;
                }
            } else if !musica.sonando(1) {
                musica.stop(0);
                musica.activas[1] = true;
            }
            musica.reproducir_activas();
            control.actualizar(&pulsados);
            // En el original Esc alterna la pantalla completa (no sale del juego).
            if control.press_esc {
                pantalla_completa = !pantalla_completa;
                set_fullscreen(pantalla_completa);
            }
            let propio = nivel_activo;
            let mut nivel = niveles[propio].take().expect("nivel activo");
            let mut ctx = Contexto {
                propio,
                nivel_activo: &mut nivel_activo,
                otros: &mut niveles,
                glob: &mut glob,
                assets: &assets,
                banco: &mut banco,
            };
            nivel.actualizar(&control, &mut ctx);
            niveles[propio] = Some(nivel);
        }

        // Game1.Draw.
        niveles[nivel_activo]
            .as_ref()
            .expect("nivel activo")
            .dibujar(&mut glob, &assets, &pantalla, &materiales);
        museo::presentar(&pantalla, &tactil, &pulsados);
        next_frame().await
    }
}
